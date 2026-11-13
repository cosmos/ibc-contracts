use besu_zk_types::ProofResult;

use crate::MptError;

pub(crate) fn require_present(result: &ProofResult) -> Result<[u8; 32], MptError> {
    match result {
        ProofResult::Present(value) => Ok(*value),
        ProofResult::Absent => Err(MptError::StorageAbsent),
    }
}
