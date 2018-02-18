extern crate petgraph;

use petgraph::graph::Graph;

fn main() {
    let s: Vec<u64> = squares().take(usize::pow(2, 20)).collect();
    println!("{:?}", s.len());
}

fn integers() -> std::ops::Range<u64> {
    1u64..u64::max_value()
}

fn squares() -> std::iter::Map<std::ops::Range<u64>, fn(u64) -> u64> {
    integers().map(|x| x*x)
}
