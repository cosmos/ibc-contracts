// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing,function-max-lines,gas-small-strings

import { Test } from "forge-std/Test.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";

import { TrieProof } from "../../contracts/utils/TrieProof.sol";

struct TrieMembershipTestCase {
    string name;
    bytes32 root;
    bytes key;
    bytes value;
    bytes[] proof;
}

struct TrieExclusionTestCase {
    string name;
    bytes32 root;
    bytes key;
    bytes[] proof;
}

struct TrieInvalidTestCase {
    string name;
    bytes32 root;
    bytes key;
    bytes[] proof;
    TrieProof.ProofError expectedError;
}

struct TrieMalformedTestCase {
    string name;
    bytes32 root;
    bytes key;
    bytes[] proof;
}

contract TrieProofHarness {
    function verify(
        bytes calldata value,
        bytes32 root,
        bytes calldata key,
        bytes[] calldata proof
    )
        external
        pure
        returns (bool)
    {
        return TrieProof.verify(value, root, key, proof);
    }

    function verifyExclusion(bytes32 root, bytes calldata key, bytes[] calldata proof) external pure returns (bool) {
        return TrieProof.verifyExclusion(root, key, proof);
    }

    function traverse(bytes32 root, bytes calldata key, bytes[] calldata proof) external pure returns (bytes memory) {
        return TrieProof.traverse(root, key, proof);
    }

    function tryTraverse(
        bytes32 root,
        bytes calldata key,
        bytes[] calldata proof
    )
        external
        pure
        returns (bytes memory, TrieProof.ProofError)
    {
        return TrieProof.tryTraverse(root, key, proof);
    }
}

