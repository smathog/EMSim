/// This module contains the actual election methods themselves.
/// Each should be of the form fn name<T: Voter>(voters, n, tie_resolver) -> Vec<CandidateID>
/// Where voters: &mut Vec<T>, num_candidates is usize number of candidates,
/// tie_resolver is some function to break ties as they emerge,
/// and the return is a sorted vec in order of finish (i.e. vec[0] is the winner, vec[1] is
/// the runner-up, etc.
use crate::election::election_profile::CandidateID;
use crate::election::voters::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::VecDeque;

use invoke_impl::invoke_impl;

/// Struct to serve as a namespace for election method implementations.
/// Additionally should allow for a proc macro to operate over its impl block of methods to
/// automate things like executing all methods on a given Vec of voters and counting the number of
/// available election methods.
pub struct ElectionMethods;

#[invoke_impl(name("ordinal"))]
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
        let winner = honest_runoff_driver(voters, tie_breaker, fptp_ranking[0], fptp_ranking[1]);
        if winner == fptp_ranking[1] {
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

    /// Instant-runoff voting, also known as the alternative vote or ranked choice voting.
    /// Voters cast ordinal ballots. At each round, a ballot's top active preference is counted
    /// as a plurality vote. The candidate with the lowest total is eliminated and the ballots are
    /// transferred. The process continues until a single candidate wins.
    pub fn irv<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "irv";

        // Get ballots as stacks
        let mut stack_ballots = voters
            .iter_mut()
            .map(|v| {
                v.cast_ordinal_ballot(method_name)
                    .into_iter()
                    .collect::<VecDeque<_>>()
            })
            .collect::<Vec<_>>();

        // Set up set for eliminated candidates
        let mut eliminated = HashSet::with_capacity(num_candidates);
        // Vec for elimination order, will reverse to get final ranking
        let mut elimination_order = Vec::with_capacity(num_candidates);
        // Vec for plurality vote for each round
        let mut plurality = vec![0usize; num_candidates];

        loop {
            // Tabulate plurality ballots for this round
            for ballot in &mut stack_ballots {
                // Get rid of the front of the ballot until it contains a non-eliminated candidate
                // or is empty
                while let Some(&CandidateID(value)) = ballot.front() {
                    if !eliminated.contains(value) {
                        break;
                    }
                    ballot.pop_front();
                }

                //If ballot not exhausted
                if let Some(&CandidateID(id)) = ballot.front().copied() {
                    plurality[id] += 1;
                }
            }

            // Find the loser of the round
            let loser = plurality
                .iter()
                .copied()
                .enumerate()
                .filter(|(i, _)| !eliminated.contains(i))
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap().then(tie_breaker(a, b)))
                .unwrap()
                .0;
            elimination_order.push(CandidateID(loser));
            eliminated.insert(loser);
            if elimination_order.len() == num_candidates - 1 {
                // If final round, add winner and terminate
                let winner = (0..num_candidates)
                    .find(|i| !eliminated.contains(i))
                    .unwrap();
                elimination_order.push(CandidateID(winner));
                elimination_order.reverse();
                break elimination_order;
            } else {
                // Otherwise, reset plurality vec for next round
                plurality.iter_mut().for_each(|v| *v = 0);
            }
        }
    }
}

