mod election;

use invoker_macro::show_streams;
use invoker_macro::count_statements;

#[count_statements]
fn main() {
    println!("Hello, world!");
    println!("I eat cake!");
    {}
}
