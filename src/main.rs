extern crate petgraph;
extern crate rand;

use std::cmp::min;

use rand::Rng;

fn main() {
    let g = square_sum_graph(usize::pow(2, 10));
    let ham = find_hamiltonian(&g);
    match ham {
        Some(h) => {println!("found")},
        None => {println!("fail")}
    }
    //println!("{:?}", ham);
}

fn integers() -> std::ops::Range<usize> {
    1..usize::max_value()
}

fn squares() -> std::iter::Map<std::ops::Range<usize>, fn(usize) -> usize> {
    integers().map(|x| x * x)
}

fn square_sum_graph(n: usize) -> petgraph::Graph<usize, u8, petgraph::Undirected, usize> {
    let s: Vec<usize> = squares().take_while(|&x| x <= (n * 2) - 1).collect();
    let mut g = petgraph::Graph::default(); // TODO use with_capacity
    for i in integers().take(n) {
        g.add_node(i);
        for j in integers().take(i) {
            if i == j {
                continue;
            }
            if s.contains(&(i + j)) {
                let i_index = petgraph::graph::node_index(i - 1);
                let j_index = petgraph::graph::node_index(j - 1);
                g.add_edge(i_index, j_index, 1);
            }
        }
    }
    g
}

fn order(g: &petgraph::Graph<usize, u8, petgraph::Undirected, usize>) -> usize {
    g.node_count()
}

fn setup_path(g: &petgraph::Graph<usize, u8, petgraph::Undirected, usize>, path: &mut Vec<usize>, member: &mut Vec<bool>) {
    let mut rng = rand::thread_rng();

    let start = petgraph::graph::node_index(rng.gen_range(0, order(g)));
    //println!("start: {:?}", start);
    let neighbours = g.neighbors(start).collect::<Vec<_>>();
    //println!("neighbours: {:?}", neighbours);
    let next = rng.choose(&neighbours).expect("Node had no neighbours!").index();

    path.clear();
    member.iter_mut().map(|x| *x = false).count();

    path.push(start.index());
    path.push(next);
    member[start.index()] = true;
    member[next] = true;
}

/// http://doc.sagemath.org/html/en/reference/graphs/sage/graphs/generic_graph_pyx.html#sage.graphs.generic_graph_pyx.find_hamiltonian
/// https://github.com/sagemath/sage/blob/master/src/sage/graphs/generic_graph_pyx.pyx
fn find_hamiltonian(
    g: &petgraph::Graph<usize, u8, petgraph::Undirected, usize>,
) -> Option<Vec<usize>> {
    if petgraph::algo::connected_components(&g) != 1 {
        return None;
    }

    let reverse_rate = 10;
    let backtrack_rate = 1000;
    let reset_rate = 30_000;
    let max_iterations = 100_000;

    let mut rng = rand::thread_rng();

    let mut path: Vec<usize> = Vec::with_capacity(order(g));
    let mut member: Vec<bool> = vec![false; order(g)];
    let mut longest_path: Vec<usize> = Vec::with_capacity(order(g));

    let mut iteration = 0;
    let mut resets = 0;

    setup_path(g, &mut path, &mut member);

    loop {
        // Reverse the path often
        if iteration % reverse_rate == 0 {
            path.reverse();
        }

        // Reset the search occasionally
        if iteration > reset_rate {
            iteration = 1;
            resets += 1;
            setup_path(g, &mut path, &mut member);
            continue;
        }

        // Backtrack a smidge now and again
        if iteration % backtrack_rate == 0 {
            let backtrack_amount = min(5, path.len() - 2);
            for i in &path[(path.len() - backtrack_amount)..] {
                member[*i] = false;
            }
            let new_size = path.len() - backtrack_amount;
            path.truncate(new_size);
        }

        // Current vertex is `v`
        let v = *path.last().expect("There should be at least one node in the path");

        // Create list of possible next vertices
        let possible_next_nodes: Vec<_> = g.neighbors((v).into()).filter(|n| !member[n.index()]).collect();
        let next = rng.choose(&possible_next_nodes).and_then(|i| Some(i.index()));

        // If there are any, choose one randomly and add it to the path
        if let Some(v) = next {
            path.push(v);
            member[v] = true;
        } else {
            // but we have a new longest path anyway, so set `longest_path`
            if path.len() > longest_path.len() {
                longest_path = path.clone();
            }
            // choose any neighbour, `n`, of `v` (which must already be in `path`) and reverse path from `n` (not including n) to `v`
            let previous_node = path[path.len()-2];
            let possible_pivots: Vec<_> = g.neighbors((v).into()).filter(|n| n.index() != previous_node).collect();
            if let Some(pivot) = rng.choose(&possible_pivots) {
                let pivot_pos = path.iter().position(|&v| v == pivot.index()).expect("Pivot must be in the path");
                path[pivot_pos+1..].reverse();
            }
        }

        // If we've found all nodes, return
        if path.len() == g.node_count() {
            println!("iterations: {:?}", iteration);
            println!("resets: {:?}", resets);
            return Some(path);
        }

        // If we've 'timed out', fail
        if resets * reset_rate > max_iterations {
            return None;
        }

        iteration += 1;
    }
}
