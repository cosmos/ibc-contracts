//! A bounded actor owns admission, pending and running deduplication, and result fan-out.
use crate::SchedulerConfig;
use std::{
    collections::{BTreeMap, VecDeque},
    future::Future,
    pin::Pin,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{mpsc, oneshot, watch, Semaphore},
    task::JoinSet,
    time::{timeout, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Route {
    pub source_chain_id: u64,
    pub source_router: [u8; 20],
    pub destination_light_client: [u8; 20],
    pub proof_height: u64,
}
#[derive(Clone, Debug)]
pub struct Intent {
    pub path: Vec<Vec<u8>>,
    pub expected_commitment: [u8; 32],
}
impl Intent {
    fn digest(&self) -> [u8; 32] {
        ibc_storage::path_digest(&self.path)
    }
}

pub type WorkFuture<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;
pub trait BatchExecutor: Send + Sync + 'static {
    type Output: Send + Sync + 'static;
    fn prove(&self, route: Route, entries: Vec<Intent>) -> WorkFuture<Self::Output>;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchError {
    Capacity,
    Conflict,
    Stopped,
    Proof(String),
}
type ResultValue<O> = Option<Result<Arc<O>, BatchError>>;
type Waiter<O> = watch::Receiver<ResultValue<O>>;
type Completion<O> = (Route, Vec<[u8; 32]>, Result<Arc<O>, BatchError>);
type IntentKey = (Route, [u8; 32]);
struct Entry<O> {
    intent: Intent,
    enqueued: Instant,
    result: watch::Sender<ResultValue<O>>,
}
struct Submission<O> {
    route: Route,
    intents: Vec<Intent>,
    reply: oneshot::Sender<Result<Vec<Waiter<O>>, BatchError>>,
}

pub struct Batcher<B: BatchExecutor> {
    submit: mpsc::Sender<Submission<B::Output>>,
    stop: watch::Sender<bool>,
    finished: watch::Receiver<bool>,
}
impl<B: BatchExecutor> Clone for Batcher<B> {
    fn clone(&self) -> Self {
        Self {
            submit: self.submit.clone(),
            stop: self.stop.clone(),
            finished: self.finished.clone(),
        }
    }
}
impl<B: BatchExecutor> Batcher<B> {
    pub fn start(
        config: SchedulerConfig,
        max_jobs: usize,
        permits: Arc<Semaphore>,
        job_timeout: Duration,
        backend: Arc<B>,
    ) -> Result<Self, BatchError> {
        crate::Scheduler::new(config.clone()).map_err(|_| BatchError::Conflict)?;
        if max_jobs == 0 {
            return Err(BatchError::Capacity);
        }
        let (submit, receiver) = mpsc::channel(64);
        let (stop, stopping) = watch::channel(false);
        let (done, finished) = watch::channel(false);
        tokio::spawn(async move {
            run(
                config,
                max_jobs,
                permits,
                job_timeout,
                backend,
                receiver,
                stopping,
            )
            .await;
            let _ = done.send(true);
        });
        Ok(Self {
            submit,
            stop,
            finished,
        })
    }

    pub async fn prove(
        &self,
        route: Route,
        intents: Vec<Intent>,
    ) -> Result<Vec<Arc<B::Output>>, BatchError> {
        if intents.is_empty() {
            return Ok(Vec::new());
        }
        let (reply, response) = oneshot::channel();
        self.submit
            .send(Submission {
                route,
                intents,
                reply,
            })
            .await
            .map_err(|_| BatchError::Stopped)?;
        let waiters = response.await.map_err(|_| BatchError::Stopped)??;
        let mut output = Vec::with_capacity(waiters.len());
        for mut waiter in waiters {
            loop {
                if let Some(result) = waiter.borrow_and_update().clone() {
                    output.push(result?);
                    break;
                }
                waiter.changed().await.map_err(|_| BatchError::Stopped)?;
            }
        }
        Ok(output)
    }

    #[must_use]
    pub fn is_stopped(&self) -> bool {
        *self.finished.borrow()
    }

    pub fn stop(&self) {
        let _ = self.stop.send(true);
    }
    pub async fn stopped(&self) {
        let mut finished = self.finished.clone();
        while !*finished.borrow_and_update() {
            if finished.changed().await.is_err() {
                break;
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn run<B: BatchExecutor>(
    config: SchedulerConfig,
    max_jobs: usize,
    permits: Arc<Semaphore>,
    job_timeout: Duration,
    backend: Arc<B>,
    mut submissions: mpsc::Receiver<Submission<B::Output>>,
    mut stop: watch::Receiver<bool>,
) {
    let mut entries: BTreeMap<IntentKey, Entry<B::Output>> = BTreeMap::new();
    let mut queues: BTreeMap<Route, VecDeque<[u8; 32]>> = BTreeMap::new();
    let mut jobs: JoinSet<Completion<B::Output>> = JoinSet::new();
    let mut tick = tokio::time::interval(Duration::from_millis(25));
    loop {
        tokio::select! {
            _ = stop.changed() => { break; }
            submission = submissions.recv() => {
                let Some(submission) = submission else { break; };
                if submission.reply.is_closed() { continue; }
                let mut unique = BTreeMap::new();
                let mut failure = None;
                for intent in &submission.intents {
                    let digest = intent.digest();
                    if let Some(previous) = unique.insert(digest, intent.expected_commitment) {
                        if previous != intent.expected_commitment { failure = Some(BatchError::Conflict); }
                    }
                    if entries.get(&(submission.route.clone(), digest)).is_some_and(|e| e.intent.expected_commitment != intent.expected_commitment) {
                        failure = Some(BatchError::Conflict);
                    }
                }
                let new_count = unique.keys().filter(|d| !entries.contains_key(&(submission.route.clone(), **d))).count();
                if entries.len().saturating_add(new_count) > config.max_pending_intents { failure = Some(BatchError::Capacity); }
                if let Some(error) = failure { let _ = submission.reply.send(Err(error)); continue; }
                let mut waiters = Vec::with_capacity(submission.intents.len());
                for intent in submission.intents {
                    let digest = intent.digest();
                    let entry = entries.entry((submission.route.clone(), digest)).or_insert_with(|| {
                        queues.entry(submission.route.clone()).or_default().push_back(digest);
                        let (result, _) = watch::channel(None);
                        Entry { intent, enqueued: Instant::now(), result }
                    });
                    waiters.push(entry.result.subscribe());
                }
                let _ = submission.reply.send(Ok(waiters));
            }
            completed = jobs.join_next(), if !jobs.is_empty() => {
                match completed {
                    Some(Ok((route, keys, result))) => {
                        for key in keys {
                            if let Some(entry) = entries.remove(&(route.clone(), key)) { let _ = entry.result.send(Some(result.clone())); }
                        }
                    }
                    _ => break, // A panicking worker must not leave waiters stuck forever.
                }
            }
            _ = tick.tick() => {}
        }
        queues.retain(|route, queue| {
            queue.retain(|key| {
                let identity = (route.clone(), *key);
                if entries
                    .get(&identity)
                    .is_some_and(|e| e.result.receiver_count() == 0)
                {
                    entries.remove(&identity);
                    false
                } else {
                    true
                }
            });
            !queue.is_empty()
        });
        while jobs.len() < max_jobs {
            let selected = queues
                .iter()
                .filter(|(_, queue)| !queue.is_empty())
                .filter_map(|(route, queue)| {
                    let entry = entries.get(&(route.clone(), *queue.front()?))?;
                    let age = entry.enqueued.elapsed();
                    (queue.len() >= config.preferred_batch_size
                        || queue.len() >= config.max_batch_size
                        || age >= Duration::from_millis(config.max_batch_wait_ms))
                    .then_some((route.clone(), entry.enqueued))
                })
                .min_by_key(|(_, time)| *time)
                .map(|(route, _)| route);
            let Some(route) = selected else {
                break;
            };
            let Some(queue) = queues.get_mut(&route) else {
                break;
            };
            let mut keys = Vec::new();
            for _ in 0..config.max_batch_size {
                if let Some(key) = queue.pop_front() {
                    keys.push(key);
                } else {
                    break;
                }
            }
            if queue.is_empty() {
                queues.remove(&route);
            }
            keys.sort_unstable();
            let intents = keys
                .iter()
                .filter_map(|key| {
                    entries
                        .get(&(route.clone(), *key))
                        .map(|e| e.intent.clone())
                })
                .collect();
            let backend = backend.clone();
            let permits = permits.clone();
            jobs.spawn(async move {
                let work = async {
                    let _permit = permits
                        .acquire_owned()
                        .await
                        .map_err(|_| BatchError::Stopped)?;
                    backend
                        .prove(route.clone(), intents)
                        .await
                        .map(Arc::new)
                        .map_err(BatchError::Proof)
                };
                let result = timeout(job_timeout, work).await.unwrap_or_else(|_| {
                    Err(BatchError::Proof("proof job deadline exceeded".into()))
                });
                (route, keys, result)
            });
        }
    }
    for (_, entry) in entries {
        let _ = entry.result.send(Some(Err(BatchError::Stopped)));
    }
    jobs.abort_all();
    while jobs.join_next().await.is_some() {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Backend {
        calls: Arc<AtomicUsize>,
        gate: Arc<Semaphore>,
    }
    impl BatchExecutor for Backend {
        type Output = Vec<[u8; 32]>;
        fn prove(&self, _: Route, entries: Vec<Intent>) -> WorkFuture<Self::Output> {
            let calls = self.calls.clone();
            let gate = self.gate.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                let _permit = gate.acquire().await.map_err(|e| e.to_string())?;
                Ok(entries.into_iter().map(|i| i.digest()).collect())
            })
        }
    }
    fn route(height: u64) -> Route {
        Route {
            source_chain_id: 1,
            source_router: [1; 20],
            destination_light_client: [2; 20],
            proof_height: height,
        }
    }
    fn intent(sequence: u64) -> Intent {
        Intent {
            path: ibc_storage::terminal_path(b"client-1", ibc_storage::PathKind::Packet, sequence),
            expected_commitment: [1; 32],
        }
    }
    fn batcher(
        preferred: usize,
        capacity: usize,
        gate: Arc<Semaphore>,
        calls: Arc<AtomicUsize>,
    ) -> Batcher<Backend> {
        Batcher::start(
            SchedulerConfig {
                preferred_batch_size: preferred,
                max_pending_intents: capacity,
                max_batch_wait_ms: 5,
                ..SchedulerConfig::default()
            },
            1,
            Arc::new(Semaphore::new(1)),
            Duration::from_secs(2),
            Arc::new(Backend { calls, gate }),
        )
        .unwrap_or_else(|_| unreachable!())
    }
    #[tokio::test]
    async fn splits_eighty_and_returns_each_result_in_order() -> Result<(), BatchError> {
        let calls = Arc::new(AtomicUsize::new(0));
        let batcher = batcher(10, 100, Arc::new(Semaphore::new(1)), calls.clone());
        let inputs = (1..=80).map(intent).collect::<Vec<_>>();
        let result = batcher.prove(route(12), inputs.clone()).await?;
        assert_eq!(result.len(), 80);
        for (item, output) in inputs.iter().zip(result) {
            assert!(output.contains(&item.digest()));
            assert!(output.len() <= 32);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        batcher.stop();
        batcher.stopped().await;
        Ok(())
    }
    #[tokio::test]
    async fn running_duplicates_survive_cancel_and_conflicts_are_atomic() -> Result<(), BatchError>
    {
        let calls = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(Semaphore::new(0));
        let batcher = batcher(1, 1, gate.clone(), calls.clone());
        let first = {
            let b = batcher.clone();
            tokio::spawn(async move { b.prove(route(12), vec![intent(1)]).await })
        };
        while calls.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
        let mut conflicting = intent(1);
        conflicting.expected_commitment = [99; 32];
        assert!(matches!(
            batcher.prove(route(12), vec![conflicting]).await,
            Err(BatchError::Conflict)
        ));
        assert!(matches!(
            batcher.prove(route(13), vec![intent(1)]).await,
            Err(BatchError::Capacity)
        ));
        let second = {
            let b = batcher.clone();
            tokio::spawn(async move { b.prove(route(12), vec![intent(1)]).await })
        };
        tokio::time::sleep(Duration::from_millis(10)).await;
        first.abort();
        gate.add_permits(1);
        assert_eq!(second.await.map_err(|_| BatchError::Stopped)??.len(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(batcher.prove(route(13), vec![intent(2)]).await?.len(), 1);
        batcher.stop();
        batcher.stopped().await;
        Ok(())
    }
    #[tokio::test]
    async fn shutdown_releases_pending_and_running_waiters() -> Result<(), BatchError> {
        let batcher = batcher(
            1,
            10,
            Arc::new(Semaphore::new(0)),
            Arc::new(AtomicUsize::new(0)),
        );
        let task = {
            let b = batcher.clone();
            tokio::spawn(async move { b.prove(route(12), vec![intent(1)]).await })
        };
        tokio::time::sleep(Duration::from_millis(10)).await;
        batcher.stop();
        batcher.stopped().await;
        assert!(matches!(
            task.await.map_err(|_| BatchError::Stopped)?,
            Err(BatchError::Stopped)
        ));
        Ok(())
    }
    #[tokio::test]
    async fn timed_out_job_releases_capacity_without_automatic_resubmission(
    ) -> Result<(), BatchError> {
        let calls = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(Semaphore::new(0));
        let batcher = Batcher::start(
            SchedulerConfig {
                preferred_batch_size: 1,
                max_pending_intents: 1,
                ..SchedulerConfig::default()
            },
            1,
            Arc::new(Semaphore::new(1)),
            Duration::from_millis(20),
            Arc::new(Backend {
                calls: calls.clone(),
                gate: gate.clone(),
            }),
        )?;
        assert!(matches!(
            batcher.prove(route(12), vec![intent(1)]).await,
            Err(BatchError::Proof(_))
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        gate.add_permits(1);
        assert_eq!(batcher.prove(route(12), vec![intent(2)]).await?.len(), 1);
        batcher.stop();
        batcher.stopped().await;
        Ok(())
    }
}
