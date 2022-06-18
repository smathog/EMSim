use crate::election::ElectionMethods;
use rand::thread_rng;
use rand_distr::Beta;

mod election;
mod utility_generators;
mod metrics;

fn main() {
    println!("Number of ordinal methods currently implemented: {}", ElectionMethods::METHOD_COUNT_ordinal);
    ElectionMethods::METHOD_LIST_ordinal.iter()
        .enumerate()
        .for_each(|(i, s)| println!("Election Method (Ordinal) {}: {}", i, s));
    println!("Number of cardinal methods currently implemented: {}", ElectionMethods::METHOD_COUNT_cardinal);
    ElectionMethods::METHOD_LIST_cardinal.iter()
        .enumerate()
        .for_each(|(i, s)| println!("Election Method (Cardinal) {}: {}", i, s));
    let mut rng = thread_rng();
    println!("UNIFORM UTILITY VECTORS");
    for _ in 0..10 {
        println!("{:?}", utility_generators::uniform_utilities(&mut rng, 5))
    }
    println!("BETA a = b = 2");
    let mut beta = Beta::new(2.0, 2.0).unwrap();
    for _ in 0..10 {
        println!("{:?}", utility_generators::beta_utilities(beta, &mut rng, 5))
    }

    println!("BETA a = b = .5");
    beta = Beta::new(0.5, 0.5).unwrap();
    for _ in 0..10 {
        println!("{:?}", utility_generators::beta_utilities(beta, &mut rng, 5))
    }
}
