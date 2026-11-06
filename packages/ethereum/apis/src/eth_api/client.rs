// SPDX-License-Identifier: Apache-2.0

//! This module implements the `EthApiClient` to interact with the Ethereum RPC API.

use std::str::FromStr;

use alloy::{
    primitives::{keccak256, Address, Bytes, StorageKey, B256},
    providers::Provider,
    rlp,
    rpc::types::{Block, EIP1186AccountProofResponse},
};

use super::error::EthClientError;

const RPC_METHOD_GET_PROOF: &str = "eth_getProof";

/// The api client for interacting with the Beacon API
#[allow(clippy::module_name_repetitions)]
pub struct EthApiClient<P: Provider + Clone> {
    provider: P,
}

impl<P: Provider + Clone> EthApiClient<P> {
    /// Create new `EthApiClient`
    pub const fn new(provider: P) -> Self {
        Self { provider }
    }

    /// Fetches proof for an account and optionally storage keys under the given account at the block.
    /// # Errors
    /// Returns an error if the input fails to serialize, the request fails or the response is not successful deserialized
    pub async fn get_proof(
        &self,
        address: &str,
        storage_keys: Vec<String>,
        block_hex: String,
    ) -> Result<EIP1186AccountProofResponse, EthClientError> {
        let address: Address = Address::from_str(address)
            .map_err(|e| EthClientError::ParseError(address.to_string(), e.to_string()))?;
        let storage_keys: Vec<StorageKey> = storage_keys
            .into_iter()
            .map(|key| {
                StorageKey::from_str(&key)
                    .map_err(|e| EthClientError::ParseError(key, e.to_string()))
            })
            .collect::<Result<_, _>>()?;
        Ok(self
            .provider
            .client()
            .request(RPC_METHOD_GET_PROOF, (address, storage_keys, block_hex))
            .await?)
    }

    /// Fetches the current block number.
    /// # Errors
    /// Returns an error if the request fails
    pub async fn get_block_number(&self) -> Result<u64, EthClientError> {
        Ok(self.provider.get_block_number().await?)
    }

    /// Fetches a block
    /// # Errors
    /// Returns an error if the request fails
    pub async fn get_block(&self, block_number: u64) -> Result<Block, EthClientError> {
        self.provider
            .get_block(block_number.into())
            .await?
            .ok_or_else(|| EthClientError::BlockNotFound(block_number))
    }

    /// Fetches a block by its hash
    /// # Errors
    /// Returns an error if the request fails or the block is not found
    pub async fn get_block_by_hash(&self, block_hash: B256) -> Result<Block, EthClientError> {
        self.provider
            .get_block_by_hash(block_hash)
            .await?
            .ok_or(EthClientError::BlockHashNotFound(block_hash))
    }

    /// Fetches the RLP encoded header of a block by its hash
    /// # Errors
    /// Returns an error if the request fails, the block is not found or the header cannot be
    /// reproduced from the response (i.e. a new fork added header fields we can't encode yet)
    pub async fn get_block_header_rlp(&self, block_hash: B256) -> Result<Bytes, EthClientError> {
        encode_block_header(&self.get_block_by_hash(block_hash).await?, block_hash)
    }
}

/// RLP encodes the header of a block and checks it against the expected block hash.
fn encode_block_header(block: &Block, expected: B256) -> Result<Bytes, EthClientError> {
    let header_rlp = rlp::encode(&block.header.inner);
    let found = keccak256(&header_rlp);
    if found != expected {
        return Err(EthClientError::BlockHeaderHashMismatch { expected, found });
    }

    Ok(header_rlp.into())
}

#[cfg(test)]
mod test {
    use alloy::{hex, primitives::b256};

    use super::*;

    // An Amsterdam block as returned by `eth_getBlockByHash`, and its header from `debug_getRawHeader`.
    const BLOCK_JSON: &str = r#"{"baseFeePerGas": "0x12a18", "blobGasUsed": "0x0", "blockAccessListHash": "0x72b02fe4071f2eeed18399668a0598758416b8d551a71b7dd81aa640309fb8f8", "difficulty": "0x0", "excessBlobGas": "0x0", "extraData": "0xd88301110a846765746888676f312e32372e33856c696e7578", "gasLimit": "0xbebc200", "gasUsed": "0x0", "hash": "0x1d00d753e3964751cefe1cae2d81ad8af9b59a4868df2db5ea243346e67a6038", "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "miner": "0x8943545177806ed17b9f23f0a21ee5948ecaa776", "mixHash": "0x8aa640c661e4971471d6485e6ba1981b44ae4e4cdb05a1c60857224e92cb42f5", "nonce": "0x0000000000000000", "number": "0x47", "parentBeaconBlockRoot": "0xce756ce287e17db000f9527f2744cd689676f67bfd09b11a1a76c3493d9ef9a1", "parentHash": "0x7a802472dcd4464726be3bddc3a64f36f6410fa01b733823e3b17a874a09d9d7", "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421", "requestsHash": "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347", "size": "0x2a2", "slotNumber": "0x47", "stateRoot": "0x6d4cdea27ed68f0d4cbd443db0efe584a253fccf445e280040c81ba6502cf977", "timestamp": "0x6aec45c6", "transactions": [], "transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421", "uncles": [], "withdrawals": [], "withdrawalsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"}"#;
    const HEADER_RLP: &str = "f90299a07a802472dcd4464726be3bddc3a64f36f6410fa01b733823e3b17a874a09d9d7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948943545177806ed17b9f23f0a21ee5948ecaa776a06d4cdea27ed68f0d4cbd443db0efe584a253fccf445e280040c81ba6502cf977a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008047840bebc20080846aec45c699d88301110a846765746888676f312e32372e33856c696e7578a08aa640c661e4971471d6485e6ba1981b44ae4e4cdb05a1c60857224e92cb42f588000000000000000083012a18a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4218080a0ce756ce287e17db000f9527f2744cd689676f67bfd09b11a1a76c3493d9ef9a1a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855a072b02fe4071f2eeed18399668a0598758416b8d551a71b7dd81aa640309fb8f847";

    #[test]
    fn test_encode_block_header() {
        let block: Block = serde_json::from_str(BLOCK_JSON).unwrap();

        let header_rlp = encode_block_header(&block, block.header.hash).unwrap();

        assert_eq!(hex::encode(header_rlp), HEADER_RLP);
    }

    #[test]
    fn test_encode_block_header_rejects_unexpected_hash() {
        let block: Block = serde_json::from_str(BLOCK_JSON).unwrap();

        let err = encode_block_header(
            &block,
            b256!("0000000000000000000000000000000000000000000000000000000000000001"),
        )
        .unwrap_err();

        assert!(matches!(
            err,
            EthClientError::BlockHeaderHashMismatch { .. }
        ));
    }
}
