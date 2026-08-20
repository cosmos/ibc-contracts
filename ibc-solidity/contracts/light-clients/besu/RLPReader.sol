// SPDX-License-Identifier: Apache-2.0

/*
 * @author Hamdi Allam hamdi.allam97@gmail.com
 * Please reach out with any questions or concerns
 */
// This file is copied from solidity-rlp library.
// https://github.com/hamdiallam/Solidity-RLP/blob/0212f8e754471da67fc5387df7855f47f944f925/contracts/RLPReader.sol
pragma solidity ^0.8.28;

/* solhint-disable state-visibility, no-inline-assembly, reason-string, gas-custom-errors, gas-increment-by-one,
gas-strict-inequalities */

/// @title RLP Reader
/// @notice Library for reading Recursive Length Prefix encoded values.
library RLPReader {
    /// @notice First byte for short RLP strings.
    uint8 constant STRING_SHORT_START = 0x80;
    /// @notice First byte for long RLP strings.
    uint8 constant STRING_LONG_START = 0xb8;
    /// @notice First byte for short RLP lists.
    uint8 constant LIST_SHORT_START = 0xc0;
    /// @notice First byte for long RLP lists.
    uint8 constant LIST_LONG_START = 0xf8;
    /// @notice EVM word size in bytes.
    uint8 constant WORD_SIZE = 32;

    /// @notice RLP item memory view.
    /// @param len Length of the encoded item.
    /// @param memPtr Memory pointer to the encoded item.
    struct RLPItem {
        uint256 len;
        uint256 memPtr;
    }

    /// @notice Iterator over an RLP list.
    /// @param item List item being iterated.
    /// @param nextPtr Memory pointer to the next item in the list.
    struct Iterator {
        RLPItem item; // Item that's being iterated over.
        uint256 nextPtr; // Position of the next item in the list.
    }

    /// @notice Returns the next item from an iterator.
    /// @param self The iterator.
    /// @return The next item in the iteration.
    function next(Iterator memory self) internal pure returns (RLPItem memory) {
        require(hasNext(self));

        uint256 ptr = self.nextPtr;
        uint256 itemLength = _itemLength(ptr);
        self.nextPtr = ptr + itemLength;

        return RLPItem(itemLength, ptr);
    }

    /// @notice Checks whether an iterator has more items.
    /// @param self The iterator.
    /// @return True if the iterator has another item.
    function hasNext(Iterator memory self) internal pure returns (bool) {
        RLPItem memory item = self.item;
        return self.nextPtr < item.memPtr + item.len;
    }

    /// @notice Wraps RLP-encoded bytes as an RLP item.
    /// @param item RLP-encoded bytes.
    /// @return The RLP item memory view.
    function toRlpItem(bytes memory item) internal pure returns (RLPItem memory) {
        uint256 memPtr;
        assembly {
            memPtr := add(item, 0x20)
        }

        return RLPItem(item.length, memPtr);
    }

    /// @notice Creates an iterator over an RLP list.
    /// @param self The RLP list item.
    /// @return An iterator over the list.
    function iterator(RLPItem memory self) internal pure returns (Iterator memory) {
        require(isList(self));

        uint256 ptr = self.memPtr + _payloadOffset(self.memPtr);
        return Iterator(self, ptr);
    }

    /// @notice Returns the encoded length of an RLP item.
    /// @param item The RLP item.
    /// @return The encoded item length.
    function rlpLen(RLPItem memory item) internal pure returns (uint256) {
        return item.len;
    }

    /// @notice Returns the payload memory location for an RLP item.
    /// @param item The RLP item.
    /// @return The payload memory pointer.
    /// @return The payload length.
    function payloadLocation(RLPItem memory item) internal pure returns (uint256, uint256) {
        uint256 offset = _payloadOffset(item.memPtr);
        uint256 memPtr = item.memPtr + offset;
        uint256 len = item.len - offset; // data length
        return (memPtr, len);
    }

    /// @notice Returns the payload length for an RLP item.
    /// @param item The RLP item.
    /// @return The payload length.
    function payloadLen(RLPItem memory item) internal pure returns (uint256) {
        (, uint256 len) = payloadLocation(item);
        return len;
    }

    /// @notice Decodes an RLP list into item views.
    /// @param item The RLP list item.
    /// @return The list item views.
    function toList(RLPItem memory item) internal pure returns (RLPItem[] memory) {
        require(isList(item));

        uint256 items = numItems(item);
        RLPItem[] memory result = new RLPItem[](items);

        uint256 memPtr = item.memPtr + _payloadOffset(item.memPtr);
        uint256 dataLen;
        for (uint256 i = 0; i < items; i++) {
            dataLen = _itemLength(memPtr);
            result[i] = RLPItem(dataLen, memPtr);
            memPtr = memPtr + dataLen;
        }

        return result;
    }

    /// @notice Checks whether an RLP item is a list.
    /// @param item The RLP item.
    /// @return True if the encoded payload is a list.
    function isList(RLPItem memory item) internal pure returns (bool) {
        if (item.len == 0) return false;

        uint8 byte0;
        uint256 memPtr = item.memPtr;
        assembly {
            byte0 := byte(0, mload(memPtr))
        }

        if (byte0 < LIST_SHORT_START) return false;
        return true;
    }

    /// @notice Computes the keccak256 hash of the full RLP encoding without copying.
    /// @param item The RLP item.
    /// @return The keccak256 hash of the encoded item.
    function rlpBytesKeccak256(RLPItem memory item) internal pure returns (bytes32) {
        uint256 ptr = item.memPtr;
        uint256 len = item.len;
        bytes32 result;
        assembly {
            result := keccak256(ptr, len)
        }
        return result;
    }

    /// @notice Computes the keccak256 hash of an RLP payload without copying.
    /// @param item The RLP item.
    /// @return The keccak256 hash of the item payload.
    function payloadKeccak256(RLPItem memory item) internal pure returns (bytes32) {
        (uint256 memPtr, uint256 len) = payloadLocation(item);
        bytes32 result;
        assembly {
            result := keccak256(memPtr, len)
        }
        return result;
    }

    /*
     * RLPItem conversions into data types.
     */

    /// @notice Copies the full RLP encoding into a bytes array.
    /// @param item The RLP item.
    /// @return The RLP-encoded bytes.
    function toRlpBytes(RLPItem memory item) internal pure returns (bytes memory) {
        bytes memory result = new bytes(item.len);
        if (result.length == 0) return result;

        uint256 ptr;
        assembly {
            ptr := add(0x20, result)
        }

        copy(item.memPtr, ptr, item.len);
        return result;
    }

    /// @notice Converts an RLP item to a boolean.
    /// @param item The RLP item.
    /// @return The decoded boolean value.
    function toBoolean(RLPItem memory item) internal pure returns (bool) {
        require(item.len == 1);
        uint256 result;
        uint256 memPtr = item.memPtr;
        assembly {
            result := byte(0, mload(memPtr))
        }

        // SEE Github Issue #5.
        // Summary: Most commonly used RLP libraries (i.e Geth) will encode
        // "0" as "0x80" instead of as "0". We handle this edge case explicitly
        // here.
        if (result == 0 || result == STRING_SHORT_START) {
            return false;
        } else {
            return true;
        }
    }

    /// @notice Converts an RLP item to an address.
    /// @param item The RLP item.
    /// @return The decoded address.
    function toAddress(RLPItem memory item) internal pure returns (address) {
        // 1 byte for the length prefix
        require(item.len == 21);

        return address(uint160(toUint(item)));
    }

    /// @notice Converts an RLP item to an unsigned integer.
    /// @param item The RLP item.
    /// @return The decoded unsigned integer.
    function toUint(RLPItem memory item) internal pure returns (uint256) {
        require(item.len > 0 && item.len <= 33);

        (uint256 memPtr, uint256 len) = payloadLocation(item);

        uint256 result;
        assembly {
            result := mload(memPtr)

            // shift to the correct location if neccesary
            if lt(len, 32) { result := div(result, exp(256, sub(32, len))) }
        }

        return result;
    }

    /// @notice Converts an RLP item to an unsigned integer and requires a 32-byte payload.
    /// @param item The RLP item.
    /// @return The decoded unsigned integer.
    function toUintStrict(RLPItem memory item) internal pure returns (uint256) {
        // one byte prefix
        require(item.len == 33);

        uint256 result;
        uint256 memPtr = item.memPtr + 1;
        assembly {
            result := mload(memPtr)
        }

        return result;
    }

    /// @notice Copies an RLP item payload into a bytes array.
    /// @param item The RLP item.
    /// @return The decoded payload bytes.
    function toBytes(RLPItem memory item) internal pure returns (bytes memory) {
        require(item.len > 0);

        (uint256 memPtr, uint256 len) = payloadLocation(item);
        bytes memory result = new bytes(len);

        uint256 destPtr;
        assembly {
            destPtr := add(0x20, result)
        }

        copy(memPtr, destPtr, len);
        return result;
    }

    /*
     * Private Helpers
     */

    /// @notice Counts the payload items inside an encoded list.
    /// @param item The RLP list item.
    /// @return The number of items in the list.
    function numItems(RLPItem memory item) private pure returns (uint256) {
        if (item.len == 0) return 0;

        uint256 count = 0;
        uint256 currPtr = item.memPtr + _payloadOffset(item.memPtr);
        uint256 endPtr = item.memPtr + item.len;
        while (currPtr < endPtr) {
            currPtr = currPtr + _itemLength(currPtr); // skip over an item
            count++;
        }

        return count;
    }

    /// @notice Computes the full encoded byte length of an RLP item at a memory pointer.
    /// @param memPtr Memory pointer to the item.
    /// @return The encoded item length.
    function _itemLength(uint256 memPtr) private pure returns (uint256) {
        uint256 itemLen;
        uint256 byte0;
        assembly {
            byte0 := byte(0, mload(memPtr))
        }

        if (byte0 < STRING_SHORT_START) {
            itemLen = 1;
        } else if (byte0 < STRING_LONG_START) {
            itemLen = byte0 - STRING_SHORT_START + 1;
        } else if (byte0 < LIST_SHORT_START) {
            assembly {
                let byteLen := sub(byte0, 0xb7) // # of bytes the actual length is
                memPtr := add(memPtr, 1) // skip over the first byte

                /* 32 byte word size */
                let dataLen := div(mload(memPtr), exp(256, sub(32, byteLen))) // right shifting to get the len
                itemLen := add(dataLen, add(byteLen, 1))
            }
        } else if (byte0 < LIST_LONG_START) {
            itemLen = byte0 - LIST_SHORT_START + 1;
        } else {
            assembly {
                let byteLen := sub(byte0, 0xf7)
                memPtr := add(memPtr, 1)

                let dataLen := div(mload(memPtr), exp(256, sub(32, byteLen))) // right shifting to the correct length
                itemLen := add(dataLen, add(byteLen, 1))
            }
        }

        return itemLen;
    }

    /// @notice Computes the offset from an RLP item prefix to its payload.
    /// @param memPtr Memory pointer to the item.
    /// @return The payload offset in bytes.
    function _payloadOffset(uint256 memPtr) private pure returns (uint256) {
        uint256 byte0;
        assembly {
            byte0 := byte(0, mload(memPtr))
        }

        if (byte0 < STRING_SHORT_START) {
            return 0;
        } else if (byte0 < STRING_LONG_START || (byte0 >= LIST_SHORT_START && byte0 < LIST_LONG_START)) {
            return 1;
        } else if (byte0 < LIST_SHORT_START) {
            // being explicit
            return byte0 - (STRING_LONG_START - 1) + 1;
        } else {
            return byte0 - (LIST_LONG_START - 1) + 1;
        }
    }

    /// @notice Copies memory from one pointer to another.
    /// @param src Pointer to the source memory.
    /// @param dest Pointer to the destination memory.
    /// @param len Number of bytes to copy.
    function copy(uint256 src, uint256 dest, uint256 len) private pure {
        if (len == 0) return;

        // copy as many word sizes as possible
        for (; len >= WORD_SIZE; len -= WORD_SIZE) {
            assembly {
                mstore(dest, mload(src))
            }

            src += WORD_SIZE;
            dest += WORD_SIZE;
        }

        if (len > 0) {
            // left over bytes. Mask is used to remove unwanted bytes from the word
            uint256 mask = 256 ** (WORD_SIZE - len) - 1;
            assembly {
                let srcpart := and(mload(src), not(mask)) // zero out src
                let destpart := and(mload(dest), mask) // retrieve the bytes
                mstore(dest, or(destpart, srcpart))
            }
        }
    }
}