contract TrieProofTest is Test {
    bytes32 internal constant EMPTY_ROOT_HASH = 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;

    TrieProofHarness internal harness;

    function setUp() public {
        harness = new TrieProofHarness();
    }

    function tableMembershipTest(TrieMembershipTestCase memory membership) public view {
        assertTrue(harness.verify(membership.value, membership.root, membership.key, membership.proof));
        assertFalse(
            harness.verify(bytes.concat(membership.value, hex"00"), membership.root, membership.key, membership.proof)
        );
        assertFalse(harness.verifyExclusion(membership.root, membership.key, membership.proof));
        assertEq(harness.traverse(membership.root, membership.key, membership.proof), membership.value);

        (bytes memory value, TrieProof.ProofError err) =
            harness.tryTraverse(membership.root, membership.key, membership.proof);
        assertEq(value, membership.value);
        assertEq(uint256(err), uint256(TrieProof.ProofError.NO_ERROR));
    }

    function tableExclusionTest(TrieExclusionTestCase memory exclusion) public {
        assertFalse(harness.verify("", exclusion.root, exclusion.key, exclusion.proof));
        assertTrue(harness.verifyExclusion(exclusion.root, exclusion.key, exclusion.proof));

        vm.expectRevert(
            abi.encodeWithSelector(
                TrieProof.TrieProofTraversalError.selector, TrieProof.ProofError.VALID_EXCLUSION_PROOF
            )
        );
        harness.traverse(exclusion.root, exclusion.key, exclusion.proof);

        (bytes memory value, TrieProof.ProofError err) =
            harness.tryTraverse(exclusion.root, exclusion.key, exclusion.proof);
        assertEq(value, "");
        assertEq(uint256(err), uint256(TrieProof.ProofError.VALID_EXCLUSION_PROOF));
    }

    function tableInvalidTest(TrieInvalidTestCase memory invalid) public {
        assertFalse(harness.verify("", invalid.root, invalid.key, invalid.proof));
        assertFalse(harness.verifyExclusion(invalid.root, invalid.key, invalid.proof));

        vm.expectRevert(abi.encodeWithSelector(TrieProof.TrieProofTraversalError.selector, invalid.expectedError));
        harness.traverse(invalid.root, invalid.key, invalid.proof);

        (bytes memory value, TrieProof.ProofError err) = harness.tryTraverse(invalid.root, invalid.key, invalid.proof);
        assertEq(value, "");
        assertEq(uint256(err), uint256(invalid.expectedError));
    }

    function tableMalformedTest(TrieMalformedTestCase memory malformed) public {
        vm.expectRevert(RLP.RLPInvalidEncoding.selector);
        harness.verify("", malformed.root, malformed.key, malformed.proof);

        vm.expectRevert(RLP.RLPInvalidEncoding.selector);
        harness.verifyExclusion(malformed.root, malformed.key, malformed.proof);

        vm.expectRevert(RLP.RLPInvalidEncoding.selector);
        harness.traverse(malformed.root, malformed.key, malformed.proof);

        vm.expectRevert(RLP.RLPInvalidEncoding.selector);
        harness.tryTraverse(malformed.root, malformed.key, malformed.proof);
    }

    /// @dev Ports the static valid-proof vectors used by OpenZeppelin's TrieProof TypeScript suite.
    function fixtureMembership() public pure returns (TrieMembershipTestCase[] memory testCases) {
        testCases = new TrieMembershipTestCase[](18);

        testCases[0] = _membership(
            "OpenZeppelin valid proof 1",
            0xd582f99275e227a1cf4284899e5ff06ee56da8859be71b553397c69151bc942f,
            hex"6b6579326262",
            hex"6176616c32",
            _proof3(
                hex"e68416b65793a03101b4447781f1e6c51ce76c709274fc80bd064f3a58ff981b6015348a826386",
                hex"f84580a0582eed8dd051b823d13f8648cdcd08aa2d8dac239f458863c4620e8c4d605debca83206262856176616c32ca83206363856176616c3380808080808080808080808080",
                hex"ca83206262856176616c32"
            )
        );
        testCases[1] = _membership(
            "OpenZeppelin valid proof 2",
            0xd582f99275e227a1cf4284899e5ff06ee56da8859be71b553397c69151bc942f,
            hex"6b6579316161",
            hex"303132333435363738393031323334353637383930313233343536373839303132333435363738397878",
            _proof3(
                hex"e68416b65793a03101b4447781f1e6c51ce76c709274fc80bd064f3a58ff981b6015348a826386",
                hex"f84580a0582eed8dd051b823d13f8648cdcd08aa2d8dac239f458863c4620e8c4d605debca83206262856176616c32ca83206363856176616c3380808080808080808080808080",
                hex"ef83206161aa303132333435363738393031323334353637383930313233343536373839303132333435363738397878"
            )
        );
        testCases[2] = _membership(
            "OpenZeppelin valid proof 3",
            0xf838216fa749aefa91e0b672a9c06d3e6e983f913d7107b5dab4af60b5f5abed,
            hex"6b6579316161",
            hex"303132333435363738393031323334353637383930313233343536373839303132333435363738397878",
            _proof1(
                hex"f387206b6579316161aa303132333435363738393031323334353637383930313233343536373839303132333435363738397878"
            )
        );
        testCases[3] = _membership(
            "OpenZeppelin valid proof 4",
            0x37956bab6bba472308146808d5311ac19cb4a7daae5df7efcc0f32badc97f55e,
            hex"6b6579316161",
            hex"3031323334",
            _proof1(hex"ce87206b6579316161853031323334")
        );
        testCases[4] = _membership(
            "OpenZeppelin valid proof 5",
            0xcb65032e2f76c48b82b5c24b3db8f670ce73982869d38cd39a624f23d62a9e89,
            hex"6b657931",
            hex"30313233343536373839303132333435363738393031323334353637383930313233343536373839566572795f4c6f6e67",
            _proof3(
                hex"e68416b65793a0f3f387240403976788281c0a6ee5b3fc08360d276039d635bb824ea7e6fed779",
                hex"f87180a034d14ccc7685aa2beb64f78b11ee2a335eae82047ef97c79b7dda7f0732b9f4ca05fb052b64e23d177131d9f32e9c5b942209eb7229e9a07c99a5d93245f53af18a09a137197a43a880648d5887cce656a5e6bbbe5e44ecb4f264395ccaddbe1acca80808080808080808080808080",
                hex"f862808080808080a057895fdbd71e2c67c2f9274a56811ff5cf458720a7fa713a135e3890f8cafcf8808080808080808080b130313233343536373839303132333435363738393031323334353637383930313233343536373839566572795f4c6f6e67"
            )
        );
        testCases[5] = _membership(
            "OpenZeppelin valid proof 6",
            0xcb65032e2f76c48b82b5c24b3db8f670ce73982869d38cd39a624f23d62a9e89,
            hex"6b657932",
            hex"73686f7274",
            _proof3(
                hex"e68416b65793a0f3f387240403976788281c0a6ee5b3fc08360d276039d635bb824ea7e6fed779",
                hex"f87180a034d14ccc7685aa2beb64f78b11ee2a335eae82047ef97c79b7dda7f0732b9f4ca05fb052b64e23d177131d9f32e9c5b942209eb7229e9a07c99a5d93245f53af18a09a137197a43a880648d5887cce656a5e6bbbe5e44ecb4f264395ccaddbe1acca80808080808080808080808080",
                hex"df808080808080c9823262856176616c338080808080808080808573686f7274"
            )
        );
        testCases[6] = _membership(
            "OpenZeppelin valid proof 7",
            0xcb65032e2f76c48b82b5c24b3db8f670ce73982869d38cd39a624f23d62a9e89,
            hex"6b657933",
            hex"31323334353637383930313233343536373839303132333435363738393031",
            _proof3(
                hex"e68416b65793a0f3f387240403976788281c0a6ee5b3fc08360d276039d635bb824ea7e6fed779",
                hex"f87180a034d14ccc7685aa2beb64f78b11ee2a335eae82047ef97c79b7dda7f0732b9f4ca05fb052b64e23d177131d9f32e9c5b942209eb7229e9a07c99a5d93245f53af18a09a137197a43a880648d5887cce656a5e6bbbe5e44ecb4f264395ccaddbe1acca80808080808080808080808080",
                hex"f839808080808080c9823363856176616c338080808080808080809f31323334353637383930313233343536373839303132333435363738393031"
            )
        );
        testCases[7] = _membership(
            "OpenZeppelin valid proof 8",
            0x72e6c01ad0c9a7b517d4bc68a5b323287fe80f0e68f5415b4b95ecbc8ad83978,
            hex"61",
            hex"61",
            _proof3(
                hex"d916d780c22061c22062c2206380808080808080808080808080",
                hex"d780c22061c22062c2206380808080808080808080808080",
                hex"c22061"
            )
        );
        testCases[8] = _membership(
            "OpenZeppelin valid proof 9",
            0x72e6c01ad0c9a7b517d4bc68a5b323287fe80f0e68f5415b4b95ecbc8ad83978,
            hex"62",
            hex"62",
            _proof3(
                hex"d916d780c22061c22062c2206380808080808080808080808080",
                hex"d780c22061c22062c2206380808080808080808080808080",
                hex"c22062"
            )
        );
        testCases[9] = _membership(
            "OpenZeppelin valid proof 10",
            0x72e6c01ad0c9a7b517d4bc68a5b323287fe80f0e68f5415b4b95ecbc8ad83978,
            hex"63",
            hex"63",
            _proof3(
                hex"d916d780c22061c22062c2206380808080808080808080808080",
                hex"d780c22061c22062c2206380808080808080808080808080",
                hex"c22063"
            )
        );

        _setGeneratedLeafAndBranchMembershipCases(testCases);
        _setGeneratedExtensionMembershipCases(testCases);
    }

    function _setGeneratedLeafAndBranchMembershipCases(TrieMembershipTestCase[] memory testCases) private pure {
        bytes memory evenLeaf = _leaf(hex"20abcd", hex"2a");
        bytes memory smallLeaf = _leaf(hex"30", hex"2a");
        bytes memory largeLeaf = _leaf(hex"30", _longValue());
        bytes memory inlineBranch = _branch(10, smallLeaf, false, "");
        bytes memory hashedBranch = _branch(10, largeLeaf, true, "");
        testCases[10] = _membership("generated even leaf", keccak256(evenLeaf), hex"abcd", hex"2a", _proof1(evenLeaf));
        testCases[11] = _membership(
            "generated hashed branch child",
            keccak256(hashedBranch),
            hex"a0",
            _longValue(),
            _proof2(hashedBranch, largeLeaf)
        );
        testCases[12] = _membership(
            "generated inline branch full proof",
            keccak256(inlineBranch),
            hex"a0",
            hex"2a",
            _proof2(inlineBranch, smallLeaf)
        );
        testCases[13] = _membership(
            "generated inline branch compressed proof", keccak256(inlineBranch), hex"a0", hex"2a", _proof1(inlineBranch)
        );
    }

    function _setGeneratedExtensionMembershipCases(TrieMembershipTestCase[] memory testCases) private pure {
        bytes memory oddSmallLeaf = _leaf(hex"3bcd", hex"2a");
        bytes memory oddLargeLeaf = _leaf(hex"3bcd", _longValue());
        bytes memory inlineExtension = _extension(hex"1a", oddSmallLeaf, false);
        bytes memory hashedExtension = _extension(hex"1a", oddLargeLeaf, true);
        bytes memory terminalBranch = _branch(0, "", false, hex"2a");
        bytes memory terminalExtension = _extension(hex"00ab", terminalBranch, false);
        testCases[14] = _membership(
            "generated hashed extension child",
            keccak256(hashedExtension),
            hex"abcd",
            _longValue(),
            _proof2(hashedExtension, oddLargeLeaf)
        );
        testCases[15] = _membership(
            "generated inline extension full proof",
            keccak256(inlineExtension),
            hex"abcd",
            hex"2a",
            _proof2(inlineExtension, oddSmallLeaf)
        );
        testCases[16] = _membership(
            "generated inline extension compressed proof",
            keccak256(inlineExtension),
            hex"abcd",
            hex"2a",
            _proof1(inlineExtension)
        );
        testCases[17] = _membership(
            "generated branch terminal value",
            keccak256(terminalExtension),
            hex"ab",
            hex"2a",
            _proof1(terminalExtension)
        );
    }

    function fixtureExclusion() public pure returns (TrieExclusionTestCase[] memory testCases) {
        testCases = new TrieExclusionTestCase[](12);

        bytes memory divergentLeaf = _leaf(hex"20abce", hex"2a");
        bytes memory shortLeaf = _leaf(hex"3abc", hex"2a");
        bytes memory longLeaf = _leaf(hex"3abcde", hex"2a");
        bytes memory extensionMismatch = _extension(hex"00ab", _leaf(hex"20cd", hex"2a"), false);
        bytes memory populatedBranch = _branch(11, _leaf(hex"30", hex"2b"), false, "");
        bytes memory terminalBranch = _branch(12, _leaf(hex"30", hex"2c"), false, "");
        bytes memory terminalExtension = _extension(hex"00ab", terminalBranch, false);

        testCases[0] = _exclusion("empty trie", EMPTY_ROOT_HASH, hex"01", _proof0());
        testCases[1] = _exclusion("divergent leaf", keccak256(divergentLeaf), hex"abcd", _proof1(divergentLeaf));
        testCases[2] = _exclusion("leaf path is key prefix", keccak256(shortLeaf), hex"abcd", _proof1(shortLeaf));
        testCases[3] = _exclusion("leaf path longer than key", keccak256(longLeaf), hex"abcd", _proof1(longLeaf));
        testCases[4] =
            _exclusion("divergent extension", keccak256(extensionMismatch), hex"ac00", _proof1(extensionMismatch));
        testCases[5] = _exclusion("missing branch child", keccak256(populatedBranch), hex"a0", _proof1(populatedBranch));
        testCases[6] = _exclusion(
            "empty branch terminal value", keccak256(terminalExtension), hex"ab", _proof1(terminalExtension)
        );

        testCases[7] = _exclusion(
            "OpenZeppelin nonexistent key 1",
            0xd582f99275e227a1cf4284899e5ff06ee56da8859be71b553397c69151bc942f,
            hex"6b657932",
            _proof3(
                hex"e68416b65793a03101b4447781f1e6c51ce76c709274fc80bd064f3a58ff981b6015348a826386",
                hex"f84580a0582eed8dd051b823d13f8648cdcd08aa2d8dac239f458863c4620e8c4d605debca83206262856176616c32ca83206363856176616c3380808080808080808080808080",
                hex"ca83206262856176616c32"
            )
        );
        testCases[8] = _exclusion(
            "OpenZeppelin nonexistent key 2",
            0xd582f99275e227a1cf4284899e5ff06ee56da8859be71b553397c69151bc942f,
            hex"616e7972616e646f6d6b6579",
            _proof1(hex"e68416b65793a03101b4447781f1e6c51ce76c709274fc80bd064f3a58ff981b6015348a826386")
        );
        testCases[9] = _exclusion(
            "OpenZeppelin zero branch value",
            0xe04b3589eef96b237cd49ccb5dcf6e654a47682bfa0961d563ab843f7ad1e035,
            hex"aa",
            _proof2(
                hex"dd8200aad98080808080808080808080c43b82aabbc43c82aacc80808080",
                hex"d98080808080808080808080c43b82aabbc43c82aacc80808080"
            )
        );
        testCases[10] = _exclusion(
            "OpenZeppelin smaller path 1",
            0xa513ba530659356fb7588a2c831944e80fd8aedaa5a4dc36f918152be2be0605,
            hex"01",
            _proof3(
                hex"db10d9c32081bbc582202381aa808080808080808080808080808080",
                hex"d9c32081bbc582202381aa808080808080808080808080808080",
                hex"c582202381aa"
            )
        );
        testCases[11] = _exclusion(
            "OpenZeppelin smaller path 2",
            0xa06abffaec4ebe8ccde595f4547b864b4421b21c1fc699973f94710c9bc17979,
            hex"aa",
            _proof3(
                hex"e21aa07ea462226a3dc0a46afb4ded39306d7a84d311ada3557dfc75a909fd25530905",
                hex"f380808080808080808080a027f11bd3af96d137b9287632f44dd00fea1ca1bd70386c30985ede8cc287476e808080c220338080",
                hex"e48200bba0a6911545ed01c2d3f4e15b8b27c7bfba97738bd5e6dd674dd07033428a4c53af"
            )
        );
    }

    function fixtureInvalid() public pure returns (TrieInvalidTestCase[] memory testCases) {
        testCases = new TrieInvalidTestCase[](18);

        _setBasicInvalidCases(testCases);
        _setInvalidExtraProofCases(testCases);
        _setInvalidChildCases(testCases);
        _setOpenZeppelinInvalidCases(testCases);
    }

    function _setBasicInvalidCases(TrieInvalidTestCase[] memory testCases) private pure {
        bytes memory dummy = RLP.encode(new bytes[](0));
        bytes memory emptyPath = _leaf("", hex"2a");
        bytes memory emptyExtensionPath = _extension(hex"00", _leaf(hex"2000", hex"2a"), false);
        bytes memory unknownPrefix = _leaf(hex"40", hex"2a");
        bytes memory unparsable = _list3(hex"00", hex"00", hex"00");
        bytes memory emptyValue = _leaf(hex"20abcd", "");

        testCases[0] = _invalid("empty key", bytes32(0), "", _proof0(), TrieProof.ProofError.EMPTY_KEY);
        testCases[1] = _invalid(
            "proof after empty trie",
            EMPTY_ROOT_HASH,
            hex"01",
            _proof1(dummy),
            TrieProof.ProofError.INVALID_EXTRA_PROOF_ELEMENT
        );
        testCases[2] = _invalid(
            "wrong root", bytes32(0), hex"abcd", _proof1(_leaf(hex"20abcd", hex"2a")), TrieProof.ProofError.INVALID_ROOT
        );
        testCases[3] = _invalid("proof exhausted", bytes32(0), hex"abcd", _proof0(), TrieProof.ProofError.INVALID_PROOF);
        testCases[4] = _invalid(
            "empty compact path", keccak256(emptyPath), hex"00", _proof1(emptyPath), TrieProof.ProofError.EMPTY_PATH
        );
        testCases[5] = _invalid(
            "empty extension path",
            keccak256(emptyExtensionPath),
            hex"00",
            _proof1(emptyExtensionPath),
            TrieProof.ProofError.EMPTY_EXTENSION_PATH_REMAINDER
        );
        testCases[6] = _invalid(
            "unknown compact prefix",
            keccak256(unknownPrefix),
            hex"00",
            _proof1(unknownPrefix),
            TrieProof.ProofError.UNKNOWN_NODE_PREFIX
        );
        testCases[7] = _invalid(
            "unparseable node",
            keccak256(unparsable),
            hex"00",
            _proof1(unparsable),
            TrieProof.ProofError.UNPARSEABLE_NODE
        );
        testCases[8] = _invalid(
            "empty leaf value", keccak256(emptyValue), hex"abcd", _proof1(emptyValue), TrieProof.ProofError.EMPTY_VALUE
        );
    }

    function _setInvalidExtraProofCases(TrieInvalidTestCase[] memory testCases) private pure {
        bytes memory validLeaf = _leaf(hex"20abcd", hex"2a");
        bytes memory divergentLeaf = _leaf(hex"20abce", hex"2a");
        bytes memory populatedBranch = _branch(11, _leaf(hex"30", hex"2b"), false, "");
        bytes memory terminalBranch = _branch(12, _leaf(hex"30", hex"2c"), false, hex"2a");
        bytes memory terminalExtension = _extension(hex"00ab", terminalBranch, false);
        bytes memory dummy = RLP.encode(new bytes[](0));

        testCases[9] = _invalid(
            "proof after leaf",
            keccak256(validLeaf),
            hex"abcd",
            _proof2(validLeaf, dummy),
            TrieProof.ProofError.INVALID_EXTRA_PROOF_ELEMENT
        );
        testCases[10] = _invalid(
            "proof after divergent leaf",
            keccak256(divergentLeaf),
            hex"abcd",
            _proof2(divergentLeaf, dummy),
            TrieProof.ProofError.INVALID_EXTRA_PROOF_ELEMENT
        );
        testCases[11] = _invalid(
            "proof after missing branch child",
            keccak256(populatedBranch),
            hex"a0",
            _proof2(populatedBranch, dummy),
            TrieProof.ProofError.INVALID_EXTRA_PROOF_ELEMENT
        );
        testCases[12] = _invalid(
            "proof after branch terminal",
            keccak256(terminalExtension),
            hex"ab",
            _proof2(terminalExtension, dummy),
            TrieProof.ProofError.INVALID_EXTRA_PROOF_ELEMENT
        );
    }

    function _setInvalidChildCases(TrieInvalidTestCase[] memory testCases) private pure {
        bytes memory largeLeaf = _leaf(hex"30", _longValue());
        bytes memory wrongLargeLeaf = _leaf(hex"30", bytes.concat(_longValue(), hex"00"));
        bytes memory hashedBranch = _branch(10, largeLeaf, true, "");
        bytes memory rawThirtyOneByteChild = RLP.encode(bytes.concat(bytes30(0), hex"01"));
        bytes memory invalidShortRoot = _extension(hex"1a", rawThirtyOneByteChild, false);

        testCases[13] = _invalid(
            "invalid large child",
            keccak256(hashedBranch),
            hex"a0",
            _proof2(hashedBranch, wrongLargeLeaf),
            TrieProof.ProofError.INVALID_LARGE_NODE
        );
        testCases[14] = _invalid(
            "invalid short child",
            keccak256(invalidShortRoot),
            hex"abcd",
            _proof2(invalidShortRoot, _leaf(hex"3bcd", hex"2a")),
            TrieProof.ProofError.INVALID_SHORT_NODE
        );
    }

    function _setOpenZeppelinInvalidCases(TrieInvalidTestCase[] memory testCases) private pure {
        testCases[15] = _invalid(
            "OpenZeppelin wrong key proof",
            0x2858eebfa9d96c8a9e6a0cae9d86ec9189127110f132d63f07d3544c2a75a696,
            hex"6b6579316161",
            _proof3(
                hex"e216a04892c039d654f1be9af20e88ae53e9ab5fa5520190e0fb2f805823e45ebad22f",
                hex"f84780d687206e6f746865728d33343938683472697568677765808080808080808080a0854405b57aa6dc458bc41899a761cbbb1f66a4998af6dd0e8601c1b845395ae38080808080",
                hex"d687206e6f746865728d33343938683472697568677765"
            ),
            TrieProof.ProofError.INVALID_SHORT_NODE
        );
        testCases[16] = _invalid(
            "OpenZeppelin invalid internal node hash",
            0xa827dff1a657bb9bb9a1c3abe9db173e2f1359f15eb06f1647ea21ac7c95d8fa,
            hex"aa",
            _proof3(
                hex"e21aa09862c6b113008c4204c13755693cbb868acc25ebaa98db11df8c89a0c0dd3157",
                hex"f380808080808080808080a0de2a9c6a46b6ea71ab9e881c8420570cf19e833c85df6026b04f085016e78f00c220118080808080",
                hex"de2a9c6a46b6ea71ab9e881c8420570cf19e833c85df6026b04f085016e78f"
            ),
            TrieProof.ProofError.INVALID_SHORT_NODE
        );
        testCases[17] = _invalid(
            "OpenZeppelin extra proof elements",
            0x278c88eb59beba4f8b94f940c41614bb0dd80c305859ebffcd6ce07c93ca3749,
            hex"aa",
            _proof4(
                hex"d91ad780808080808080808080c32081aac32081ab8080808080",
                hex"d780808080808080808080c32081aac32081ab8080808080",
                hex"c32081aa",
                hex"c32081aa"
            ),
            TrieProof.ProofError.INVALID_EXTRA_PROOF_ELEMENT
        );
    }

    function fixtureMalformed() public pure returns (TrieMalformedTestCase[] memory testCases) {
        testCases = new TrieMalformedTestCase[](3);
        testCases[0] = _malformed("RLP scalar instead of list", hex"01", hex"00");
        testCases[1] = _malformed("truncated long string", hex"b801", hex"00");
        testCases[2] = _malformed("OpenZeppelin invalid short child encoding", hex"c7820000822bad", hex"00");
    }

    function _membership(
        string memory name,
        bytes32 root,
        bytes memory key,
        bytes memory value,
        bytes[] memory proof
    )
        private
        pure
        returns (TrieMembershipTestCase memory)
    {
        return TrieMembershipTestCase({ name: name, root: root, key: key, value: value, proof: proof });
    }

    function _exclusion(
        string memory name,
        bytes32 root,
        bytes memory key,
        bytes[] memory proof
    )
        private
        pure
        returns (TrieExclusionTestCase memory)
    {
        return TrieExclusionTestCase({ name: name, root: root, key: key, proof: proof });
    }

    function _invalid(
        string memory name,
        bytes32 root,
        bytes memory key,
        bytes[] memory proof,
        TrieProof.ProofError expectedError
    )
        private
        pure
        returns (TrieInvalidTestCase memory)
    {
        return TrieInvalidTestCase({ name: name, root: root, key: key, proof: proof, expectedError: expectedError });
    }

    function _malformed(
        string memory name,
        bytes memory encoded,
        bytes memory key
    )
        private
        pure
        returns (TrieMalformedTestCase memory)
    {
        return TrieMalformedTestCase({ name: name, root: keccak256(encoded), key: key, proof: _proof1(encoded) });
    }

    function _leaf(bytes memory compactPath, bytes memory value) private pure returns (bytes memory) {
        bytes[] memory items = new bytes[](2);
        items[0] = RLP.encode(compactPath);
        items[1] = RLP.encode(value);
        return RLP.encode(items);
    }

    function _extension(
        bytes memory compactPath,
        bytes memory child,
        bool hashChild
    )
        private
        pure
        returns (bytes memory)
    {
        bytes[] memory items = new bytes[](2);
        items[0] = RLP.encode(compactPath);
        items[1] = hashChild ? RLP.encode(keccak256(child)) : child;
        return RLP.encode(items);
    }

    function _branch(
        uint8 childIndex,
        bytes memory child,
        bool hashChild,
        bytes memory value
    )
        private
        pure
        returns (bytes memory)
    {
        bytes[] memory items = new bytes[](17);
        for (uint256 i = 0; i < items.length; ++i) {
            items[i] = RLP.encode(bytes(""));
        }
        if (child.length != 0) {
            items[childIndex] = hashChild ? RLP.encode(keccak256(child)) : child;
        }
        items[16] = RLP.encode(value);
        return RLP.encode(items);
    }

    function _list3(bytes memory a, bytes memory b, bytes memory c) private pure returns (bytes memory) {
        bytes[] memory items = new bytes[](3);
        items[0] = RLP.encode(a);
        items[1] = RLP.encode(b);
        items[2] = RLP.encode(c);
        return RLP.encode(items);
    }

    function _longValue() private pure returns (bytes memory) {
        return hex"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    }

    function _proof0() private pure returns (bytes[] memory proof) {
        return new bytes[](0);
    }

    function _proof1(bytes memory a) private pure returns (bytes[] memory proof) {
        proof = new bytes[](1);
        proof[0] = a;
    }

    function _proof2(bytes memory a, bytes memory b) private pure returns (bytes[] memory proof) {
        proof = new bytes[](2);
        proof[0] = a;
        proof[1] = b;
    }

    function _proof3(bytes memory a, bytes memory b, bytes memory c) private pure returns (bytes[] memory proof) {
        proof = new bytes[](3);
        proof[0] = a;
        proof[1] = b;
        proof[2] = c;
    }

    function _proof4(
        bytes memory a,
        bytes memory b,
        bytes memory c,
        bytes memory d
    )
        private
        pure
        returns (bytes[] memory proof)
    {
        proof = new bytes[](4);
        proof[0] = a;
        proof[1] = b;
        proof[2] = c;
        proof[3] = d;
    }
}
