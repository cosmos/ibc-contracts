#![forbid(unsafe_code)]

use std::{
    env,
    error::Error,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Duration,
};

static ARTIFACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub type E2eResult<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
pub struct Direction {
    pub source_name: &'static str,
    pub destination_name: &'static str,
    pub source_rpc: String,
    pub destination_rpc: String,
    pub source_chain_id: u64,
    pub source_router: String,
    pub destination_router: String,
    pub source_app: String,
    pub destination_app: String,
    pub destination_light_client: String,
    pub source_key: String,
    pub destination_key: String,
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub sequence: u64,
    pub timeout_timestamp: u64,
    pub payload: Vec<u8>,
    pub source_port: String,
    pub destination_port: String,
}

impl Direction {
    pub fn a_to_b() -> E2eResult<Self> {
        Self::load("chainA", "chainB", "chain-a", "chain-b", 13_371)
    }

    pub fn b_to_a() -> E2eResult<Self> {
        Self::load("chainB", "chainA", "chain-b", "chain-a", 13_372)
    }

    fn load(
        source_json: &'static str,
        destination_json: &'static str,
        source_name: &'static str,
        destination_name: &'static str,
        source_chain_id: u64,
    ) -> E2eResult<Self> {
        require_enabled()?;
        let deployments = deployments_path()?;
        let state_root = deployments
            .parent()
            .ok_or("E2E deployments file has no parent directory")?;
        Ok(Self {
            source_name,
            destination_name,
            source_rpc: rpc_for(source_name),
            destination_rpc: rpc_for(destination_name),
            source_chain_id,
            source_router: jq_file(&deployments, &format!(".{source_json}.router"))?,
            destination_router: jq_file(&deployments, &format!(".{destination_json}.router"))?,
            source_app: jq_file(&deployments, &format!(".{source_json}.testApp"))?,
            destination_app: jq_file(&deployments, &format!(".{destination_json}.testApp"))?,
            destination_light_client: jq_file(
                &deployments,
                &format!(".{destination_json}.lightClient"),
            )?,
            source_key: read_secret(&state_root.join(source_name).join("deployer.key"))?,
            destination_key: read_secret(&state_root.join(destination_name).join("deployer.key"))?,
        })
    }

    pub fn update_to(&self, target_height: u64) -> E2eResult<PathBuf> {
        let trusted_height = self.latest_height()?;
        if target_height <= trusted_height {
            return Err(format!(
                "target height {target_height} must exceed trusted height {trusted_height}"
            )
            .into());
        }
        let artifact = artifact_path("update-client")?;
        let target = target_height.to_string();
        let trusted = trusted_height.to_string();
        run_prover(
            self,
            "prove-update",
            "update-client.elf",
            &[
                "--trusted-height",
                &trusted,
                "--target-height",
                &target,
                "--output",
                path_str(&artifact)?,
            ],
        )?;
        let envelope = jq_file(&artifact, ".proofEnvelope")?;
        cast_send(
            &self.destination_rpc,
            &self.destination_key,
            &self.destination_router,
            "updateClient(string,bytes)",
            &["client-0", &envelope],
            true,
        )?;
        let stored = self.latest_height()?;
        if stored < target_height {
            return Err(format!(
                "light client stored height {stored}, expected at least {target_height}"
            )
            .into());
        }
        Ok(artifact)
    }

    pub fn update_to_new_head(&self) -> E2eResult<u64> {
        let trusted = self.latest_height()?;
        let target = wait_for_height_above(&self.source_rpc, trusted)?;
        self.update_to(target)?;
        Ok(target)
    }

    pub fn latest_height(&self) -> E2eResult<u64> {
        cast_call_u64(
            &self.destination_rpc,
            &self.destination_light_client,
            "latestHeight()(uint64)",
            &[],
        )
    }

