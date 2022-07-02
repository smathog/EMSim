//! A mod to hold the struct and implementation to represent real-world cardinal ballots of fixed
//! range.

use super::voters::Voter;
use crate::election::election_profile::CandidateID;
use crate::election::election_methods::OrdinalEnum;
use crate::election::election_methods::CardinalEnum;
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::cmp::Reverse;

/// A struct that represents a real cardinal ballot of fixed range (i.e. a real voter's 0-10 score
/// ballot).
#[derive(Debug)]
pub struct RealCardinalVoter {
    range: usize,
    cardinal_ballot: Vec<usize>,
    approval_ballot: Option<Vec<CandidateID>>,
    ordinal_equal_ballot: Vec<Vec<CandidateID>>,
    ordinal_ballot: Vec<CandidateID>,
}

impl RealCardinalVoter {
    /// Create a new RealCardinalVoter with a given ballot and range, using the specified tiebreaker
    /// function to break ties for the ordinal ranking of candidates.
    pub fn new<F: Fn(&usize, &usize) -> Ordering + Copy>(
        range: usize,
        ballot: Vec<usize>,
        tiebreaker: F,
    ) -> Self {
        // Build approval ballot from the cast cardinal ballot
        let approval_ballot = if range == 1 {
            Some(
                ballot
                    .iter()
                    .copied()
                    .enumerate()
                    .filter(|&(_, i)| i == 1)
                    .map(|(i, _)| CandidateID(i))
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };

        // Build ordinal ballot from the given cardinal ballot
        let num_candidates = ballot.len();
        let mut ordinal_ballot = (0..num_candidates)
            .map(|i| CandidateID(i))
            .collect::<Vec<_>>();
        ordinal_ballot.sort_unstable_by(|&CandidateID(i), &CandidateID(j)| {
            ballot[j].partial_cmp(&ballot[i])
                .unwrap()
                .then(tiebreaker(&i, &j))
        });

        // Build ordinal-equal ballot
        let mut map : HashMap<usize, Vec<CandidateID>> = HashMap::new();
        for (index, &score) in ballot.iter().enumerate() {
            let entry = map.entry(score).or_insert(Vec::new());
            entry.push(CandidateID(index));
        }
        let mut ordinal_equal_ballot = map
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<Vec<CandidateID>>>();
        ordinal_equal_ballot.sort_unstable_by_key(|v| Reverse(ballot[v[0].0]));


        Self {
            range,
            cardinal_ballot: ballot,
            approval_ballot,
            ordinal_equal_ballot,
            ordinal_ballot,
        }
    }

    const UTILITY_WARNING: &'static str = "A RealCardinalVoter does not contain raw \
    utility information!";

    const HONESTY_WARNING: &'static str = "A RealCardinalVoter represents a real ballot, \
    so honesty cannot be inferred!";

    const RANGE_WARNING: &'static str = "Invalid range of ballot ratings requested \
    from RealCardinalVoter";
}

impl Voter for RealCardinalVoter {
    fn cast_ordinal_ballot(&mut self, method: OrdinalEnum) -> &Vec<CandidateID> {
        &self.ordinal_ballot
    }

    fn cast_ordinal_equal_ballot(&mut self, method_name: &str) -> &Vec<Vec<CandidateID>> {
        &self.ordinal_equal_ballot
    }

    fn cast_cardinal_ballot(&mut self, range: usize, method: CardinalEnum) -> &Vec<usize> {
        if range == self.range {
            &self.cardinal_ballot
        } else {
            panic!("{}", RealCardinalVoter::RANGE_WARNING)
        }
    }

    fn cast_approval_ballot(&mut self, method: CardinalEnum) -> &Vec<CandidateID> {
        if let Some(ballot_ref) = self.approval_ballot.as_ref() {
            ballot_ref
        } else {
            panic!("{}", RealCardinalVoter::RANGE_WARNING)
        }
    }

    fn honest_preference(&self, first: CandidateID, second: CandidateID) -> Ordering {
        panic!("{}", RealCardinalVoter::HONESTY_WARNING)
    }

    fn utilities(&self) -> &Vec<f64> {
        panic!("{}", RealCardinalVoter::UTILITY_WARNING)
    }

    fn candidate_utility(&self, _: CandidateID) -> f64 {
        panic!("{}", RealCardinalVoter::UTILITY_WARNING)
    }
}
