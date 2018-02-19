extern crate petgraph;
extern crate rand;

use std::collections::VecDeque;

use rand::Rng;

fn main() {
    let g = square_sum_graph(100);
    //println!("{:?}", find_hamiltonian(g));
}

fn integers() -> std::ops::Range<usize> {
    1..usize::max_value()
}

fn squares() -> std::iter::Map<std::ops::Range<usize>, fn(usize) -> usize> {
    integers().map(|x| x * x)
}

fn square_sum_graph(n: usize) -> petgraph::Graph<usize, u8, petgraph::Undirected, usize> {
    let s: Vec<usize> = squares().take_while(|&x| x <= (n * 2) - 1).collect();
    let mut deps = petgraph::Graph::default(); // TODO use with_capacity
    for i in integers().take(n) {
        deps.add_node(i);
        for j in integers().take(i) {
            if s.contains(&(i + j)) {
                let i_index = petgraph::graph::node_index(i - 1);
                let j_index = petgraph::graph::node_index(j - 1);
                deps.add_edge(i_index, j_index, 1);
            }
        }
    }
    deps
}

fn order(g: &petgraph::Graph<usize, u8, petgraph::Undirected, usize>) -> usize {
    g.node_count()
}

/// http://doc.sagemath.org/html/en/reference/graphs/sage/graphs/generic_graph_pyx.html#sage.graphs.generic_graph_pyx.find_hamiltonian
/// https://github.com/sagemath/sage/blob/master/src/sage/graphs/generic_graph_pyx.pyx
fn find_hamiltonian(
    g: &petgraph::Graph<usize, u8, petgraph::Undirected, usize>,
) -> Option<Vec<petgraph::graph::NodeIndex<usize>>> {
    if petgraph::algo::connected_components(&g) != 1 {
        return None;
    }

    let mut rng = rand::thread_rng();
    let start = rng.gen_range(1, order(g) + 1);

    //let start = petgraph::graph::node_index(start);
    //let goal = petgraph::graph::node_index(5);

    let path: VecDeque<usize> = VecDeque::with_capacity(order(g));
    let member: Vec<bool> = vec![false; order(g)];
    let path: Vec<usize> = Vec::with_capacity(order(g));

    None
}
