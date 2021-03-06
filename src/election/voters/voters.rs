//! This module contains the implementations for how voters work.
//! Each type of voter implements the Voter trait.
//! Additionally, to avoid dynamic dispatch, the enum Voters is used with enum_dispatch
//! to allow things like Vecs of multiple kinds of Voter.

use crate::election::election_profile::CandidateID;
use crate::election::election_methods::CardinalEnum;
use crate::election::election_methods::OrdinalEnum;
use enum_dispatch::enum_dispatch;
use std::cmp::Ordering;
use voters::honest_voter::HonestVoter;
use voters::real_ordinal_voter::RealOrdinalVoter;
use voters::real_cardinal_voter::RealCardinalVoter;
use crate::election::voters;

/// Trait to define a voter
/// Voters can cast either ordinal ballots (ranked, i.e. A>B>C>D) or cardinal ballots
/// i.e. A:10 B:8 C:5 D:0
#[enum_dispatch]
pub trait Voter {
    /// A voter casts an ordinal ballot by returning a sorted (descending) Vec
    /// of preferences by CandidateID. Note that this style of ordinal ballot does
    /// not permit equalities.
    fn cast_ordinal_ballot(&mut self, method: OrdinalEnum) -> &Vec<CandidateID>;

    /// A voter casts an ordinal ballot by returning a sorted (descending) ballot.
    /// This style of ballot does permit ranked equalities, so a ballot A > B = C > D would be
    /// a vec of the form {{A}, {B, C}, {D}}
    fn cast_ordinal_equal_ballot(&mut self, method_name: &str) -> &Vec<Vec<CandidateID>>;

    /// A voter casts a cardinal ballot by returning a Vec of ratings of candidates
    /// The score of CandidateID.0 is cast_cardinal_ballot(range)[CandidateID.0].
    /// range indicates the possible valid ratings range: [0, range].
    fn cast_cardinal_ballot(&mut self, range: usize, method: CardinalEnum) -> &Vec<usize>;

    /// A voter casts an approval ballot by returning a Vec of those candidates of which they
    /// approve.
    fn cast_approval_ballot(&mut self, method: CardinalEnum) -> &Vec<CandidateID>;

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
    RealOrdinalVoter,
    RealCardinalVoter,
}

/// Helper enum to indicate where a voter would honestly put their Approval threshold.
/// Voters will cast an approval ballot in support of any candidate above or equal to the threshold.
/// Note that regardless of threshold, the voter will always approve of at least one candidate
/// (their favorite).
pub enum ApprovalThresholdBehavior {
    /// Set by closure for custom behavior
    Function(Box<dyn Fn(&Vec<f64>) -> f64>),
    /// Set as greater than or equal to the mean of utilities
    Mean,
    /// Set threshold directly
    Preset(f64),
}

/// Unit tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::election::voters::ApprovalThresholdBehavior::Mean;
    use crate::election::voters::HonestVoter;

    /// Make sure that enum_dispatch is working for what we need it for; we can make a Vec holding
    /// multiple types of voters
    #[test]
    pub fn can_make_vec() {
        let mut v = Vec::new();
        let first = HonestVoter::new(vec![0.1, 0.2], true,Mean);
        let second = RealOrdinalVoter::new(vec![CandidateID(0), CandidateID(1)]);
        v.push(Voters::from(first));
        v.push(Voters::from(second));
        for voter in v.iter_mut() {
            println!("{:?}", voter.cast_ordinal_ballot(OrdinalEnum::plurality));
        }
    }
}