    pub fn send_packet(&self, timeout_timestamp: u64, payload: &[u8]) -> E2eResult<(Packet, u64)> {
        // The live scenario tests share chains but Rust does not guarantee their discovery order.
        // Ensure this transaction is mined above the direction's current trusted height so a
        // preceding scenario cannot make the resulting membership height stale.
        let trusted_height = self.latest_height()?;
        wait_for_height_above(&self.source_rpc, trusted_height)?;

        let payload_hex = encode_hex(payload);
        let destination_port = self.destination_app.to_ascii_lowercase();
        let timeout = timeout_timestamp.to_string();
        let signature = "sendPacket(string,string,uint64,bytes)(uint64)";
        let args = [
            "client-0",
            destination_port.as_str(),
            timeout.as_str(),
            payload_hex.as_str(),
        ];
        let sequence = cast_call_u64(&self.source_rpc, &self.source_app, signature, &args)?;
        let receipt = cast_send(
            &self.source_rpc,
            &self.source_key,
            &self.source_app,
            signature,
            &args,
            true,
        )?;
        let height = receipt_height(&receipt)?;
        Ok((
            Packet {
                sequence,
                timeout_timestamp,
                payload: payload.to_vec(),
                source_port: self.source_app.to_ascii_lowercase(),
                destination_port,
            },
            height,
        ))
    }

    pub fn prove_packet_membership(&self, packet: &Packet, height: u64) -> E2eResult<PathBuf> {
        self.prove_membership(packet, height, 0x01)
    }

    pub fn prove_ack_membership(&self, packet: &Packet, height: u64) -> E2eResult<PathBuf> {
        self.prove_membership(packet, height, 0x03)
    }

    fn prove_membership(&self, packet: &Packet, height: u64, path_type: u8) -> E2eResult<PathBuf> {
        let path = ibc_path(path_type, packet.sequence);
        let commitment = cast_call(
            &self.source_rpc,
            &self.source_router,
            "getCommitment(bytes32)(bytes32)",
            &[&keccak(&path)?],
        )?;
        if is_zero_word(&commitment) {
            return Err("source Router commitment is absent at the requested path".into());
        }
        let artifact = artifact_path("membership")?;
        let height_arg = height.to_string();
        run_prover(
            self,
            "prove-membership",
            "membership.elf",
            &[
                "--proof-height",
                &height_arg,
                "--path",
                &path,
                "--expected-commitment",
                &commitment,
                "--output",
                path_str(&artifact)?,
            ],
        )?;
        Ok(artifact)
    }

    pub fn prove_receipt_absence(&self, packet: &Packet, height: u64) -> E2eResult<PathBuf> {
        let path = ibc_path(0x02, packet.sequence);
        let commitment = cast_call(
            &self.source_rpc,
            &self.source_router,
            "getCommitment(bytes32)(bytes32)",
            &[&keccak(&path)?],
        )?;
        if !is_zero_word(&commitment) {
            return Err("receipt is present; refusing to request a non-membership proof".into());
        }
        let artifact = artifact_path("non-membership")?;
        let height_arg = height.to_string();
        run_prover(
            self,
            "prove-non-membership",
            "non-membership.elf",
            &[
                "--proof-height",
                &height_arg,
                "--path",
                &path,
                "--output",
                path_str(&artifact)?,
            ],
        )?;
        Ok(artifact)
    }

    pub fn recv_packet(&self, packet: &Packet, proof: &Path, height: u64) -> E2eResult<u64> {
        let envelope = jq_file(proof, ".proofEnvelope")?;
        let receipt = self.recv_packet_message(packet, &envelope, height, true)?;
        receipt_height(&receipt)
    }

    pub fn reject_recv_packet_replay(
        &self,
        packet: &Packet,
        proof: &Path,
        height: u64,
    ) -> E2eResult<()> {
        let envelope = jq_file(proof, ".proofEnvelope")?;
        self.recv_packet_message(packet, &envelope, height, false)?;
        Ok(())
    }

    pub fn reject_packet_context_mutations(
        &self,
        packet: &Packet,
        proof: &Path,
        height: u64,
    ) -> E2eResult<()> {
        let envelope = jq_file(proof, ".proofEnvelope")?;
        let mut fake_proof = envelope.clone();
        mutate_hex_byte(&mut fake_proof)?;
        self.recv_packet_message(packet, &fake_proof, height, false)?;

        for word in 1..=5 {
            let mut changed = envelope.clone();
            mutate_enveloped_public_word(&mut changed, word)?;
            self.recv_packet_message(packet, &changed, height, false)?;
        }

        let mut wrong_sequence = packet.clone();
        wrong_sequence.sequence = wrong_sequence
            .sequence
            .checked_add(1)
            .ok_or("packet sequence overflow")?;
        self.recv_packet_message(&wrong_sequence, &envelope, height, false)?;

        let mut wrong_payload = packet.clone();
        wrong_payload.payload.push(0xff);
        self.recv_packet_message(&wrong_payload, &envelope, height, false)?;

        let wrong_height = height.checked_add(1).ok_or("proof height overflow")?;
        self.recv_packet_message(packet, &envelope, wrong_height, false)?;
        Ok(())
    }

