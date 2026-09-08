// SPDX-License-Identifier: Apache-2.0

//! Quorum-based consensus utilities for aggregating attestor responses.

use thiserror::Error;

/// Selects the minimum height from a collection that meets quorum requirements.
///
/// Returns the minimum height if at least `quorum_threshold` heights are provided.
/// The minimum is chosen because it guarantees all attestors in the quorum have
/// observed at least that height.
///
/// # Errors
/// Returns an error if fewer than `quorum_threshold` heights are provided.
pub fn select_quorum_height(
    heights: Vec<u64>,
    quorum_threshold: usize,
) -> Result<u64, QuorumError> {
    if heights.len() < quorum_threshold {
        return Err(QuorumError::InsufficientResponses {
            received: heights.len(),
            required: quorum_threshold,
        });
    }

    heights
        .into_iter()
        .min()
        .ok_or(QuorumError::InsufficientResponses {
            received: 0,
            required: quorum_threshold,
        })
}

/// Errors that can occur during quorum operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum QuorumError {
    #[error("quorum not met: got {received} responses, need {required}")]
    InsufficientResponses { received: usize, required: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_minimum_height_when_quorum_met() {
        let heights = vec![100, 98, 99];
        let result = select_quorum_height(heights, 2);
        assert_eq!(result, Ok(98));
    }

    #[test]
    fn returns_minimum_with_exact_quorum() {
        let heights = vec![50, 60];
        let result = select_quorum_height(heights, 2);
        assert_eq!(result, Ok(50));
    }

    #[test]
    fn returns_single_height_when_quorum_is_one() {
        let heights = vec![42];
        let result = select_quorum_height(heights, 1);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn fails_when_below_quorum() {
        let heights = vec![100];
        let result = select_quorum_height(heights, 2);
        assert_eq!(
            result,
            Err(QuorumError::InsufficientResponses {
                received: 1,
                required: 2
            })
        );
    }

    #[test]
    fn fails_with_empty_heights() {
        let heights: Vec<u64> = vec![];
        let result = select_quorum_height(heights, 1);
        assert_eq!(
            result,
            Err(QuorumError::InsufficientResponses {
                received: 0,
                required: 1
            })
        );
    }

    #[test]
    fn handles_duplicate_heights() {
        let heights = vec![100, 100, 100];
        let result = select_quorum_height(heights, 2);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn handles_large_height_differences() {
        let heights = vec![1_000_000, 1, 500_000];
        let result = select_quorum_height(heights, 3);
        assert_eq!(result, Ok(1));
    }
}
