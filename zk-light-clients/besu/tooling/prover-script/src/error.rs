use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("RPC response exceeds configured limit")]
    RpcResponseTooLarge,
    #[error("invalid or unsupported RPC endpoint")]
    InvalidRpcEndpoint,
    #[error("RPC transport failed")]
    RpcTransport,
    #[error("RPC transport timed out")]
    RpcTimeout,
    #[error("RPC endpoint returned HTTP status {0}")]
    RpcHttpStatus(u16),
    #[error("malformed JSON-RPC response")]
    MalformedRpcResponse,
    #[error("JSON-RPC request was rejected with code {0}")]
    RpcRejected(i64),
    #[error("historical state is unavailable from the configured RPC endpoint")]
    HistoricalStateUnavailable,
    #[error("required JSON-RPC field is missing: {0}")]
    MissingField(&'static str),
    #[error("JSON-RPC field has the wrong JSON type: {0}")]
    WrongJsonType(&'static str),
    #[error("malformed hexadecimal value in field: {0}")]
    MalformedHex(&'static str),
    #[error("field {field} has byte length {actual}; expected {expected}")]
    WrongByteLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("non-canonical JSON-RPC quantity in field: {0}")]
    NonCanonicalQuantity(&'static str),
    #[error("numeric value overflows field: {0}")]
    NumericOverflow(&'static str),
    #[error("RPC returned null for required result: {0}")]
    NullResult(&'static str),
    #[error("RPC returned block {actual}; expected {expected}")]
    WrongBlockNumber { expected: u64, actual: u64 },
    #[error("RPC block contains an unsupported field: {0}")]
    UnsupportedBlockField(String),
    #[error("source chain ID mismatch: expected {expected}, got {actual}")]
    ChainIdMismatch { expected: u64, actual: u64 },
    #[error("RPC proof address does not equal configured source Router")]
    WrongProofAddress,
    #[error("account proof must contain at least one node")]
    EmptyAccountProof,
    #[error("RPC returned {0} storage proofs; exactly one is required")]
    WrongStorageProofCount(usize),
    #[error("RPC returned a storage proof for a different key")]
    WrongStorageKey,
    #[error("RPC returned a malformed account/storage proof node")]
    MalformedProofNode,
    #[error("destination light client has no consensus state at height {0}")]
    MissingConsensusState(u64),
    #[error("destination consensus state encoding is malformed")]
    MalformedConsensusState,
    #[error("invalid deterministic witness: {0}")]
    InvalidWitness(String),
    #[error("prepared witness JSON exceeds the {max_bytes}-byte limit")]
    WitnessInputTooLarge { max_bytes: usize },
    #[error("release program identity is invalid: {0}")]
    InvalidProgramIdentity(String),
    #[error("invalid proof artifact: {0}")]
    InvalidArtifact(&'static str),
    #[error("proof artifact JSON serialization failed: {0}")]
    ArtifactSerialization(#[source] serde_json::Error),
    #[error("I/O operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("contract getter {signature} failed: {reason}")]
    ContractGetter {
        signature: &'static str,
        reason: String,
    },
    #[error("deployed verification keys do not match approved programs: {0}")]
    VkeyMismatch(String),
    #[error("SP1 operation failed: {0}")]
    Sp1(String),
    #[error("unsupported SP1 prover backend: {0}")]
    UnsupportedSp1Prover(String),
    #[error("environment variable {0} is not valid Unicode")]
    NonUnicodeEnvironment(&'static str),
    #[error("NETWORK_PRIVATE_KEY is required when SP1_PROVER=network")]
    MissingNetworkPrivateKey,
    #[error("NETWORK_PRIVATE_KEY is malformed")]
    InvalidNetworkPrivateKey,
    #[error("NETWORK_RPC_URL must be a valid HTTP(S) URL")]
    InvalidNetworkRpcUrl,
    #[error("proof mode must be Groth16")]
    UnsupportedProofMode,
    #[error("validator mode must be header-based QBFT")]
    UnsupportedValidatorMode,
}
