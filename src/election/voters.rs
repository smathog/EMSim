//! This module contains the implementations for how voters work.
//! Each type of voter implements the Voter trait.
//! Additionally, to avoid dynamic dispatch, the enum Voters is used with enum_dispatch
//! to allow things like Vecs of multiple kinds of Voter.

use enum_dispatch::enum_dispatch;
use crate::election::election_profile::CandidateID;

/// Trait to define a voter
/// Voters can cast either ordinal ballots (ranked, i.e. A>B>C>D) or cardinal ballots
/// i.e. A:10 B:8 C:5 D:0
#[enum_dispatch]
trait Voter {
    /// A voter casts an ordinal ballot by returning a sorted (descending) Vec
    /// of preferences by CandidateID
    fn cast_ordinal_ballot(&self) -> Vec<CandidateID>;

    /// A voter casts a cardinal ballot by returning a Vec of ratings of candidates
    /// The score of CandidateID.0 is cast_cardinal_ballot()[CandidateID.0]
    fn cast_cardinal_ballot(&self) -> Vec<usize>;
}

/// Enum for static polymorphism (enum dispatch) of all voters
#[enum_dispatch(Voter)]
enum Voters {
    HonestVoter,
}

struct HonestVoter{}

impl Voter for HonestVoter {
    fn cast_ordinal_ballot(&self) -> Vec<CandidateID> {
        todo!()
    }

    fn cast_cardinal_ballot(&self) -> Vec<usize> {
        todo!()
    }
}

