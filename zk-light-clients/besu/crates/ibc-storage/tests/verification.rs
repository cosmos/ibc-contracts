use besu_zk_types::{
    keccak256, Eip1186ProofWitness, MembershipEntryWitness, MembershipWitness,
    NonMembershipWitness, MEMBERSHIP_SCHEMA_VERSION, SCHEMA_VERSION,
};
use evm_mpt::rlp::{encode_bytes, encode_list, encode_u256};
use ibc_storage::{
    logical_storage_slot, path_digest, terminal_path, verify_membership, verify_non_membership,
    IbcStorageError, PathKind,
};

fn leaf_for_secure_key(key: [u8; 32], value: &[u8]) -> Vec<u8> {
    let mut compact_path = Vec::with_capacity(33);
    compact_path.push(0x20);
    compact_path.extend_from_slice(&key);
    encode_list(&[encode_bytes(&compact_path), encode_bytes(value)])
}

fn account_proof(router: [u8; 20], storage_root: [u8; 32]) -> ([u8; 32], Vec<Vec<u8>>) {
    let account = encode_list(&[
        encode_bytes(&[]),
        encode_bytes(&[]),
        encode_bytes(&storage_root),
        encode_bytes(&keccak256(&[])),
    ]);
    let leaf = leaf_for_secure_key(keccak256(&router), &account);
    (keccak256(&leaf), vec![leaf])
}

fn membership_witness_for(
    client_id: &[u8],
    kind: PathKind,
    sequence: u64,
    commitment: [u8; 32],
) -> MembershipWitness {
    let router = [0x11; 20];
    let path = terminal_path(client_id, kind, sequence);
    let terminal = path.first().map(Vec::as_slice).unwrap_or_default();
    let slot = logical_storage_slot(terminal);
    let storage_leaf = leaf_for_secure_key(keccak256(&slot), &encode_u256(&commitment));
    let storage_root = keccak256(&storage_leaf);
    let (state_root, account_proof) = account_proof(router, storage_root);
    MembershipWitness {
        schema_version: MEMBERSHIP_SCHEMA_VERSION,
        proof_height: 9,
        state_root,
        source_router: router,
        account_proof,
        entries: vec![MembershipEntryWitness {
            path,
            expected_commitment: commitment,
            storage_proof: vec![storage_leaf],
        }],
    }
}

fn membership_witness() -> MembershipWitness {
    let mut commitment = [0_u8; 32];
    commitment[31] = 0x42;
    membership_witness_for(b"client-7", PathKind::Packet, 42, commitment)
}

// Build a canonical two-level storage trie: 16 root children, each with two
// leaves. Distinct second nibbles avoid collisions without a general trie builder.
fn membership_batch_witness() -> Result<MembershipWitness, Box<dyn std::error::Error>> {
    let mut groups = vec![Vec::new(); 16];
    for sequence in 1_u64..=10_000 {
        let kind = if sequence % 2 == 0 {
            PathKind::Packet
        } else {
            PathKind::Acknowledgement
        };
        let path = terminal_path(b"client-7", kind, sequence);
        let key = keccak256(&logical_storage_slot(&path[0]));
        let group = &mut groups[usize::from(key[0] >> 4)];
        if group.len() == 2
            || group
                .iter()
                .any(|(existing, _): &([u8; 32], MembershipEntryWitness)| existing[0] == key[0])
        {
            continue;
        }
        let mut compact = vec![0x20];
        compact.extend_from_slice(&key[1..]);
        let commitment = keccak256(&sequence.to_be_bytes());
        let leaf = encode_list(&[
            encode_bytes(&compact),
            encode_bytes(&encode_u256(&commitment)),
        ]);
        group.push((
            key,
            MembershipEntryWitness {
                path,
                expected_commitment: commitment,
                storage_proof: vec![leaf],
            },
        ));
        if groups.iter().all(|group| group.len() == 2) {
            break;
        }
    }
    if groups.iter().any(|group| group.len() != 2) {
        return Err("could not populate storage trie fixture".into());
    }
    let mut root_fields = vec![encode_bytes(&[]); 17];
    let mut entries = Vec::new();
    for (index, group) in groups.into_iter().enumerate() {
        let mut fields = vec![encode_bytes(&[]); 17];
        for (key, entry) in &group {
            fields[usize::from(key[0] & 0x0f)] = encode_bytes(&keccak256(&entry.storage_proof[0]));
        }
        let branch = encode_list(&fields);
        root_fields[index] = encode_bytes(&keccak256(&branch));
        for (_, mut entry) in group {
            entry.storage_proof.insert(0, branch.clone());
            entries.push(entry);
        }
    }
    let root = encode_list(&root_fields);
    for entry in &mut entries {
        entry.storage_proof.insert(0, root.clone());
    }
    entries.sort_by_key(|entry| path_digest(&entry.path));
    let router = [0x11; 20];
    let (state_root, account_proof) = account_proof(router, keccak256(&root));
    Ok(MembershipWitness {
        schema_version: MEMBERSHIP_SCHEMA_VERSION,
        proof_height: 9,
        state_root,
        source_router: router,
        account_proof,
        entries,
    })
}

