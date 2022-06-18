//! mod containing metrics for spatial-based distance calculations. For the moment, only holds
//! a generic L_p metric function.


/// The L_p metric, where p is passed in a const generic argument. For those unfamiliar, p = 1 is
/// taxicab distance and p = 2 is the familiar Euclidean distance.
pub fn lp_metric<const P: usize>(location_1: &Vec<f64>, location_2: &Vec<f64>) -> f64 {
    location_1
        .into_iter()
        .zip(location_2.into_iter())
        .map(|(&x1, &x2)| (x1 - x2).abs().powi(P as i32))
        .sum::<f64>()
        .powf(1f64 / P as f64)
}