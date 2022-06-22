//! A mod to hold the struct and implementation to represent real-world ordinal ballots.

use crate::election::election_profile::CandidateID;
use crate::election::voters::Voter;
use std::cmp::Ordering;
use crate::election::election_methods::ElectionMethods_invoke_impl_enum_cardinal as CardinalEnum;
use crate::election::election_methods::ElectionMethods_invoke_impl_enum_ordinal as OrdinalEnum;

/// A struct that represents an actual cast ordinal ballot
#[derive(Debug)]
pub struct RealOrdinalVoter {
    ordinal_ballot: Vec<CandidateID>
}

impl RealOrdinalVoter {
    const WARNING_STRING: &'static str =
        "RealOrdinalVoter does not contain cardinal or utility information!";

    const WARNING_STRING_EQUALITY: &'static str =
        "RealOrdinalVoter does not permit equalities on a ballot!";

    pub fn new(ballot: Vec<CandidateID>) -> Self {
        Self {
            ordinal_ballot: ballot,
        }
    }
}

impl Voter for RealOrdinalVoter {
    fn cast_ordinal_ballot(&mut self, method: OrdinalEnum) -> &Vec<CandidateID> {
        &self.ordinal_ballot
    }

    fn cast_ordinal_equal_ballot(&mut self, method_name: &str) -> &Vec<Vec<CandidateID>> {
        panic!("{}", RealOrdinalVoter::WARNING_STRING_EQUALITY)
    }

    fn cast_cardinal_ballot(&mut self, range: usize, method: CardinalEnum) -> &Vec<usize> {
        panic!("{}", RealOrdinalVoter::WARNING_STRING)
    }

    fn cast_approval_ballot(&mut self, method: CardinalEnum) -> &Vec<CandidateID> {
        panic!("{}", RealOrdinalVoter::WARNING_STRING)
    }

    fn honest_preference(&self, first: CandidateID, second: CandidateID) -> Ordering {
        panic!("{}", RealOrdinalVoter::WARNING_STRING)
    }

    fn utilities(&self) -> &Vec<f64> {
        panic!("{}", RealOrdinalVoter::WARNING_STRING)
    }

    fn candidate_utility(&self, _: CandidateID) -> f64 {
        panic!("{}", RealOrdinalVoter::WARNING_STRING)
    }
}