#[test]
fn membership_batches_bind_ordered_entries_and_public_values(
) -> Result<(), Box<dyn std::error::Error>> {
    let complete = membership_batch_witness()?;
    for count in [1, 2, 3, 5, 10, 20, 32] {
        let mut witness = complete.clone();
        witness.entries.truncate(count);
        let values = verify_membership(&witness)?;
        let leaves: Vec<_> = witness
            .entries
            .iter()
            .map(|entry| {
                batch_merkle::membership_leaf(path_digest(&entry.path), entry.expected_commitment)
            })
            .collect();
        assert_eq!(values.entry_count, u32::try_from(count)?);
        assert_eq!(values.batch_root, batch_merkle::root(&leaves)?);
        assert_eq!(values.schema_version, witness.schema_version);
        assert_eq!(values.proof_height, witness.proof_height);
        assert_eq!(values.state_root, witness.state_root);
        assert_eq!(values.source_router, witness.source_router);
    }
    Ok(())
}

#[test]
fn membership_batches_reject_empty_oversized_duplicate_and_unordered_entries(
) -> Result<(), Box<dyn std::error::Error>> {
    let valid = membership_batch_witness()?;
    let mut witness = valid.clone();
    witness.entries.clear();
    assert_eq!(
        verify_membership(&witness),
        Err(IbcStorageError::BatchMerkle(
            batch_merkle::BatchMerkleError::EmptyBatch
        ))
    );

    witness = valid.clone();
    witness.entries.push(valid.entries[0].clone());
    assert_eq!(
        verify_membership(&witness),
        Err(IbcStorageError::BatchMerkle(
            batch_merkle::BatchMerkleError::BatchTooLarge(33)
        ))
    );

    witness = valid.clone();
    witness.entries.insert(1, witness.entries[0].clone());
    witness.entries.truncate(32);
    assert_eq!(
        verify_membership(&witness),
        Err(IbcStorageError::DuplicatePathDigest)
    );

    witness = valid;
    witness.entries.swap(0, 1);
    assert_eq!(
        verify_membership(&witness),
        Err(IbcStorageError::NonCanonicalBatchOrder)
    );
    Ok(())
}

