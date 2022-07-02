use crate::election::election_profile::CandidateID;
use crate::election::voters::*;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::election::election_methods::CardinalEnum;
use crate::election::election_methods::OrdinalEnum;

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

    /// This voter's ApprovalThresholdBehavior
    threshold_behavior: ApprovalThresholdBehavior,

    /// Since an HonestVoter always votes honestly, their approval ballot should never change.
    cached_approval_ballot: Vec<CandidateID>,

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
    pub fn new(
        utilities: Vec<f64>,
        scales: bool,
        threshold_behavior: ApprovalThresholdBehavior,
    ) -> Self {
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

        // Precompute approval ballot
        let cached_approval_ballot = match &threshold_behavior {
            ApprovalThresholdBehavior::Function(f) => {
                let bound = f(&utilities);
                generate_approval_ballot(&utilities, bound)
            }
            ApprovalThresholdBehavior::Mean => {
                let mean = utilities.iter().copied().sum::<f64>() / (utilities.len() as f64);
                (0..(utilities.len()))
                    .filter(|&i| utilities[i] >= mean)
                    .map(|i| CandidateID(i))
                    .collect()
            }
            ApprovalThresholdBehavior::Preset(bound) => {
                generate_approval_ballot(&utilities, *bound)
            }
        };

        if scales {
            let scaled_utilities = scale_utilities_linearly(&utilities);
            Self {
                utilities,
                scales,
                threshold_behavior,
                cached_approval_ballot,
                cached_ordinal_vote: candidates,
                cached_ordinal_equal_vote: candidates_with_equality,
                cached_scaled_utilities: Some(scaled_utilities),
                cached_cardinal_ballots: HashMap::new(),
            }
        } else {
            Self {
                utilities,
                scales,
                threshold_behavior,
                cached_approval_ballot,
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
    fn cast_ordinal_ballot(&mut self, method: OrdinalEnum) -> &Vec<CandidateID> {
        &self.cached_ordinal_vote
    }

    /// Returns a reference to a precomputed ordinal-equal ballot
    fn cast_ordinal_equal_ballot(&mut self, method_name: &str) -> &Vec<Vec<CandidateID>> {
        &self.cached_ordinal_equal_vote
    }

    /// Returns a rating in [0, range] for each candidate based on the HonestVoter's honest utility,
    /// possibly scaling to min/max the ballot.
    fn cast_cardinal_ballot(&mut self, range: usize, method: CardinalEnum) -> &Vec<usize> {
        // Calculate the ballot, if not already cached
        self.calculate_cardinal_ballot(range);
        // Return reference to the ballot vec
        self.cached_cardinal_ballots.get(&range).unwrap()
    }

    /// Returns a reference to the precomputed approval ballot
    fn cast_approval_ballot(&mut self, method: CardinalEnum) -> &Vec<CandidateID> {
        &self.cached_approval_ballot
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

/// Unit tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::election::voters::ApprovalThresholdBehavior::Mean;

    // Unit tests for HonestVoter
    #[test]
    fn ordinal_order_correct() {
        let mut voter = HonestVoter::new(vec![0.3, 0.5, 0.1], false, Mean);
        assert_eq!(
            voter.cast_ordinal_ballot(OrdinalEnum::plurality),
            &vec![CandidateID(1), CandidateID(0), CandidateID(2)]
        );
    }

    #[test]
    fn ordinal_equal_ballot_correct() {
        let mut voter = HonestVoter::new(vec![0.3, 0.5, 0.5, 0.1], false, Mean);
        assert_eq!(
            voter.cast_ordinal_equal_ballot("test"),
            &vec![
                vec![CandidateID(1), CandidateID(2)],
                vec![CandidateID(0)],
                vec![CandidateID(3)]
            ]
        );
    }

    #[test]
    fn scales_correct() {
        let mut voter = HonestVoter::new(vec![0.3, 0.5, 0.1], true, Mean);
        assert_eq!(voter.cast_cardinal_ballot(10, CardinalEnum::score_10), &vec![5, 10, 0]);
    }

    #[test]
    fn no_scales_correct() {
        let mut voter = HonestVoter::new(vec![0.3, 0.5, 0.1], false, Mean);
        assert_eq!(voter.cast_cardinal_ballot(10, CardinalEnum::score_10), &vec![3, 5, 1]);
    }
}
