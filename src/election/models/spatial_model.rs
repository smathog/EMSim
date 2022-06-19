//! Mod for the spatial model of voting. Contains generators and related functions dedicated to
//! building spatial models.

use rand::distributions::Distribution;
use rand::Rng;

/// Generate a n-dimensional spatial distribution of the specified number of voters and candidates
/// from the given distributions and rng. Note that for the moment, this generic specification
/// requires that the underlying type of distribution used be the same (i.e. all normal
/// distributions). The two references to slices of Vecs of distribution are to be used in the
/// following manner: candidate_distributions\[0] contains a reference to the n-dimensional vec
/// of distributions that are to be used to generate the k candidates_per_distribution\[0]
/// candidates. The return type is a pair of Vec<Vec<f64>>, the first being the locations of the
/// candidates, the second being the locations of the voters.
fn generate_distances<R: Rng, D: Distribution<f64> + Copy>(
    rng: &mut R,
    candidate_distributions: &[Vec<D>],
    candidates_per_distribution: &[usize],
    voter_distributions: &[Vec<D>],
    voters_per_distribution: &[usize],
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let n_dimensions = candidate_distributions[0].len();
    let num_candidates = candidates_per_distribution.iter().copied().sum();
    let num_voters = voters_per_distribution.iter().copied().sum();
    let mut candidates = Vec::with_capacity(num_candidates);
    let mut voters = Vec::with_capacity(num_voters);

    fn build_locations<R: Rng, D: Distribution<f64> + Copy>(
        n_dimensions: usize,
        rng: &mut R,
        distribution_list: &[Vec<D>],
        count: &[usize],
        output: &mut Vec<Vec<f64>>,
    ) {
        for (&n, distributions) in count.into_iter().zip(distribution_list.into_iter()) {
            for _ in 0..n {
                let mut location = Vec::with_capacity(n_dimensions);
                for &distribution in distributions {
                    location.push(rng.sample(distribution));
                }
                output.push(location);
            }
        }
    }

    // Build candidate locations:
    build_locations(
        n_dimensions,
        rng,
        candidate_distributions,
        candidates_per_distribution,
        &mut candidates,
    );

    // Build voter locations:
    build_locations(
        n_dimensions,
        rng,
        voter_distributions,
        voters_per_distribution,
        &mut voters,
    );

    (candidates, voters)
}
