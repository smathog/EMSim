//! This module contains the implementations for how voters work.
//! Each type of voter implements the Voter trait.
//! Additionally, to avoid dynamic dispatch, the enum Voters is used with enum_dispatch
//! to allow things like Vecs of multiple kinds of Voter.

use crate::election::election_profile::CandidateID;
use enum_dispatch::enum_dispatch;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Trait to define a voter
/// Voters can cast either ordinal ballots (ranked, i.e. A>B>C>D) or cardinal ballots
/// i.e. A:10 B:8 C:5 D:0
#[enum_dispatch]
pub trait Voter {
    /// A voter casts an ordinal ballot by returning a sorted (descending) Vec
    /// of preferences by CandidateID. Note that this style of ordinal ballot does
    /// not permit equalities.
    fn cast_ordinal_ballot(&mut self, method_name: &str) -> &Vec<CandidateID>;

    /// A voter casts an ordinal ballot by returning a sorted (descending) ballot.
    /// This style of ballot does permit ranked equalities, so a ballot A > B = C > D would be
    /// a vec of the form {{A}, {B, C}, {D}}
    fn cast_ordinal_equal_ballot(&mut self, method_name: &str) -> &Vec<Vec<CandidateID>>;

    /// A voter casts a cardinal ballot by returning a Vec of ratings of candidates
    /// The score of CandidateID.0 is cast_cardinal_ballot(range)[CandidateID.0].
    /// range indicates the possible valid ratings range: [0, range].
    fn cast_cardinal_ballot(&mut self, range: usize, method_name: &str) -> &Vec<usize>;

    /// Given two candidates (first, second), return whether votes likes first more, less, or equal
    /// to second.
    fn honest_preference(&self, first: CandidateID, second: CandidateID) -> Ordering;

    /// Return a reference to the voter's utility vec
    fn utilities(&self) -> &Vec<f64>;

    /// Return the voter's honest utility assessment of candidate id
    fn candidate_utility(&self, _: CandidateID) -> f64;
}

/// Enum for static polymorphism (enum dispatch) of all voters
#[enum_dispatch(Voter)]
pub enum Voters {
    HonestVoter,
}

/// An HonestVoter represents a voter who casts their ballot directly off of their utility
/// assessment of the candidates; that is, non-strategically.
pub struct HonestVoter {
    /// A vector containing this voter's assessment of the utility the candidates provide them
    /// as a float in the range [0, 1].
    /// That is, utilities[0] is the utility this voter ascribes CandidateID(0) for the election
    /// they are a part of.
    utilities: Vec<f64>,

    /// Indicates whether this voter will scale a cardinal ballot.
    /// That is, if the voter's utilities are {.01, 0.0, 0.2}, with scales = true the voter
    /// would cast A:5 B:0 C:10 with range=10; if scales = false, the same voter will cast
    /// A:1 B:0 C:2.
    /// Note that this is *not* considered strategic voting for the purposes of this simulator.
    scales: bool,

    /// Since an HonestVoter always votes honestly, their ordinal vote should never change.
    /// Thus extra calculation can be avoided by caching
    cached_ordinal_vote: Vec<CandidateID>,

    /// Since an HonestVoter always votes honestly, their ordinal equal vote should never change.
    /// Thus extra calculation can be avoided by caching
    cached_ordinal_equal_vote: Vec<Vec<CandidateID>>,

    /// Since an HonestVoter always votes honestly, their scaled utilities should never change.
    cached_scaled_utilities: Option<Vec<f64>>,

    /// Since an HonestVoter always votes honestly, their given vote for a given rating should
    /// never change.
    cached_cardinal_ballots: HashMap<usize, Vec<usize>>,
}

impl HonestVoter {
    pub fn new(utilities: Vec<f64>, scales: bool) -> Self {
        // Precompute ordinal ballot
        let mut candidates: Vec<_> = (0..(utilities.len())).map(|i| CandidateID(i)).collect();
        candidates.sort_unstable_by(|&CandidateID(a), &CandidateID(b)| {
            utilities[b].partial_cmp(&utilities[a]).unwrap()
        });

        // Precompute ordinal-equal ballot
        let candidates_with_equality = candidates
            .iter()
            .fold((Vec::new(), f64::NAN), |(mut vec, mut val), &candidate| {
                let CandidateID(id) = candidate;
                if utilities[id] != val {
                    vec.push(vec![candidate]);
                    val = utilities[id];
                } else {
                    vec.last_mut().unwrap().push(candidate);
                }
                (vec, val)
            })
            .0;


        if scales {
            let scaled_utilities = scale_utilities_linearly(&utilities);
            Self {
                utilities,
                scales,
                cached_ordinal_vote: candidates,
                cached_ordinal_equal_vote: candidates_with_equality,
                cached_scaled_utilities: Some(scaled_utilities),
                cached_cardinal_ballots: HashMap::new(),
            }
        } else {
            Self {
                utilities,
                scales,
                cached_ordinal_vote: candidates,
                cached_ordinal_equal_vote: candidates_with_equality,
                cached_scaled_utilities: None,
                cached_cardinal_ballots: HashMap::new(),
            }
        }
    }

