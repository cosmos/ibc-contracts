// SPDX-License-Identifier: Apache-2.0

//! Solidity types for IBCMulticall.sol

use alloy_primitives::{Address, Bytes};
use alloy_sol_types::SolCall;

alloy_sol_types::sol!(
    #[derive(Debug, PartialEq, Eq)]
    ibc_multicall,
    "../../ibc-solidity/abi/IBCMulticall.json"
);

/// Encodes an `IBCMulticall.multicall` call that calls `target` with each of `calls` in order.
///
/// The first `strict_calls` calls must succeed or the whole batch reverts. The remaining calls are
/// allowed to fail, in which case only their own effects are rolled back.
#[must_use]
pub fn multicall_calldata(
    target: Address,
    calls: impl IntoIterator<Item = Bytes>,
    strict_calls: usize,
) -> Vec<u8> {
    ibc_multicall::multicallCall {
        calls: calls
            .into_iter()
            .enumerate()
            .map(|(i, call_data)| IIBCMulticallMsgs::Call {
                target,
                allowFailure: i >= strict_calls,
                callData: call_data,
            })
            .collect(),
    }
    .abi_encode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multicall_calldata_marks_only_strict_calls() {
        let target = Address::repeat_byte(0x26);
        let calls = [
            Bytes::from_static(b"a"),
            Bytes::from_static(b"b"),
            Bytes::from_static(b"c"),
        ];

        let calldata = multicall_calldata(target, calls.clone(), 1);
        let decoded = ibc_multicall::multicallCall::abi_decode(&calldata).unwrap();

        assert_eq!(decoded.calls.len(), calls.len());
        for (i, (call, data)) in decoded.calls.iter().zip(calls).enumerate() {
            assert_eq!(call.target, target);
            assert_eq!(call.callData, data);
            assert_eq!(call.allowFailure, i >= 1);
        }
    }
}