    pub fn reject_non_membership_proof_on_membership_path(
        &self,
        packet: &Packet,
        height: u64,
    ) -> E2eResult<()> {
        let proof = self.prove_receipt_absence(packet, height)?;
        let envelope = jq_file(&proof, ".proofEnvelope")?;
        self.recv_packet_message(packet, &envelope, height, false)?;
        Ok(())
    }

    fn recv_packet_message(
        &self,
        packet: &Packet,
        envelope: &str,
        height: u64,
        expect_success: bool,
    ) -> E2eResult<String> {
        let message = recv_message(packet, envelope, height);
        cast_send(
            &self.destination_rpc,
            &self.destination_key,
            &self.destination_router,
            "recvPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)))",
            &[&message],
            expect_success,
        )
    }

    pub fn acknowledge_packet(
        &self,
        packet: &Packet,
        acknowledgement: &[u8],
        proof: &Path,
        height: u64,
    ) -> E2eResult<()> {
        self.acknowledge_packet_expect(packet, acknowledgement, proof, height, true)
    }

    pub fn acknowledge_packet_expect(
        &self,
        packet: &Packet,
        acknowledgement: &[u8],
        proof: &Path,
        height: u64,
        expect_success: bool,
    ) -> E2eResult<()> {
        let envelope = jq_file(proof, ".proofEnvelope")?;
        let message = ack_message(packet, acknowledgement, &envelope, height);
        cast_send(
            &self.destination_rpc,
            &self.destination_key,
            &self.destination_router,
            "ackPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,bytes,(uint64,uint64)))",
            &[&message],
            expect_success,
        )?;
        Ok(())
    }

    pub fn timeout_packet(
        &self,
        packet: &Packet,
        proof: &Path,
        height: u64,
        expect_success: bool,
    ) -> E2eResult<()> {
        let envelope = jq_file(proof, ".proofEnvelope")?;
        let message = timeout_message(packet, &envelope, height);
        cast_send(
            &self.destination_rpc,
            &self.destination_key,
            &self.destination_router,
            "timeoutPacket(((uint64,string,string,uint64,(string,string,string,string,bytes)[]),bytes,(uint64,uint64)))",
            &[&message],
            expect_success,
        )?;
        Ok(())
    }

    pub fn verifier_rejects_mutated_update(&self) -> E2eResult<()> {
        let trusted = self.latest_height()?;
        let target = wait_for_height_above(&self.source_rpc, trusted)?;
        let artifact = self.update_artifact_only(trusted, target)?;
        let mut envelope = jq_file(&artifact, ".proofEnvelope")?;
        mutate_hex_byte(&mut envelope)?;
        cast_send(
            &self.destination_rpc,
            &self.destination_key,
            &self.destination_router,
            "updateClient(string,bytes)",
            &["client-0", &envelope],
            false,
        )?;
        Ok(())
    }

    fn update_artifact_only(&self, trusted: u64, target: u64) -> E2eResult<PathBuf> {
        let artifact = artifact_path("invalid-update")?;
        let target_arg = target.to_string();
        let trusted_arg = trusted.to_string();
        run_prover(
            self,
            "prove-update",
            "update-client.elf",
            &[
                "--trusted-height",
                &trusted_arg,
                "--target-height",
                &target_arg,
                "--output",
                path_str(&artifact)?,
            ],
        )?;
        Ok(artifact)
    }
}

pub fn complete_packet_and_ack(direction: &Direction, payload: &[u8]) -> E2eResult<Packet> {
    let timeout = latest_timestamp(&direction.destination_rpc)? + 3_600;
    let (packet, packet_height) = direction.send_packet(timeout, payload)?;
    direction.update_to(packet_height)?;
    let membership = direction.prove_packet_membership(&packet, packet_height)?;
    let ack_height = direction.recv_packet(&packet, &membership, packet_height)?;

    let reverse = if direction.source_name == "chain-a" {
        Direction::b_to_a()?
    } else {
        Direction::a_to_b()?
    };
    reverse.update_to(ack_height)?;
    let ack_proof = reverse.prove_ack_membership(&packet, ack_height)?;
    let mut acknowledgement = b"ack:".to_vec();
    acknowledgement.extend_from_slice(payload);
    let mut wrong_acknowledgement = acknowledgement.clone();
    wrong_acknowledgement.push(0xff);
    reverse.acknowledge_packet_expect(
        &packet,
        &wrong_acknowledgement,
        &ack_proof,
        ack_height,
        false,
    )?;
    reverse.acknowledge_packet(&packet, &acknowledgement, &ack_proof, ack_height)?;
    Ok(packet)
}