    fn calculate_cardinal_ballot(&mut self, range: usize) {
        // Check if already cached; if it is, just return
        if self.cached_cardinal_ballots.contains_key(&range) {
            return;
        }

        // Need to calculate the ballot and cache it
        // Get reference to the utility vector we're using
        let adjusted_utilities = if let Some(ref utils) = self.cached_scaled_utilities {
            utils
        } else {
            &self.utilities
        };

        // Convert f64 utilities to usize ratings in range [0, range]
        let ballot = adjusted_utilities
            .into_iter()
            .map(|&f| (range as f64 * f).round() as usize)
            .collect();

        // Cache the ballot vec
        self.cached_cardinal_ballots.insert(range, ballot);
    }
}

impl Voter for HonestVoter {
    /// Sorts the candidates in order of descending honest utility according to the HonestVoter
    /// Returns a reference to a precomputed ordinal ballot
    fn cast_ordinal_ballot(&mut self, method_name: &str) -> &Vec<CandidateID> {
        &self.cached_ordinal_vote
    }

    /// Returns a reference to a precomputed ordinal-equal ballot
    fn cast_ordinal_equal_ballot(&mut self, method_name: &str) -> &Vec<Vec<CandidateID>> {
        &self.cached_ordinal_equal_vote
    }

    /// Returns a rating in [0, range] for each candidate based on the HonestVoter's honest utility,
    /// possibly scaling to min/max the ballot.
    fn cast_cardinal_ballot(&mut self, range: usize, method_name: &str) -> &Vec<usize> {
        // Calculate the ballot, if not already cached
        self.calculate_cardinal_ballot(range);
        // Return reference to the ballot vec
        self.cached_cardinal_ballots.get(&range).unwrap()
    }

    fn honest_preference(&self, first: CandidateID, second: CandidateID) -> Ordering {
        if self.utilities[first.0] > self.utilities[second.0] {
            Ordering::Greater
        } else if self.utilities[first.0] < self.utilities[second.0] {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }

    fn utilities(&self) -> &Vec<f64> {
        &self.utilities
    }

    fn candidate_utility(&self, CandidateID(id): CandidateID) -> f64 {
        self.utilities[id]
    }
}

/// Helper function to scale utilities linearly so the min is 0 and max is 1, provided min != max
fn scale_utilities_linearly(utilities: &Vec<f64>) -> Vec<f64> {
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
            if max != min {
                (f - min) / (max - min)
            } else {
                max
            }
        })
        .collect()
}

/// Unit tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for HonestVoter
    #[test]
    fn ordinal_order_correct() {
        let mut voter = HonestVoter::new(vec![0.3, 0.5, 0.1], false);
        assert_eq!(
            voter.cast_ordinal_ballot("test"),
            &vec![CandidateID(1), CandidateID(0), CandidateID(2)]
        );
    }

    #[test]
    fn ordinal_equal_ballot_correct() {
        let mut voter = HonestVoter::new(vec![0.3, 0.5, 0.5, 0.1], false);
        assert_eq!(
            voter.cast_ordinal_equal_ballot("test"),
            &vec![vec![CandidateID(1), CandidateID(2)], vec![CandidateID(0)], vec![CandidateID(3)]]
        );
    }

    #[test]
    fn scales_correct() {
        let mut voter = HonestVoter::new(vec![0.3, 0.5, 0.1], true);
        assert_eq!(voter.cast_cardinal_ballot(10, "test"), &vec![5, 10, 0]);
    }

    #[test]
    fn no_scales_correct() {
        let mut voter = HonestVoter::new(vec![0.3, 0.5, 0.1], false);
        assert_eq!(voter.cast_cardinal_ballot(10, "test"), &vec![3, 5, 1]);
    }
}
