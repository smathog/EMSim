/// This module contains the actual election methods themselves.
/// Each should be of the form fn name<T: Voter>(voters, n, tie_resolver) -> Vec<CandidateID>
/// Where voters: &mut Vec<T>, num_candidates is usize number of candidates,
/// tie_resolver is some function to break ties as they emerge,
/// and the return is a sorted vec in order of finish (i.e. vec[0] is the winner, vec[1] is
/// the runner-up, etc.
use crate::election::election_profile::CandidateID;
use crate::election::voters::*;
use std::cmp::Ordering;

use invoker_macro::invoke_all;

/// Struct to serve as a namespace for election method implementations.
/// Additionally should allow for a proc macro to operate over its impl block of methods to
/// automate things like executing all methods on a given Vec of voters and counting the number of
/// available election methods.
pub struct ElectionMethods;

#[invoke_all]
impl ElectionMethods {
    /// Also known as FPTP (First Past the Post).
    /// All voters vote for their top-ranked candidate. The candidate with the most votes wins.
    pub fn plurality<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        // Method identifier:
        let method_name = "plurality";
        plurality_driver(voters, num_candidates, tie_breaker, method_name)
    }

    /// Top-two runoff, with the top-two winners determined via an initial non-instant FPTP race.
    pub fn fptp_runoff<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        // Method identifier:
        let method_name = "fptp_runoff";

        // Get a FPTP ranking:
        let mut fptp_ranking = plurality_driver(voters, num_candidates, tie_breaker, method_name);

        // Find which of the top-two FPTP ranked candidates is preferred
        let mut vote_totals = vec![0usize; 2];
        for voter in voters {
            match voter.honest_preference(fptp_ranking[0], fptp_ranking[1]) {
                Ordering::Less => vote_totals[1] += 1,
                Ordering::Equal => {} // No preference
                Ordering::Greater => vote_totals[0] += 1,
            }
        }
        let winner = (0usize..2)
            .max_by(|u1, u2| {
                vote_totals[*u1]
                    .partial_cmp(&vote_totals[*u2])
                    .unwrap()
                    .then(tie_breaker(u1, u2))
            })
            .unwrap();
        if winner == 1 {
            fptp_ranking.swap(0, 1);
        }
        fptp_ranking
    }

    /// Voters cast ordinal ballots. Top-two candidates by plurality advance to an instant runoff.
    pub fn contingent_vote<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "contingent_vote";

        // Get ordinal ballots
        let ballots = voters
            .iter_mut()
            .map(|v| v.cast_ordinal_ballot(method_name))
            .collect::<Vec<_>>();

        // Run FPTP election:
        let mut vote_totals = vec![0; num_candidates];
        for &ballot in &ballots {
            let CandidateID(top) = ballot[0];
            vote_totals[top] += 1;
        }

        // Get FPTP ranking of candidates:
        let mut candidates = (0..num_candidates)
            .map(|i| CandidateID(i))
            .collect::<Vec<_>>();
        sort_candidates_by_vec(&mut candidates, &vote_totals, tie_breaker);

        // See whether candidate first or second is preferred on ballots:
        let (first_c, second_c) = (candidates[0], candidates[1]);
        let votes = ballots
            .into_iter()
            .fold((0, 0), |(mut first, mut second), ballot| {
                for &candidate in ballot {
                    if first_c == candidate {
                        first += 1;
                        break;
                    } else if second_c == candidate {
                        second += 1;
                        break;
                    }
                }
                (first, second)
            });

        if votes.0 > votes.1 {
            candidates
        } else if votes.1 > votes.0 {
            candidates.swap(0, 1);
            candidates
        } else {
            match tie_breaker(&first_c.0, &second_c.0) {
                Ordering::Less => {
                    candidates.swap(0, 1);
                    candidates
                }
                Ordering::Equal => {
                    panic!("Tie-breakers into voting methods should be decisive!")
                }
                Ordering::Greater => candidates,
            }
        }
    }



}

