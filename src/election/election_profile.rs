//! An ElectionProfile represents an entire election as a system; voters, candidates, and
//! various statistics about outcomes.

use crate::election::voters::Voter;

/// Core ElectionProfile struct
pub struct ElectionProfile<T>
where
    T: Voter, {
    voters: Vec<T>,
}

/// Separate type for indexing candidates
#[derive(Debug, Eq, PartialEq)]
pub struct CandidateID(pub(crate) usize);
