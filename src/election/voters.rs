//! This module contains the implementations for how voters work.
//! Each type of voter implements the Voter trait.
//! Additionally, to avoid dynamic dispatch, the enum Voters is used with enum_dispatch
//! to allow things like Vecs of multiple kinds of Voter.

use crate::election::election_profile::CandidateID;
use enum_dispatch::enum_dispatch;

/// Trait to define a voter
/// Voters can cast either ordinal ballots (ranked, i.e. A>B>C>D) or cardinal ballots
/// i.e. A:10 B:8 C:5 D:0
#[enum_dispatch]
pub trait Voter {
    /// A voter casts an ordinal ballot by returning a sorted (descending) Vec
    /// of preferences by CandidateID
    fn cast_ordinal_ballot(&self) -> Vec<CandidateID>;

    /// A voter casts a cardinal ballot by returning a Vec of ratings of candidates
    /// The score of CandidateID.0 is cast_cardinal_ballot(range)[CandidateID.0].
    /// range indicates the possible valid ratings range: [0, range].
    fn cast_cardinal_ballot(&self, range: usize) -> Vec<usize>;
}

/// Enum for static polymorphism (enum dispatch) of all voters
#[enum_dispatch(Voter)]
enum Voters {
    HonestVoter,
}

/// An HonestVoter represents a voter who casts their ballot directly off of their utility
/// assessment of the candidates; that is, non-strategically.
struct HonestVoter {
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
}

impl Voter for HonestVoter {
    /// Sorts the candidates in order of descending honest utility according to the HonestVoter
    fn cast_ordinal_ballot(&self) -> Vec<CandidateID> {
        let mut candidates: Vec<_> = (0..(self.utilities.len()))
            .map(|i| CandidateID(i))
            .collect();
        candidates.sort_unstable_by(|&CandidateID(a), &CandidateID(b)|
            b.partial_cmp(&a).unwrap());
        candidates
    }

    /// Returns a rating in [0, range] for each candidate based on the HonestVoter's honest utility,
    /// possibly scaling to min/max the ballot.
    fn cast_cardinal_ballot(&self, range: usize) -> Vec<usize> {
        // Adjust utilities to convert to ratings if should be scaled
        let adjusted_utilities = if self.scales {
            let max = self.utilities.iter().max().copied().unwrap();
            let min = self.utilities.iter().min.copied().unwrap();
            self.utilities.iter()
                .map(|&f| if max != min {(f - min) / (max - min)} else {max})
                .collect()
        } else {
            self.utilities.clone()
        };

        // Convert f64 utilities to usize ratings in range [0, range]
        adjusted_utilities.into_iter()
            .map(|f| (range as f64 * f).round() as usize)
            .collect()
    }
}
