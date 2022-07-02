//! An ElectionProfile represents an entire election as a system; voters, candidates, and
//! various statistics about outcomes.

use crate::election::voters::Voter;
use std::cmp::Ordering;

/// Core ElectionProfile struct. Note that instead of the voters vec containing the Voters enum type
/// we are using for static polymorphism, we instead use the generic parameter T: Voter. This lets
/// us do things like creating ElectionProfile<HonestVoter, F> which is less memory intensive when
/// we do not need multiple types of voters present. Since the Voters enum also implements Voter,
/// it satisfies this trait bound and thus we can use ElectionProfile<Voters, F> in the case we
/// wish to mix different kinds of voters.
pub struct ElectionProfile<T, F>
where
    T: Voter,
    F: Fn(&usize, &usize) -> Ordering + Copy,
{
    voters: Vec<T>,
    candidates: Vec<CandidateID>,
    tie_breaker: F,
}

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

/// Separate type for indexing candidates
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct CandidateID(pub(crate) usize);
