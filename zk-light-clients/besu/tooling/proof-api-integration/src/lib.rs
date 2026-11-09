#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use besu_zk_types::MAX_BATCH_MESSAGES;
use ibc_storage::path_digest;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PREFERRED_BATCH_SIZE: usize = 10;
pub const DEFAULT_MAX_BATCH_SIZE: usize = MAX_BATCH_MESSAGES;
pub const DEFAULT_MAX_BATCH_WAIT_MS: u64 = 3_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerConfig {
    pub preferred_batch_size: usize,
    pub max_batch_size: usize,
    pub max_batch_wait_ms: u64,
    pub allow_parallel_update_and_membership: bool,
    pub max_pending_intents: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            preferred_batch_size: DEFAULT_PREFERRED_BATCH_SIZE,
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
            max_batch_wait_ms: DEFAULT_MAX_BATCH_WAIT_MS,
            allow_parallel_update_and_membership: true,
            max_pending_intents: 1_024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct RouteId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MembershipIntent {
    pub route: RouteId,
    pub source_router: [u8; 20],
    pub path: Vec<Vec<u8>>,
    pub expected_commitment: [u8; 32],
    pub minimum_source_height: u64,
    pub operation_id: String,
    pub enqueued_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenCohort {
    pub route: RouteId,
    pub proof_height: u64,
    pub intents: Vec<MembershipIntent>,
    pub update_required: bool,
    /// Only this cohort may launch the update; other cohorts share the existing job.
    pub update_job_claimed: bool,
    pub allow_parallel_update_and_membership: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulerError {
    InvalidConfig,
    QueueFull,
    Duplicate,
    EmptyCohort,
    IncompatibleRoute,
    HeightTooLow,
    ConflictingCommitment,
}

pub struct Scheduler {
    config: SchedulerConfig,
    pending: BTreeMap<RouteId, VecDeque<MembershipIntent>>,
    dedupe: BTreeSet<(RouteId, [u8; 32], [u8; 32])>,
    active_updates: BTreeSet<(RouteId, u64)>,
    pending_count: usize,
}

impl Scheduler {
    pub fn new(config: SchedulerConfig) -> Result<Self, SchedulerError> {
        if config.preferred_batch_size == 0
            || config.preferred_batch_size > config.max_batch_size
            || config.max_batch_size == 0
            || config.max_batch_size > MAX_BATCH_MESSAGES
            || config.max_pending_intents == 0
        {
            return Err(SchedulerError::InvalidConfig);
        }
        Ok(Self {
            config,
            pending: BTreeMap::new(),
            dedupe: BTreeSet::new(),
            active_updates: BTreeSet::new(),
            pending_count: 0,
        })
    }

    pub fn enqueue(&mut self, intent: MembershipIntent) -> Result<(), SchedulerError> {
        if self.pending_count >= self.config.max_pending_intents {
            return Err(SchedulerError::QueueFull);
        }
        let digest = path_digest(&intent.path);
        if self.pending.get(&intent.route).is_some_and(|queue| {
            queue
                .iter()
                .any(|existing| existing.source_router != intent.source_router)
        }) {
            return Err(SchedulerError::IncompatibleRoute);
        }
        if self.dedupe.iter().any(|(route, path, commitment)| {
            route == &intent.route && *path == digest && *commitment != intent.expected_commitment
        }) {
            return Err(SchedulerError::ConflictingCommitment);
        }
        let key = (intent.route.clone(), digest, intent.expected_commitment);
        if !self.dedupe.insert(key) {
            return Err(SchedulerError::Duplicate);
        }
        self.pending
            .entry(intent.route.clone())
            .or_default()
            .push_back(intent);
        self.pending_count += 1;
        Ok(())
    }

    #[must_use]
    pub fn should_seal(&self, route: &RouteId, now_ms: u64, prover_available: bool) -> bool {
        let Some(queue) = self.pending.get(route) else {
            return false;
        };
        queue.len() >= self.config.max_batch_size
            || (prover_available && queue.len() >= self.config.preferred_batch_size)
            || queue.front().is_some_and(|oldest| {
                now_ms.saturating_sub(oldest.enqueued_at_ms) >= self.config.max_batch_wait_ms
            })
    }

    pub fn freeze(
        &mut self,
        route: &RouteId,
        proof_height: u64,
        consensus_state_exists: bool,
    ) -> Result<FrozenCohort, SchedulerError> {
        let queue = self
            .pending
            .get_mut(route)
            .ok_or(SchedulerError::EmptyCohort)?;
        let count = queue.len().min(self.config.max_batch_size);
        if count == 0 {
            return Err(SchedulerError::EmptyCohort);
        }
        // Validate before changing any queue, dedupe, or capacity state.
        if queue
            .iter()
            .take(count)
            .any(|intent| proof_height < intent.minimum_source_height)
        {
            return Err(SchedulerError::HeightTooLow);
        }
        let mut intents = Vec::with_capacity(count);
        for _ in 0..count {
            let intent = queue.pop_front().ok_or(SchedulerError::EmptyCohort)?;
            self.dedupe.remove(&(
                route.clone(),
                path_digest(&intent.path),
                intent.expected_commitment,
            ));
            intents.push(intent);
            self.pending_count -= 1;
        }
        let update_required = !consensus_state_exists;
        let update_job_claimed =
            update_required && self.active_updates.insert((route.clone(), proof_height));
        Ok(FrozenCohort {
            route: route.clone(),
            proof_height,
            intents,
            update_required,
            update_job_claimed,
            allow_parallel_update_and_membership: self.config.allow_parallel_update_and_membership,
        })
    }

    /// Release a claim on both success and failure, allowing a subsequent retry.
    pub fn complete_update_job(&mut self, route: &RouteId, height: u64) -> bool {
        self.active_updates.remove(&(route.clone(), height))
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending_count
    }

    /// Claims are owned by the caller that created a cohort with `update_job_claimed`,
    /// or by the first independent update job to claim the same route and height.
    #[must_use]
    pub fn claim_update_job(&mut self, route: &RouteId, height: u64) -> bool {
        self.active_updates.insert((route.clone(), height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent(index: u64) -> MembershipIntent {
        MembershipIntent {
            route: RouteId("route-a".to_owned()),
            source_router: [1; 20],
            path: vec![format!("client-0\x01{index:08}").into_bytes()],
            expected_commitment: [index.to_le_bytes()[0]; 32],
            minimum_source_height: 10,
            operation_id: index.to_string(),
            enqueued_at_ms: 1_000,
        }
    }

    #[test]
    fn defaults_dedupe_wait_batch_freeze_and_update_dedupe() {
        let mut scheduler =
            Scheduler::new(SchedulerConfig::default()).unwrap_or_else(|_| unreachable!());
        let route = RouteId("route-a".to_owned());
        scheduler
            .enqueue(intent(1))
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(scheduler.enqueue(intent(1)), Err(SchedulerError::Duplicate));
        assert!(!scheduler.should_seal(&route, 3_999, true));
        assert!(scheduler.should_seal(&route, 4_000, false));
        let cohort = scheduler
            .freeze(&route, 10, false)
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(cohort.intents.len(), 1);
        assert!(cohort.update_required);
        assert!(!scheduler.claim_update_job(&route, 10));
    }

    #[test]
    fn rejects_oversized_configuration_and_applies_backpressure() {
        let invalid = SchedulerConfig {
            max_batch_size: 33,
            ..SchedulerConfig::default()
        };
        assert!(matches!(
            Scheduler::new(invalid),
            Err(SchedulerError::InvalidConfig)
        ));
        let config = SchedulerConfig {
            max_pending_intents: 1,
            ..SchedulerConfig::default()
        };
        let mut scheduler = Scheduler::new(config).unwrap_or_else(|_| unreachable!());
        scheduler
            .enqueue(intent(1))
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(scheduler.enqueue(intent(2)), Err(SchedulerError::QueueFull));
    }
    #[test]
    fn failed_freeze_preserves_every_intent_and_dedupe_entry() {
        let mut scheduler =
            Scheduler::new(SchedulerConfig::default()).unwrap_or_else(|_| unreachable!());
        let route = RouteId("route-a".to_owned());
        scheduler
            .enqueue(intent(1))
            .unwrap_or_else(|_| unreachable!());
        let mut later = intent(2);
        later.minimum_source_height = 20;
        scheduler.enqueue(later).unwrap_or_else(|_| unreachable!());
        assert_eq!(
            scheduler.freeze(&route, 10, false),
            Err(SchedulerError::HeightTooLow)
        );
        assert_eq!(scheduler.pending_count(), 2);
        assert_eq!(scheduler.enqueue(intent(1)), Err(SchedulerError::Duplicate));
        assert!(scheduler.claim_update_job(&route, 10));
        let cohort = scheduler
            .freeze(&route, 20, false)
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(cohort.intents.len(), 2);
        assert_eq!(cohort.intents[0], intent(1));
        assert_eq!(scheduler.pending_count(), 0);
        assert!(scheduler.complete_update_job(&route, 20));
        assert!(scheduler.claim_update_job(&route, 20));
    }

    #[test]
    fn rejects_route_and_commitment_conflicts_without_consuming_capacity() {
        let mut scheduler =
            Scheduler::new(SchedulerConfig::default()).unwrap_or_else(|_| unreachable!());
        scheduler
            .enqueue(intent(1))
            .unwrap_or_else(|_| unreachable!());
        let mut conflict = intent(1);
        conflict.expected_commitment = [99; 32];
        assert_eq!(
            scheduler.enqueue(conflict),
            Err(SchedulerError::ConflictingCommitment)
        );
        let mut incompatible = intent(2);
        incompatible.source_router = [99; 20];
        assert_eq!(
            scheduler.enqueue(incompatible),
            Err(SchedulerError::IncompatibleRoute)
        );
        assert_eq!(scheduler.pending_count(), 1);
    }
    #[test]
    fn only_one_cohort_owns_an_update_until_it_completes() {
        let mut scheduler =
            Scheduler::new(SchedulerConfig::default()).unwrap_or_else(|_| unreachable!());
        let route = RouteId("route-a".to_owned());
        scheduler
            .enqueue(intent(1))
            .unwrap_or_else(|_| unreachable!());
        let first = scheduler
            .freeze(&route, 10, false)
            .unwrap_or_else(|_| unreachable!());
        scheduler
            .enqueue(intent(2))
            .unwrap_or_else(|_| unreachable!());
        let second = scheduler
            .freeze(&route, 10, false)
            .unwrap_or_else(|_| unreachable!());
        assert!(first.update_required && first.update_job_claimed);
        assert!(second.update_required && !second.update_job_claimed);
        assert!(scheduler.complete_update_job(&route, 10));
        assert!(!scheduler.complete_update_job(&route, 10));
        assert!(scheduler.claim_update_job(&route, 10));
    }
}

pub mod runtime;
