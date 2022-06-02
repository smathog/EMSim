//! An ElectionProfile represents an entire election as a system; voters, candidates, and
//! various statistics about outcomes.

/// Core ElectionProfile struct
pub struct ElectionProfile {
}

/// Separate type for indexing candidates
pub struct CandidateID(usize);