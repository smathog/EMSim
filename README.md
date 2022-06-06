# Description

This is my implemenetation of an election methods simulator, written in Rust. My goal is to create an application which can replicate the capabilities of previous election method simulators, and hopefully extend upon them in certain areas (namely the simulation of strategic behavior).

For now, this is just a preliminary README.

## Implementation Details

The core idea behind this simulator is that we are interested in what outcomes are produced for various election methods for a given election profile (a set of voters and a set of candidates, combined with information such as what the voters think of the candidates, etc). As such, the simulator is designed around generating different election profiles using various proccesses and then evaluating the cumulative outcomes for given election methods. 

### Voters

At the core of each election profile are the voters. These are the dynamic component of each election profile; it is voter behavior that drives different outcomes given that the same election method is applied to the profile. A voter with a particular strategy will often cast a completely different vote than they would if they were voting without any strategy in mind despite having identical utility evaluations of individual candidates. With this in mind, strategic (and other) behavioral differences are modeled using different structs that all implement the Voter trait.

In addition, we want to be able to model elections with different kinds of voters. For example, we might imagine running a simulation where half the voters in a given election profile are completely honest, and the other half employ a particular strategy. This means each election profile should be able to contain a list or set of voters of different fundamental types. The obvious answer to this is to employ dynamic dispatch, where each election profile might contain some Vec<Box<dyn Voter>>. However, I have instead opted to use static dispatch using the [enum_dispatch](https://crates.io/crates/enum_dispatch/0.3.8) crate, which I use to wrap all types of struct implementing Voter in an enum Voters (which also implements the trait Voter) so that instead election profiles might contain Vec<Voters>. This approach means longer compilation and more code generated, but should typically be considerably faster than relying on dynamic dispatch. Note that this approach is the Rust equivalent to a C++ std::variant and std::visit pattern. 

### Election Methods

Election methods are modeled as generic associated functions. They are generic over T: Voter, as well as over a type F representing a tie-breaker function. They are associated with the marker type ElectionMethods, which just serves essentially to provide a namespace and the ability to enclose all the election method functions within an impl block. All election methods should have the following signature (except obviously having a different name):

```Rust
    pub fn method_name<T: Voter, F: Fn(&usize, &usize) -> Ordering + Copy>(
        voters: &mut Vec<T>,
        num_candidates: usize,
        tie_breaker: F,
    ) -> Vec<CandidateID>
```

Note that voters is &mut because each election method should ask each voter to cast a ballot, and the act of casting a ballot may fundamentally alter a voter's underlying data in the case that voter is strategic; for example, the HonestVoter type will cache a rated ballot if the scale hasn't been seen before. 

The return type is Vec<CandidateID> to represent a ranking (decreasing) of the candidates according to that method. The winner is obviously the first candidate in the vec; to prevent ties, the tie_breaker will be invoked as needed by each method.

For convenience, all election methods are written in the same impl block, which is tagged with the macro #[invoke_all]. This custom procedural macro inserts additional code into the impl block, creating associated constants for the total number of methods available, an array of available methods by their identifiers, and a function to invoke election methods one after another and pass the result to a FnMut closure for consumption. 

All election methods should be written to be truncation safe. If a voter does not rank all candidates (ratings just assume unrated candidates are given minimum score, so this is less of an issue), an election method should handle this appropriately. 