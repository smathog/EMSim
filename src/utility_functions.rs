//! Mod to contain utility functions (utility as in useful, not as in voter utility) that are
//! useful throughout the entire program and not better off siloed into a more specific
//! mod.

use std::cmp::Ordering;

use crate::election::CandidateID;

/// Helper function: generate a vec of CandidateIDs, from CandidateID(0) to CandidateID(n - 1)
pub fn generate_candidates(n: usize) -> Vec<CandidateID> {
    (0..n)
        .map(|i| CandidateID(i))
        .collect()
}

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

/// Helper function to scale utilities linearly so the min is 0 and max is 1, provided min != max
pub fn scale_utilities_linearly(utilities: &Vec<f64>) -> Vec<f64> {
    let max = utilities
        .iter()
        .max_by(|&a, &b| a.partial_cmp(b).unwrap())
        .copied()
        .unwrap();
    let min = utilities
        .iter()
        .min_by(|&a, &b| a.partial_cmp(b).unwrap())
        .copied()
        .unwrap();
    utilities
        .iter()
        .map(|&f| {
            let f = if max != min {
                (f - min) / (max - min)
            } else {
                max
            };
            f.clamp(0f64, 1f64)
        })
        .collect()
}

/// Helper function to generate approval ballots based on a set bound:
pub fn generate_approval_ballot(utilities: &Vec<f64>, bound: f64) -> Vec<CandidateID> {
    let mut ballot: Vec<CandidateID> = (0..(utilities.len()))
        .filter(|&i| utilities[i] >= bound)
        .map(|i| CandidateID(i))
        .collect();
    if ballot.is_empty() {
        ballot.push(CandidateID(
            utilities
                .iter()
                .copied()
                .enumerate()
                .map(|(i, u)| (u, i))
                .max_by(|&(a, _), &(b, _)| a.partial_cmp(&b).unwrap())
                .unwrap()
                .1,
        ));
    }
    ballot
}

#[cfg(test)]
mod tests {
    use crate::election::CandidateID;
    use crate::utility_functions::{generate_candidates, sort_candidates_by_vec};

    #[test]
    fn test_sort_candidates_by_vec() {
        let mut v = generate_candidates(3);
        let key = vec![20, 50, 10];
        sort_candidates_by_vec(&mut v, &key, usize::cmp);
        assert_eq!(v, vec![CandidateID(1), CandidateID(0), CandidateID(2)])
    }
}