/// Driver for plurality elections; necessary so that voters who use method-based strategic voting
/// can differentiate between FPTP and TTR
fn plurality_driver<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
    voters: &mut Vec<T>,
    num_candidates: usize,
    tie_breaker: F,
    method_name: &str,
) -> Vec<CandidateID> {
    // Calculate the vote total each candidate has earned
    let mut vote_totals = vec![0usize; num_candidates];
    for voter in voters {
        let ballot = voter.cast_ordinal_ballot(method_name);
        let choice = ballot[0].0;
        vote_totals[choice] += 1;
    }

    // Generate a list of candidates sorted descending on vote total
    let mut results = (0..num_candidates)
        .map(|v| CandidateID(v))
        .collect::<Vec<_>>();
    sort_candidates_by_vec(&mut results, &vote_totals, tie_breaker);
    results
}

/// Helper function: given a vector of candidates and a vector of some quantity of the same length,
/// sorts the vector of candidates in decreasing order by the corresponding field in the quantity
/// vector (that is, Candidate(x) is sorted by key v[x] descending) with a passed-in tie breaker.
fn sort_candidates_by_vec<T: PartialOrd, F: Fn(&usize, &usize) -> Ordering + Copy>(
    candidates: &mut Vec<CandidateID>,
    v: &Vec<T>,
    tie_breaker: F,
) {
    candidates.sort_unstable_by(|&CandidateID(a), &CandidateID(b)| {
        v[b]
            .partial_cmp(&v[a])
            .unwrap()
            .then(tie_breaker(&a, &b))
    });
}

/// Unit tests for this module
#[cfg(test)]
mod tests {
    use crate::election::voters::ApprovalThresholdBehavior::Mean;
    use super::*;

    // Helper voter-production functions
    fn majority_election() -> Vec<HonestVoter> {
        let mut voters = Vec::new();
        voters.push(HonestVoter::new(vec![0.1, 0.4, 0.6], true, Mean));
        voters.push(HonestVoter::new(vec![0.5, 0.4, 0.8], true, Mean));
        voters.push(HonestVoter::new(vec![0.3, 0.7, 0.2], false, Mean));
        voters
    }

    fn runoff_differs() -> Vec<HonestVoter> {
        let mut voters = Vec::new();
        voters.push(HonestVoter::new(vec![0.1, 0.4, 0.6], true, Mean));
        voters.push(HonestVoter::new(vec![0.5, 0.4, 0.8], true, Mean));
        voters.push(HonestVoter::new(vec![0.5, 0.4, 0.8], true, Mean));
        voters.push(HonestVoter::new(vec![0.5, 0.4, 0.8], true, Mean));
        voters.push(HonestVoter::new(vec![0.3, 0.7, 0.2], false, Mean));
        voters.push(HonestVoter::new(vec![0.3, 0.7, 0.2], false, Mean));
        voters.push(HonestVoter::new(vec![0.3, 0.7, 0.2], false, Mean));
        voters.push(HonestVoter::new(vec![0.8, 0.6, 0.1], false, Mean));
        voters.push(HonestVoter::new(vec![0.8, 0.6, 0.1], false, Mean));
        voters
    }

    // plurality unit tests
    #[test]
    fn test_plurality() {
        assert_eq!(
            ElectionMethods::plurality(&mut majority_election(), 3, usize::cmp),
            vec![CandidateID(2), CandidateID(1), CandidateID(0)]
        );
        assert_eq!(
            ElectionMethods::plurality(&mut runoff_differs(), 3, usize::cmp),
            vec![CandidateID(2), CandidateID(1), CandidateID(0)]
        );
    }

    // fptp_runoff unit tests
    #[test]
    fn test_fptp_runoff() {
        assert_eq!(
            ElectionMethods::fptp_runoff(&mut majority_election(), 3, usize::cmp),
            vec![CandidateID(2), CandidateID(1), CandidateID(0)]
        );
        assert_eq!(
            ElectionMethods::fptp_runoff(&mut runoff_differs(), 3, usize::cmp),
            vec![CandidateID(1), CandidateID(2), CandidateID(0)]
        );
    }

    // Test invoke_all function
    #[test]
    fn test_all() {
        ElectionMethods::invoke_all(&mut runoff_differs(), 3, usize::cmp, |v| {
            println!("Called with {:?}", v)
        });
    }
}
