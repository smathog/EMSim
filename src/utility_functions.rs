//! Mod to contain utility functions (utility as in useful, not as in voter utility) that are
//! useful throughout the entire program and not better off siloed into a more specific
//! mod.

use std::cmp::Ordering;

use crate::election::CandidateID;

/// Helper function: given a vector of candidates and a vector of some quantity of the same length,
/// sorts the vector of candidates in decreasing order by the corresponding field in the quantity
/// vector (that is, Candidate(x) is sorted by key v\[x] descending) with a passed-in tie breaker.
pub fn sort_candidates_by_vec<T: PartialOrd, F: Fn(&usize, &usize) -> Ordering + Copy>(
    candidates: &mut Vec<CandidateID>,
    v: &Vec<T>,
    tie_breaker: F,
) {
    candidates.sort_unstable_by(|&CandidateID(a), &CandidateID(b)| {
        v[b].partial_cmp(&v[a]).unwrap().then(tie_breaker(&b, &a))
    });
}