pub fn assert_app_payload(rpc: &str, app: &str, expected: &[u8]) -> E2eResult<()> {
    let actual = cast_call(rpc, app, "lastPayload()(bytes)", &[])?;
    if actual.to_ascii_lowercase() != encode_hex(expected) {
        return Err(format!("application payload mismatch: {actual}").into());
    }
    Ok(())
}

pub fn assert_app_ack(rpc: &str, app: &str, expected: &[u8]) -> E2eResult<()> {
    let actual = cast_call(rpc, app, "lastAcknowledgement()(bytes)", &[])?;
    if actual.to_ascii_lowercase() != encode_hex(expected) {
        return Err(format!("application acknowledgement mismatch: {actual}").into());
    }
    Ok(())
}

pub fn assert_app_timed_out(rpc: &str, app: &str) -> E2eResult<()> {
    let actual = cast_call(rpc, app, "timedOut()(bool)", &[])?;
    if actual != "true" {
        return Err(format!("expected timedOut=true, got {actual}").into());
    }
    Ok(())
}

pub fn wait_for_timestamp(rpc: &str, minimum: u64) -> E2eResult<u64> {
    for _ in 0..120 {
        let height = block_number(rpc)?;
        let timestamp = block_timestamp(rpc, height)?;
        if timestamp >= minimum {
            return Ok(height);
        }
        thread::sleep(Duration::from_secs(1));
    }
    Err(format!("chain did not reach timestamp {minimum}").into())
}

pub fn latest_timestamp(rpc: &str) -> E2eResult<u64> {
    let height = block_number(rpc)?;
    block_timestamp(rpc, height)
}

pub fn receipt_path(sequence: u64) -> String {
    ibc_path(0x02, sequence)
}

fn require_enabled() -> E2eResult<()> {
    if env::var("RUN_BESU_E2E").as_deref() != Ok("1") {
        return Err("RUN_BESU_E2E=1 is required for the heavy E2E suite".into());
    }
    Ok(())
}

fn deployments_path() -> E2eResult<PathBuf> {
    let value = env::var("E2E_DEPLOYMENTS")?;
    let path = PathBuf::from(value);
    if !path.is_file() {
        return Err(format!("E2E deployments file is missing: {}", path.display()).into());
    }
    Ok(path)
}

fn rpc_for(chain: &str) -> String {
    match chain {
        "chain-a" => {
            env::var("CHAIN_A_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:18545".into())
        }
        "chain-b" => {
            env::var("CHAIN_B_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:28545".into())
        }
        _ => unreachable!("fixed E2E chain name"),
    }
}

fn read_secret(path: &Path) -> E2eResult<String> {
    let value = fs::read_to_string(path)?.trim().to_owned();
    if !value.starts_with("0x") || value.len() != 66 {
        return Err(format!("invalid generated E2E key: {}", path.display()).into());
    }
    Ok(value)
}

fn repo_root() -> E2eResult<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "E2E crate has no repository parent".into())
}

fn artifact_path(program: &str) -> E2eResult<PathBuf> {
    let root = deployments_path()?
        .parent()
        .ok_or("E2E deployments file has no parent")?
        .join("proofs");
    fs::create_dir_all(&root)?;
    let sequence = ARTIFACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(root.join(format!("{program}-{}-{sequence}.json", std::process::id())))
}

