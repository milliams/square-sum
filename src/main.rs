extern crate petgraph;
extern crate rand;
extern crate time;

use std::cmp::{max, min};

use rand::Rng;

fn main() {
    println!("Generating graph... ");
    let start_graph = time::PreciseTime::now();
    let g = square_sum_graph(usize::pow(2, 18));
    let end_graph = time::PreciseTime::now();
    println!("{} seconds", start_graph.to(end_graph));

    println!("nodes: {} edges: {}", g.node_count(), g.edge_count());

    println!("Finding Hamiltonian path... ");
    let start_ham = time::PreciseTime::now();
    let ham = find_hamiltonian(&g);
    let end_ham = time::PreciseTime::now();
    println!("{} seconds", start_ham.to(end_ham));

    match ham {
        Ok(h) => {
            println!("found");
            println!("valid square-sum sequence {}", check_sum_squares(&h));
        }
        Err(e) => {
            println!("fail {:?}", e);
            ::std::process::exit(1);
        }
    }
}

fn integers() -> std::ops::Range<usize> {
    1..usize::max_value()
}

fn squares() -> std::iter::Map<std::ops::Range<usize>, fn(usize) -> usize> {
    integers().map(|x| x * x)
}

fn square_sum_graph(n: usize) -> petgraph::Graph<(), (), petgraph::Undirected, usize> {
    let mut g = init_square_sum_path(n);
    let s: Vec<usize> = squares().take_while(|&x| x <= (n * 2) - 1).collect();
    for _i in integers().take(n) {
        add_square_sum_node(&mut g, &s);
    }
    g
}

fn init_square_sum_path(n: usize) -> petgraph::Graph<(), (), petgraph::Undirected, usize> {
    let num_edges: usize = integers()
        .take(n)
        .map(|i| {
            f64::floor(f64::sqrt(((i * 2) - 1) as f64)) as usize
                - f64::floor(f64::sqrt(i as f64)) as usize
        })
        .sum();
    petgraph::Graph::with_capacity(n, num_edges)
}

fn add_square_sum_node(
    g: &mut petgraph::Graph<(), (), petgraph::Undirected, usize>,
    square_numbers: &[usize],
) {
    let i = g.node_count() + 1;
    g.add_node(());
    for sq in square_numbers
        .iter()
        .skip_while(|&sq| sq <= &i)
        .take_while(|&sq| sq <= &((i * 2) - 1))
    {
        let i_index = petgraph::graph::node_index(i - 1);
        let j_index = petgraph::graph::node_index(sq - i - 1);
        g.update_edge(i_index, j_index, ());
    }
}

fn order<N, E, Ty>(g: &petgraph::Graph<N, E, Ty, usize>) -> usize
where
    Ty: petgraph::EdgeType,
{
    g.node_count()
}

struct Path {
    path: Vec<usize>,
    member: Vec<bool>,
}

impl Path {
    fn new(size: usize) -> Path {
        Path {
            path: Vec::with_capacity(size),
            member: vec![false; size],
        }
    }

    fn push(&mut self, node_index: usize) {
        self.path.push(node_index);
        self.member[node_index] = true;
    }

    fn len(&self) -> usize {
        self.path.len()
    }

    fn contains(&self, node_index: usize) -> bool {
        self.member[node_index]
    }

    fn backtrack(&mut self, amount: usize) {
        let actual_backtrack_amount = min(amount, self.path.len() - 2);
        for i in &self.path[(self.path.len() - actual_backtrack_amount)..] {
            self.member[*i] = false;
        }
        let new_size = self.path.len() - actual_backtrack_amount;
        self.path.truncate(new_size);
    }

    fn reverse(&mut self) {
        self.path.reverse();
    }

    fn iter(&self) -> std::slice::Iter<usize> {
        self.path.iter()
    }
}

