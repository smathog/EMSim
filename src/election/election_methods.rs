/// This module contains the actual election methods themselves.
/// Each should be of the form fn name<T: Voter>(voters, n, tie_resolver) -> Vec<CandidateID>
/// Where voters: &Vec<T>, num_candidates is usize number of candidates,
/// tie_resolver is some function to break ties as they emerge,
/// and the return is a sorted vec in order of finish (i.e. vec[0] is the winner, vec[1] is
/// the runner-up, etc.

use crate::election::election_profile::CandidateID;
use crate::election::voters::*;
use std::cmp::Ordering;

/// Also known as FPTP (First Past the Post).
/// All voters vote for their top-ranked candidate. The candidate with the most votes wins.
pub fn plurality<T: Voter, F: Fn(&usize, &usize) -> Ordering>(
    voters: &Vec<T>,
    num_candidates: usize,
    tie_breaker: F,
) -> Vec<CandidateID> {
    let mut vote_totals = vec![0usize; num_candidates];
    for voter in voters {
        let ballot = voter.cast_ordinal_ballot();
        let choice = ballot[0].0;
        vote_totals[choice] += 1;
    }
    let mut results = (0..num_candidates)
        .map(|v| CandidateID(v))
        .collect::<Vec<_>>();
    results.sort_by(|&CandidateID(a), &CandidateID(b)| {
        vote_totals[b]
            .partial_cmp(&vote_totals[a])
            .unwrap()
            .then(tie_breaker(&a, &b))
    });
    results
}

/// Unit tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    // plurality unit tests
    #[test]
    fn test_plurality() {
        let mut voters = Vec::new();
        voters.push(HonestVoter::new(vec![0.1, 0.4, 0.6], true));
        voters.push(HonestVoter::new(vec![0.5, 0.4, 0.8], true));
        voters.push(HonestVoter::new(vec![0.3, 0.7, 0.2], false));
        assert_eq!(plurality(&voters, 3, usize::cmp),
        vec![CandidateID(2), CandidateID(1), CandidateID(0)])
    }
}
