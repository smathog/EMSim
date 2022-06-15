//! An ElectionProfile represents an entire election as a system; voters, candidates, and
//! various statistics about outcomes.

use crate::election::voters::Voter;
use std::cmp::Ordering;

/// Core ElectionProfile struct
pub struct ElectionProfile<T, F>
where
    T: Voter,
    F: Fn(&usize, &usize) -> Ordering + Copy,
{
    voters: Vec<T>,
    candidates: Vec<CandidateID>,
    tie_breaker: F,
}

/// Separate type for indexing candidates
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct CandidateID(pub(crate) usize);

impl<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy> ElectionProfile<T, F> {
    /// Get a mut reference to the vec of voters
    pub fn get_voters(&mut self) -> &mut Vec<T> {
        &mut self.voters
    }

    /// Get the number of voters this ElectionProfile has
    pub fn num_voters(&self) -> usize {
        self.voters.len()
    }

    /// Get a reference to the vec of candidates
    pub fn get_candidates(&self) -> &Vec<CandidateID> {
        &self.candidates
    }

    /// Get the number of candidates this ElectionProfile has
    pub fn num_candidates(&self) -> usize {
        self.candidates.len()
    }

    /// Return a copy of the tiebreaker this ElectionProfile uses
    pub fn get_tie_breaker(&self) -> F {
        self.tie_breaker
    }
}
