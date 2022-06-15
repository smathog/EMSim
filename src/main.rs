use crate::election::ElectionMethods;

mod election;
mod utility_generators;

use utility_generators::foo;

fn main() {
    println!("Number of ordinal methods currently implemented: {}", ElectionMethods::METHOD_COUNT_ordinal);
    ElectionMethods::METHOD_LIST_ordinal.iter()
        .enumerate()
        .for_each(|(i, s)| println!("Election Method (Ordinal) {}: {}", i, s));
    println!("Number of cardinal methods currently implemented: {}", ElectionMethods::METHOD_COUNT_cardinal);
    ElectionMethods::METHOD_LIST_cardinal.iter()
        .enumerate()
        .for_each(|(i, s)| println!("Election Method (Cardinal) {}: {}", i, s));
    foo();
}