#[test]
fn membership_batches_authenticate_every_entry() -> Result<(), Box<dyn std::error::Error>> {
    let valid = membership_batch_witness()?;
    for index in 0..valid.entries.len() {
        let mut witness = valid.clone();
        witness.entries[index].expected_commitment[0] ^= 1;
        assert_eq!(
            verify_membership(&witness),
            Err(IbcStorageError::CommitmentMismatch)
        );

        witness = valid.clone();
        witness.entries[index].storage_proof[0][0] ^= 1;
        assert!(matches!(
            verify_membership(&witness),
            Err(IbcStorageError::Mpt(_))
        ));

        witness = valid.clone();
        witness.entries[index].path = terminal_path(b"client-7", PathKind::Receipt, 42);
        assert_eq!(
            verify_membership(&witness),
            Err(IbcStorageError::WrongPathType(2))
        );
    }
    Ok(())
}

#[test]
fn verifies_complete_router_membership_statement() -> Result<(), Box<dyn std::error::Error>> {
    let witness = membership_witness();
    let public_values = verify_membership(&witness)?;
    assert_eq!(public_values.entry_count, 1);
    assert_eq!(public_values.state_root, witness.state_root);
    assert_eq!(
        public_values.batch_root,
        batch_merkle::membership_leaf(
            path_digest(&witness.entries[0].path),
            witness.entries[0].expected_commitment,
        )
    );
    Ok(())
}

#[test]
fn binds_exact_packet_and_ack_commitments_from_pinned_upstream_vectors(
) -> Result<(), Box<dyn std::error::Error>> {
    // Produced by the pinned ICS24Host implementation and cross-checked in
    // contracts/test/UpstreamIBCCommitmentVectors.t.sol.
    let mut packet = [0_u8; 32];
    hex::decode_to_slice(
        "2e6c14ba0b7da7433158ba2356eeb1dd6ecde71a9f69ce0f2d9c5e38073146eb",
        &mut packet,
    )?;
    let mut acknowledgement = [0_u8; 32];
    hex::decode_to_slice(
        "f03b4667413e56aaf086663267913e525c442b56fa1af4fa3f3dab9f37044c5b",
        &mut acknowledgement,
    )?;
    for witness in [
        membership_witness_for(b"client-0", PathKind::Packet, 1, packet),
        membership_witness_for(b"client-2", PathKind::Acknowledgement, 3, acknowledgement),
    ] {
        let values = verify_membership(&witness)?;
        assert_eq!(values.entry_count, 1);

        let mut mutated = witness.clone();
        mutated.entries[0].expected_commitment[0] ^= 1;
        assert_eq!(
            verify_membership(&mutated),
            Err(IbcStorageError::CommitmentMismatch)
        );
        mutated = witness.clone();
        mutated.entries[0].path = terminal_path(b"client-0", PathKind::Packet, 2);
        assert!(verify_membership(&mutated).is_err());
        mutated = witness.clone();
        mutated.entries[0].storage_proof[0][0] ^= 1;
        assert!(verify_membership(&mutated).is_err());
    }
    Ok(())
}

#[test]
fn membership_rejects_wrong_value_and_untrusted_shapes() {
    let mut witness = membership_witness();
    witness.entries[0].expected_commitment[31] ^= 1;
    assert_eq!(
        verify_membership(&witness),
        Err(IbcStorageError::CommitmentMismatch)
    );

    witness = membership_witness();
    witness.source_router = [0; 20];
    assert_eq!(
        verify_membership(&witness),
        Err(IbcStorageError::ZeroSourceRouter)
    );

    witness = membership_witness();
    witness.schema_version = MEMBERSHIP_SCHEMA_VERSION + 1;
    assert_eq!(
        verify_membership(&witness),
        Err(IbcStorageError::UnsupportedSchemaVersion(
            MEMBERSHIP_SCHEMA_VERSION + 1
        ))
    );

    witness = membership_witness();
    witness.proof_height = 0;
    assert_eq!(
        verify_membership(&witness),
        Err(IbcStorageError::ZeroProofHeight)
    );

    witness = membership_witness();
    witness.state_root[0] ^= 1;
    assert!(verify_membership(&witness).is_err());

    witness = membership_witness();
    witness.entries[0].path = terminal_path(b"client-7", PathKind::Acknowledgement, 42);
    assert!(verify_membership(&witness).is_err());

    witness = membership_witness();
    witness.account_proof[0][0] ^= 1;
    assert!(verify_membership(&witness).is_err());

    witness = membership_witness();
    witness.entries[0].storage_proof[0][0] ^= 1;
    assert!(verify_membership(&witness).is_err());
}