fn setup_path<N, E, Ty>(g: &petgraph::Graph<N, E, Ty, usize>) -> Result<Path, &'static str>
where
    Ty: petgraph::EdgeType,
{
    let mut rng = rand::thread_rng();

    let start = petgraph::graph::node_index(rng.gen_range(0, order(g)));
    let neighbours = g.neighbors(start).collect::<Vec<_>>();
    let next = rng.choose(&neighbours).ok_or("Node had no neighbours!")?;

    let mut path = Path::new(order(g));

    path.push(start.index());
    path.push(next.index());

    Ok(path)
}

fn find_hamiltonian<N, E, Ty>(
    g: &petgraph::Graph<N, E, Ty, usize>,
) -> Result<Vec<usize>, &'static str>
where
    Ty: petgraph::EdgeType,
{
    if petgraph::algo::connected_components(&g) != 1 {
        return Err("Not a fully-connected graph");
    }

    let reverse_rate = max(100, g.node_count() / 1000);
    let backtrack_rate = max(1000, g.node_count() / 100);
    let backtrack_amount = max(5, g.node_count() / 10000);
    let reset_rate = g.node_count() * 10; // Must be larger than num nodes
    let max_iterations = reset_rate * 5;

    let mut rng = rand::thread_rng();

    let mut path = setup_path(g)?;

    let mut longest_path: Vec<usize> = Vec::with_capacity(order(g));

    let mut iteration = 0;
    let mut resets = 0;

    loop {
        // Reverse the path often
        if iteration % reverse_rate == 0 {
            path.reverse();
        }

        // Reset the search occasionally
        if iteration > reset_rate {
            iteration = 1;
            resets += 1;
            path = setup_path(g)?;
            continue;
        }

        // Backtrack a smidge now and again
        if iteration % backtrack_rate == 0 {
            path.backtrack(backtrack_amount);
        }

        // Current vertex is `v`
        let v = *path.path
            .last()
            .ok_or("There should be at least one node in the path")?;

        // Create list of possible next vertices
        let possible_next_nodes: Vec<_> = g.neighbors((v).into())
            .filter(|n| !path.contains(n.index()))
            .collect();
        let next = rng.choose(&possible_next_nodes)
            .and_then(|i| Some(i.index()));

        // If there are any, choose one randomly and add it to the path
        if let Some(v) = next {
            path.push(v);
        } else {
            // but we have a new longest path anyway, so set `longest_path`
            if path.len() > longest_path.len() {
                longest_path = path.path.clone();
            }
            // choose any neighbour, `n`, of `v` (which must already be in `path`) and reverse path from `n` (not including n) to `v`
            let previous_node = path.path[path.len() - 2];
            let possible_pivots: Vec<_> = g.neighbors((v).into())
                .filter(|n| n.index() != previous_node)
                .collect();
            if let Some(pivot) = rng.choose(&possible_pivots) {
                let pivot_pos = path.iter()
                    .position(|&v| v == pivot.index())
                    .ok_or("Pivot must be in the path")?;
                path.path[pivot_pos + 1..].reverse();
            }
        }

        // If we've found all nodes, return
        if path.len() == g.node_count() {
            println!("iterations: {:?}", iteration);
            println!("resets: {:?}", resets);
            println!("total iterations: {:?}", resets * reset_rate);
            return Ok(path.iter().map(|&a| a + 1).collect());
        }

        // If we've 'timed out', fail
        if resets * reset_rate > max_iterations {
            println!("iterations: {:?}", iteration);
            println!("resets: {:?}", resets);
            println!("total iterations: {:?}", resets * reset_rate);
            return Err("Timeout");
        }

        iteration += 1;
    }
}

fn check_sum_squares(vals: &[usize]) -> bool {
    let s: Vec<usize> = squares()
        .take_while(|&x| x <= (vals.len() * 2) - 1)
        .collect();
    vals.iter()
        .zip(vals.iter().skip(1))
        .all(|(&a, &b)| s.contains(&(a + b)))
}