#[invoke_impl(name("cardinal"))]
impl ElectionMethods {
    /// Voters cast approval votes. The candidate with the most approval wins.
    pub fn approval<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "approval";
        approval_driver(voters, num_candidates, tie_breaker, method_name)
    }

    /// Voters cast approval votes. The two candidates with the highest approvals advance to a non-
    /// instant runoff.
    pub fn approval_runoff<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "approval_runoff";

        // Get approval ranking:
        let mut approval_ranking =
            approval_driver(voters, num_candidates, tie_breaker, method_name);

        let winner = honest_runoff_driver(
            voters,
            tie_breaker,
            approval_ranking[0],
            approval_ranking[1],
        );
        if winner == approval_ranking[1] {
            approval_ranking.swap(0, 1);
        }
        approval_ranking
    }

    /// Score voting with a rating range of 0-5
    pub fn score_5<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "score_5";
        score_driver(voters, num_candidates, tie_breaker, 5, method_name)
    }

    /// Score voting with a rating range of 0-10
    pub fn score_10<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "score_10";
        score_driver(voters, num_candidates, tie_breaker, 10, method_name)
    }

    /// Score voting with a range of 0-100
    pub fn score_100<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "score_100";
        score_driver(voters, num_candidates, tie_breaker, 100, method_name)
    }

    /// Score voting with a rating range of 0-5
    /// Followed by a delayed runoff.
    pub fn score_5_runoff<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "score_5_runoff";
        let mut scores = score_driver(voters, num_candidates, tie_breaker, 5, method_name);
        let winner = honest_runoff_driver(voters, tie_breaker, scores[0], scores[1]);
        if winner == scores[1] {
            scores.swap(0, 1);
        }
        scores
    }

    /// Score voting with a rating range of 0-10
    /// Followed by a delayed runoff
    pub fn score_10_runoff<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "score_10_runoff";
        let mut scores = score_driver(voters, num_candidates, tie_breaker, 10, method_name);
        let winner = honest_runoff_driver(voters, tie_breaker, scores[0], scores[1]);
        if winner == scores[1] {
            scores.swap(0, 1);
        }
        scores
    }

    /// Score voting with a range of 0-100
    /// Followed by a delayed runoff
    pub fn score_100_runoff<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "score_100_runoff";
        let mut scores = score_driver(voters, num_candidates, tie_breaker, 100, method_name);
        let winner = honest_runoff_driver(voters, tie_breaker, scores[0], scores[1]);
        if winner == scores[1] {
            scores.swap(0, 1);
        }
        scores
    }

    /// Score voting with a range of 0-5.
    /// Followed by an instant runoff based on cardinal ballots.
    pub fn star_5<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "star_5";
        star_driver(voters, num_candidates, tie_breaker, 5, method_name)
    }

    /// Score voting with a range of 0-10.
    /// Followed by an instant runoff based on cardinal ballots.
    pub fn star_10<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "star_10";
        star_driver(voters, num_candidates, tie_breaker, 10, method_name)
    }

    /// Score voting with a range of 0-100.
    /// Followed by an instant runoff based on cardinal ballots.
    pub fn star_100<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID> {
        let method_name = "star_100";
        star_driver(voters, num_candidates, tie_breaker, 100, method_name)
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

/// Driver for score elections; avoids code duplication for Score5, Score10, and Score100
fn score_driver<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
    voters: &mut Vec<T>,
    num_candidates: usize,
    tie_breaker: F,
    range: usize,
    method_name: &str,
) -> Vec<CandidateID> {
    // Calculate the vote total each candidate has earned
    let mut vote_totals = vec![0usize; num_candidates];
    for voter in voters {
        voter
            .cast_cardinal_ballot(range, method_name)
            .into_iter()
            .copied()
            .enumerate()
            .for_each(|(id, score)| vote_totals[id] += score)
    }

    // Generate a list of candidates sorted descending on vote total
    let mut results = (0..num_candidates)
        .map(|v| CandidateID(v))
        .collect::<Vec<_>>();
    sort_candidates_by_vec(&mut results, &vote_totals, tie_breaker);
    results
}

/// Driver for approval voting to avoid code duplication
fn approval_driver<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
    voters: &mut Vec<T>,
    num_candidates: usize,
    tie_breaker: F,
    method_name: &str,
) -> Vec<CandidateID> {
    let mut approval_count = vec![0; num_candidates];
    voters
        .iter_mut()
        .map(|v| v.cast_approval_ballot(method_name))
        .for_each(|ballot| {
            ballot
                .iter()
                .for_each(|&CandidateID(id)| approval_count[id] += 1)
        });
    let mut candidates = (0..num_candidates).map(|i| CandidateID(i)).collect();
    sort_candidates_by_vec(&mut candidates, &approval_count, tie_breaker);
    candidates
}

/// Simulates an honest delayed runoff between two candidates.
fn honest_runoff_driver<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
    voters: &mut Vec<T>,
    tie_breaker: F,
    first_candidate: CandidateID,
    second_candidate: CandidateID,
) -> CandidateID {
    let mut vote_totals = vec![0usize; 2];
    for voter in voters {
        match voter.honest_preference(first_candidate, second_candidate) {
            Ordering::Less => vote_totals[1] += 1,
            Ordering::Equal => {} // No preference
            Ordering::Greater => vote_totals[0] += 1,
        }
    }
    match (0usize..2).max_by(|u1, u2| {
        vote_totals[*u1]
            .partial_cmp(&vote_totals[*u2])
            .unwrap()
            .then(tie_breaker(u1, u2))
    }) {
        Some(0) => first_candidate,
        Some(1) => second_candidate,
        _ => panic!("Tie must be definitely broken by this point!"),
    }
}

