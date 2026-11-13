use besu_qbft::{target_quorum, trusted_overlap_threshold};

#[test]
fn quorum_formulas_cover_all_residue_classes_and_large_values() {
    for count in [
        1_usize,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        usize::MAX - 2,
        usize::MAX - 1,
        usize::MAX,
    ] {
        let expected_target =
            count / 3 * 2 + usize::from(count % 3 != 0) + usize::from(count % 3 == 2);
        assert_eq!(target_quorum(count), Ok(expected_target));
        assert_eq!(trusted_overlap_threshold(count), Ok(count / 3 + 1));
    }
}