fn run_prover(direction: &Direction, command: &str, elf: &str, tail: &[&str]) -> E2eResult<()> {
    let root = repo_root()?;
    let elf_path = root.join("artifacts").join(elf);
    let mut args = vec![
        command.to_owned(),
        "--source-rpc".to_owned(),
        direction.source_rpc.clone(),
        "--destination-rpc".to_owned(),
        direction.destination_rpc.clone(),
        "--expected-source-chain-id".to_owned(),
        direction.source_chain_id.to_string(),
        "--source-validator-mode".to_owned(),
        "header-qbft".to_owned(),
        "--proof-mode".to_owned(),
        "groth16".to_owned(),
        "--trusting-period-seconds".to_owned(),
        env::var("E2E_TRUSTING_PERIOD_SECONDS").unwrap_or_else(|_| "3600".to_owned()),
        "--max-clock-drift-seconds".to_owned(),
        env::var("E2E_MAX_CLOCK_DRIFT_SECONDS").unwrap_or_else(|_| "30".to_owned()),
        "--elf".to_owned(),
        path_str(&elf_path)?.to_owned(),
    ];
    if command == "prove-update" {
        args.extend([
            "--destination-light-client".to_owned(),
            direction.destination_light_client.clone(),
        ]);
    } else {
        args.extend([
            "--source-router".to_owned(),
            direction.source_router.clone(),
            "--destination-light-client".to_owned(),
            direction.destination_light_client.clone(),
        ]);
    }
    args.extend(tail.iter().map(|value| (*value).to_owned()));
    if let Some(prover_bin) = env::var_os("E2E_PROVER_BIN") {
        run_status_checked(prover_bin, args.iter().map(String::as_str))?;
    } else {
        let manifest = root.join("Cargo.toml");
        let mut cargo_args = vec![
            "run".to_owned(),
            "--quiet".to_owned(),
            "--locked".to_owned(),
            "--release".to_owned(),
            "--manifest-path".to_owned(),
            path_str(&manifest)?.to_owned(),
            "-p".to_owned(),
            "prover-script".to_owned(),
            "--bin".to_owned(),
            "prover-script".to_owned(),
            "--".to_owned(),
        ];
        cargo_args.extend(args);
        run_status_checked("cargo", cargo_args.iter().map(String::as_str))?;
    }
    Ok(())
}