#[test]
fn verifies_receipt_absence_under_an_existing_router_account(
) -> Result<(), Box<dyn std::error::Error>> {
    let router = [0x22; 20];
    let empty_storage_root = keccak256(&encode_bytes(&[]));
    let (state_root, account_proof) = account_proof(router, empty_storage_root);
    let witness = NonMembershipWitness {
        schema_version: SCHEMA_VERSION,
        proof_height: 10,
        state_root,
        source_router: router,
        path: terminal_path(b"client-8", PathKind::Receipt, 43),
        proof: Eip1186ProofWitness {
            account_proof,
            storage_proof: Vec::new(),
        },
    };
    let public_values = verify_non_membership(&witness)?;
    assert_eq!(public_values.proof_height, witness.proof_height);
    assert_eq!(public_values.path_digest, path_digest(&witness.path));
    Ok(())
}

#[test]
fn non_membership_rejects_present_receipt_even_when_word_is_zero() {
    let router = [0x33; 20];
    let path = terminal_path(b"client-9", PathKind::Receipt, 44);
    let terminal = path.first().map(Vec::as_slice).unwrap_or_default();
    let slot = logical_storage_slot(terminal);
    let storage_leaf = leaf_for_secure_key(keccak256(&slot), &encode_u256(&[0_u8; 32]));
    let storage_root = keccak256(&storage_leaf);
    let (state_root, account_proof) = account_proof(router, storage_root);
    let witness = NonMembershipWitness {
        schema_version: SCHEMA_VERSION,
        proof_height: 11,
        state_root,
        source_router: router,
        path,
        proof: Eip1186ProofWitness {
            account_proof,
            storage_proof: vec![storage_leaf],
        },
    };
    assert!(verify_non_membership(&witness).is_err());
}

#[test]
fn non_membership_binds_schema_height_router_path_root_and_exact_exclusion() {
    let router = [0x22; 20];
    let empty_storage_root = keccak256(&encode_bytes(&[]));
    let (state_root, account_proof) = account_proof(router, empty_storage_root);
    let valid = NonMembershipWitness {
        schema_version: SCHEMA_VERSION,
        proof_height: 10,
        state_root,
        source_router: router,
        path: terminal_path(b"client-8", PathKind::Receipt, 43),
        proof: Eip1186ProofWitness {
            account_proof,
            storage_proof: Vec::new(),
        },
    };

    let mut witness = valid.clone();
    witness.schema_version += 1;
    assert!(matches!(
        verify_non_membership(&witness),
        Err(IbcStorageError::UnsupportedSchemaVersion(_))
    ));
    witness = valid.clone();
    witness.proof_height = 0;
    assert_eq!(
        verify_non_membership(&witness),
        Err(IbcStorageError::ZeroProofHeight)
    );
    witness = valid.clone();
    witness.source_router = [0; 20];
    assert_eq!(
        verify_non_membership(&witness),
        Err(IbcStorageError::ZeroSourceRouter)
    );
    witness = valid.clone();
    witness.path = terminal_path(b"client-8", PathKind::Packet, 43);
    assert!(verify_non_membership(&witness).is_err());
    witness = valid.clone();
    witness.path = terminal_path(b"client-8", PathKind::Receipt, 44);
    assert!(verify_non_membership(&witness).is_ok());
    witness = valid.clone();
    witness.state_root[0] ^= 1;
    assert!(verify_non_membership(&witness).is_err());
    witness = valid;
    witness.proof.account_proof.clear();
    assert!(verify_non_membership(&witness).is_err());
}
