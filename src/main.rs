use crate::election::ElectionMethods;

mod election;

fn main() {
    println!("Number of methods currently implemented: {}", ElectionMethods::METHOD_COUNT);
    ElectionMethods::METHOD_LIST.iter()
        .enumerate()
        .for_each(|(i, s)| println!("Election Method {}: {}", i, s));
}
