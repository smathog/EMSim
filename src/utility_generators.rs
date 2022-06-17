//! mod to hold the various utility generators. Most are for generating the utility vecs for voters
//! based on candidates

use rand::Rng;
use rand_distr::Beta;

/// Given a number of candidates n, generate a utility vector of n elements where the utilities are
/// chosen at random from the uniform distribution over [0, 1].
pub fn uniform_utilities<T: Rng>(rng: &mut T, n: usize) -> Vec<f64> {
    (0..n)
        .map(|_| rng.gen::<f64>())
        .collect()
}

/// Given a number of candidates n, generate a utility vector of n elements where the utilities are
/// chosen at random from a Beta distribution over [0, 1]. The caller determines the alpha and beta
/// parameters on the distribution.
pub fn beta_utilities<T: Rng>(beta: Beta<f64>, rng: &mut T, n: usize) -> Vec<f64> {
    (0..n)
        .map(|_| rng.sample(beta))
        .collect()
}