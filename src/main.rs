extern crate petgraph;

fn main() {
    let g = square_sum_graph(100);
    //println!("{:?}", g);
}

fn integers() -> std::ops::Range<usize> {
    1..usize::max_value()
}

fn squares() -> std::iter::Map<std::ops::Range<usize>, fn(usize) -> usize> {
    integers().map(|x| x*x)
}

fn square_sum_graph(n: usize) -> petgraph::Graph<usize, u8, petgraph::Undirected, usize> {
    let s: Vec<usize> = squares().take_while(|&x| x <= (n*2) - 1).collect();
    let mut deps = petgraph::Graph::default();  // TODO use with_capacity
    for i in integers().take(n) {
        deps.add_node(i);
        for j in integers().take(i) {
            if s.contains(&(i + j)) {
                let i_index = petgraph::graph::node_index(i-1);
                let j_index = petgraph::graph::node_index(j-1);
                deps.add_edge(i_index, j_index, 1);
            }
        }
    }
    deps
}
