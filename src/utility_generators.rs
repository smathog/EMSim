//! mod to hold the various utility generators. Most are for generating the utility vecs for voters
//! based on candidates

use rand::Rng;
use rand_distr::Beta;

/// Given a number of candidates n, generate a utility vector of n elements where the utilities are
/// chosen at random from the uniform distribution over [0, 1].
pub fn uniform_utilities<T: Rng>(rng: &mut T, n: usize) -> Vec<f64> {
    (0..n).map(|_| rng.gen::<f64>()).collect()
}

/// Given a number of candidates n, generate a utility vector of n elements where the utilities are
/// chosen at random from a Beta distribution over [0, 1]. The caller determines the alpha and beta
/// parameters on the distribution.
pub fn beta_utilities<T: Rng>(beta: Beta<f64>, rng: &mut T, n: usize) -> Vec<f64> {
    (0..n).map(|_| rng.sample(beta)).collect()
}

/// Given a list of n candidates location's in k-dimensional space and this voter's location in the
/// same k-dimensional space, calculate the voter's utility vector for each candidate based upon
/// the specified metric. In this case, we will define the assigned utility of a voter for candidate
/// i by 1 / (1 + metric(voter_location, candidate_location). Here, candidate_location[i] is the
/// location of CandidateID(i) in the election profile.
pub fn distance_utilities<Metric: Fn(&Vec<f64>, &Vec<f64>) -> f64>(
    candidate_locations: &Vec<Vec<f64>>,
    voter_location: &Vec<f64>,
    m: Metric,
) -> Vec<f64> {
    candidate_locations
        .into_iter()
        .map(|candidate_location| {
            // Clamp to deal with the odd floating point error
            (1f64 / (1f64 + m(candidate_location, voter_location))).clamp(0f64, 1f64)
        })
        .collect()
}