fn cast_call(rpc: &str, address: &str, signature: &str, args: &[&str]) -> E2eResult<String> {
    let mut command_args = vec!["call", "--rpc-url", rpc, address, signature];
    command_args.extend_from_slice(args);
    let output = run_checked(cast_bin(), command_args, true)?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn cast_call_u64(rpc: &str, address: &str, signature: &str, args: &[&str]) -> E2eResult<u64> {
    let value = cast_call(rpc, address, signature, args)?;
    value
        .split_whitespace()
        .next()
        .ok_or_else(|| Box::<dyn Error>::from("cast call returned no value"))?
        .parse()
        .map_err(Into::into)
}

fn cast_send(
    rpc: &str,
    private_key: &str,
    address: &str,
    signature: &str,
    args: &[&str],
    expect_success: bool,
) -> E2eResult<String> {
    let mut command_args = vec![
        "send",
        "--rpc-url",
        rpc,
        "--private-key",
        private_key,
        "--json",
        address,
        signature,
    ];
    command_args.extend_from_slice(args);
    let output = run_checked(cast_bin(), command_args, expect_success)?;
    if expect_success {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Ok(String::new())
    }
}

fn block_number(rpc: &str) -> E2eResult<u64> {
    let output = run_checked(cast_bin(), ["block-number", "--rpc-url", rpc], true)?;
    String::from_utf8(output.stdout)?
        .trim()
        .parse()
        .map_err(Into::into)
}

fn block_timestamp(rpc: &str, height: u64) -> E2eResult<u64> {
    let height_arg = height.to_string();
    let output = run_checked(
        cast_bin(),
        ["block", &height_arg, "--rpc-url", rpc, "--json"],
        true,
    )?;
    let value = jq_input(&String::from_utf8(output.stdout)?, ".timestamp")?;
    parse_quantity(&value)
}

fn wait_for_height_above(rpc: &str, minimum: u64) -> E2eResult<u64> {
    for _ in 0..120 {
        let height = block_number(rpc)?;
        if height > minimum {
            return Ok(height);
        }
        thread::sleep(Duration::from_secs(1));
    }
    Err(format!("chain height did not advance beyond {minimum}").into())
}

fn receipt_height(receipt: &str) -> E2eResult<u64> {
    parse_quantity(&jq_input(receipt, ".blockNumber")?)
}

fn parse_quantity(value: &str) -> E2eResult<u64> {
    let value = value.trim_matches('"');
    if let Some(hex) = value.strip_prefix("0x") {
        Ok(u64::from_str_radix(hex, 16)?)
    } else {
        Ok(value.parse()?)
    }
}

fn jq_file(path: &Path, query: &str) -> E2eResult<String> {
    let output = run_checked("jq", ["-er", query, path_str(path)?], true)?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn jq_input(input: &str, query: &str) -> E2eResult<String> {
    let mut child = Command::new("jq")
        .args(["-er", query])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .ok_or("jq stdin was unavailable")?
        .write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    require_status(OsStr::new("jq"), &output, true)?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn keccak(value: &str) -> E2eResult<String> {
    let output = run_checked(cast_bin(), ["keccak", value], true)?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn ibc_path(path_type: u8, sequence: u64) -> String {
    format!("0x636c69656e742d30{path_type:02x}{sequence:016x}")
}

fn packet_tuple(packet: &Packet) -> String {
    format!(
        "({},\"client-0\",\"client-0\",{},[(\"{}\",\"{}\",\"besu-zk-e2e-v1\",\"application/octet-stream\",{})])",
        packet.sequence,
        packet.timeout_timestamp,
        packet.source_port,
        packet.destination_port,
        encode_hex(&packet.payload)
    )
}

fn recv_message(packet: &Packet, proof: &str, height: u64) -> String {
    format!("({},{proof},(0,{height}))", packet_tuple(packet))
}

fn ack_message(packet: &Packet, acknowledgement: &[u8], proof: &str, height: u64) -> String {
    format!(
        "({},{},{proof},(0,{height}))",
        packet_tuple(packet),
        encode_hex(acknowledgement)
    )
}

fn timeout_message(packet: &Packet, proof: &str, height: u64) -> String {
    format!("({},{proof},(0,{height}))", packet_tuple(packet))
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(2 + bytes.len() * 2);
    encoded.push_str("0x");
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn is_zero_word(value: &str) -> bool {
    value.trim() == format!("0x{}", "0".repeat(64))
}

fn mutate_hex_byte(value: &mut String) -> E2eResult<()> {
    if value.len() < 4 || !value.starts_with("0x") {
        return Err("cannot mutate malformed hex artifact".into());
    }
    let replacement = if value.ends_with("00") { "01" } else { "00" };
    value.truncate(value.len() - 2);
    value.push_str(replacement);
    Ok(())
}

fn mutate_enveloped_public_word(envelope: &mut String, word: usize) -> E2eResult<()> {
    // The envelope ABI head is 64 bytes, followed by the public-values length word. The fixed
    // public values begin at byte 96 and every field occupies one 32-byte ABI word.
    let byte_offset = 96_usize
        .checked_add(
            word.checked_mul(32)
                .ok_or("public-value word offset overflow")?,
        )
        .ok_or("public-value byte offset overflow")?;
    let character = 2_usize
        .checked_add(
            byte_offset
                .checked_mul(2)
                .ok_or("public-value hex offset overflow")?,
        )
        .ok_or("public-value character offset overflow")?;
    let current = envelope
        .as_bytes()
        .get(character)
        .copied()
        .ok_or("proof envelope is too short for public-value mutation")?;
    let replacement = if current == b'0' { "1" } else { "0" };
    envelope.replace_range(character..=character, replacement);
    Ok(())
}

fn cast_bin() -> String {
    env::var("CAST_BIN").unwrap_or_else(|_| "cast".into())
}

fn path_str(path: &Path) -> E2eResult<&str> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn run_checked<I, S>(program: impl AsRef<OsStr>, args: I, expect_success: bool) -> E2eResult<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(&program).args(args).output()?;
    require_status(program.as_ref(), &output, expect_success)?;
    Ok(output)
}

fn run_status_checked<I, S>(program: impl AsRef<OsStr>, args: I) -> E2eResult<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(&program).args(args).status()?;
    if status.success() {
        return Ok(());
    }
    Err(format!(
        "{} failed with status {status}",
        program.as_ref().to_string_lossy()
    )
    .into())
}

fn require_status(program: &OsStr, output: &Output, expect_success: bool) -> E2eResult<()> {
    if output.status.success() == expect_success {
        return Ok(());
    }
    Err(format!(
        "{} {} unexpectedly (status {}): {}",
        program.to_string_lossy(),
        if expect_success {
            "failed"
        } else {
            "succeeded"
        },
        output.status,
        String::from_utf8_lossy(&output.stderr)
    )
    .into())
}