/// Driver function for STAR methods
fn star_driver<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
    voters: &mut Vec<T>,
    num_candidates: usize,
    tie_breaker: F,
    range: usize,
    method_name: &str,
) -> Vec<CandidateID> {
    let method_name = "star_5";

    // Get a vec of cardinal ballots
    let ballots = voters
        .iter_mut()
        .map(|v| v.cast_cardinal_ballot(range, method_name))
        .collect::<Vec<_>>();

    // Use ballots to generate scores for candidates
    let mut scores = vec![0; num_candidates];
    ballots.iter().for_each(|ballot| {
        ballot
            .iter()
            .zip(scores.iter_mut())
            .for_each(|(&score, total)| {
                *total += score;
            })
    });

    // Generate a ranking of candidates:
    let mut candidates = (0..num_candidates)
        .map(|i| CandidateID(i))
        .collect::<Vec<_>>();
    sort_candidates_by_vec(&mut candidates, &scores, tie_breaker);

    // Determine which of candidates[0] and candidates[1] is preferred base on the ballots:
    let (CandidateID(first_index), CandidateID(second_index)) = (candidates[0], candidates[1]);
    let (mut first, mut second) = (0, 0);
    ballots.into_iter().for_each(|ballot| {
        if ballot[first] > ballot[second] {
            first += 1;
        } else if ballot[first] < ballot[second] {
            second += 1;
        } else {
            match tie_breaker(&ballot[first], &ballot[second]) {
                Ordering::Less => second += 1,
                Ordering::Equal => {
                    panic!("Tie-breaker functions must not return equal!")
                }
                Ordering::Greater => first += 1,
            }
        }
    });
    if first > second {
        candidates
    } else if second < first {
        candidates.swap(0, 1);
        candidates
    } else {
        match tie_breaker(&first, &second) {
            Ordering::Less => {
                candidates.swap(0, 1);
                candidates
            }
            Ordering::Equal => {
                panic!("Tie-breaker functions must not return equal!")
            }
            Ordering::Greater => candidates,
        }
    }
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
        v[b].partial_cmp(&v[a]).unwrap().then(tie_breaker(&b, &a))
    });
}

/// Unit tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::election::voters::ApprovalThresholdBehavior::Mean;
    use crate::election::voters::*;

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
        for _ in 0..3 {
            voters.push(HonestVoter::new(vec![0.5, 0.4, 0.8], true, Mean));
        }
        for _ in 0..3 {
            voters.push(HonestVoter::new(vec![0.3, 0.7, 0.2], false, Mean));
        }
        for _ in 0..2 {
            voters.push(HonestVoter::new(vec![0.8, 0.6, 0.1], false, Mean));
        }
        voters
    }

    /*
    Profile produced:
    24: A1 > A2 > B1 > B2 > B3
    24: A2 > A1 > B1 > B2 > B3
    20: B1 > B2 > B3 > A1 > A2
    20: B2 > B3 > B1 > A2 > A1
    12: B3 > B1 > B2 > A1 > A2
     */
    fn irv_differs() -> Vec<HonestVoter> {
        let mut voters = Vec::new();
        (0..24).for_each(|_| {
            voters.push(HonestVoter::new(vec![1.0, 0.9, 0.5, 0.4, 0.3], false, Mean));
            voters.push(HonestVoter::new(vec![0.9, 1.0, 0.5, 0.4, 0.3], false, Mean));
        });
        (0..20).for_each(|_| {
            voters.push(HonestVoter::new(vec![0.2, 0.3, 1.0, 0.9, 0.7], false, Mean));
            voters.push(HonestVoter::new(vec![0.3, 0.2, 0.7, 1.0, 0.9], false, Mean));
        });
        (0..12).for_each(|_| {
            voters.push(HonestVoter::new(vec![0.3, 0.2, 0.9, 0.7, 1.0], false, Mean));
        });
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

    #[test]
    fn test_irv() {
        assert_ne!(
            ElectionMethods::irv(&mut irv_differs(), 5, usize::cmp)[0],
            ElectionMethods::fptp_runoff(&mut irv_differs(), 5, usize::cmp)[0]
        )
    }

    // Test invoke_all function
    // doesn't work with star methods atm because i need a tiebreaker that actually doesn't just
    // return Ordering::Equal
    // #[test]
    // fn test_all() {
    //     ElectionMethods::invoke_all_enum_ordinal(&mut runoff_differs(), 3, usize::cmp, |e, _v| {
    //         println!("Called with ordinal method {}", <&str>::from(e))
    //     });
    //     ElectionMethods::invoke_all_enum_cardinal(&mut runoff_differs(), 3, usize::cmp, |e, _v| {
    //         println!("Called with cardinal method {}", <&str>::from(e))
    //     });
    // }
}
