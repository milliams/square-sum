extern crate petgraph;
extern crate rand;
extern crate time;
extern crate clap;

use std::cmp::{max, min};
use std::collections::HashSet;

use rand::Rng;
use time::PreciseTime;

enum Method {
    Any,
    All,
}

fn main() {
    let matches = clap::App::new("square-sum")
        .about("Calculates solutions to the square sum problem")
        .author("Matt Williams")
        .arg(clap::Arg::with_name("start")
            .short("s")
            .long("start")
            .value_name("N")
            .default_value("0")
            .help("The start of the sequence to calculate"))
        .arg(clap::Arg::with_name("end")
            .short("e")
            .long("end")
            .value_name("N")
            .default_value("100")
            .help("The end of the sequence to calculate (exclusive)"))
        .arg(clap::Arg::with_name("find")
            .short("f")
            .long("find")
            .value_name("METHOD")
            .default_value("any")
            .possible_values(&["any", "all"])
            .help("Whether to find *all* paths for each graph or *any* path for each graph"))
        .get_matches();

    let start_time = PreciseTime::now();

    let start: usize = matches.value_of("start").unwrap().parse().expect("Could not parse start value");
    let limit: usize = matches.value_of("end").unwrap().parse().expect("Could not parse end value");
    let method = match matches.value_of("find").unwrap() {
        "any" => Method::Any,
        "all" => Method::All,
        _ => panic!(),
    };

    let mut g = init_square_sum_path(limit);
    let s: Vec<usize> = squares().take_while(|&x| x <= (limit * 2) - 1).collect();

    // Prime the graph up to the start of the search
    for _ in 1..start {
        add_square_sum_node(&mut g, &s);
    }

    let mut ham = None; // Cache for previous loop's path


    match method {
        Method::All => {
            for _ in start..limit {
                add_square_sum_node(&mut g, &s);
                let paths = find_all_paths(&g);
                if !paths.is_empty() {
                    let next_num = g.node_count() + 1;
                    let relevant_squares: Vec<_> = squares()
                        .skip_while(|&sq| sq <= next_num)
                        .take_while(|&sq| sq <= (next_num * 2) - 1)
                        .collect();
                    let magic_paths: Vec<_> = paths
                        .iter()
                        .filter(|&p| {
                            relevant_squares
                                .iter()
                                .any(|sq| *p.first().unwrap() == sq - next_num || *p.last().unwrap() == sq - next_num)
                        })
                        .collect();
                    if magic_paths.is_empty() {
                        println!("{} has no magic paths", g.node_count());
                    } else {
                        println!("{} has {} magic paths", g.node_count(), magic_paths.len());
                    }
                }
            }
        },
        Method::Any => {
            for _ in start..limit {
                add_square_sum_node(&mut g, &s);
                ham = find_any_path(&g, ham);
            }
        }
    }

    let end_time = PreciseTime::now();
    println!("{} seconds.", start_time.to(end_time).num_seconds());
}

fn find_any_path<N, E, Ty>(
    g: &petgraph::Graph<N, E, Ty, usize>,
    ham: Option<Vec<usize>>,
) -> Option<Vec<usize>>
where
    Ty: petgraph::EdgeType,
{
    match find_hamiltonian(g, ham) {
        Ok(h) => Some(h),
        Err(e) => {
            println!("{} fails with {}", g.node_count(), e);
            None
        }
    }
}

fn find_all_paths<N, E, Ty>(g: &petgraph::Graph<N, E, Ty, usize>) -> HashSet<std::vec::Vec<usize>>
where
    Ty: petgraph::EdgeType,
{
    let mut tries = 0;
    let mut failed_tries = 0;
    let mut paths = HashSet::new();
    loop {
        tries += 1;

        let ham = match find_hamiltonian(g, None) {
            Ok(h) => Some(h),
            Err(_) => None,
        };

        if let Some(mut p) = ham.clone() {
            if p.first().unwrap() > p.last().unwrap() {
                p.reverse();
            }
            if paths.insert(p) {
                failed_tries = 0;
            } else {
                failed_tries += 1;
            }
        } else {
            failed_tries += 1;
        }

        if failed_tries > max(3, (tries as f32 * 0.7) as usize) {
            break;
        }
    }
    println!(
        "{} has {} paths from {} tries",
        g.node_count(),
        paths.len(),
        tries
    );

    paths
}

fn integers() -> std::ops::Range<usize> {
    1..usize::max_value()
}

fn squares() -> std::iter::Map<std::ops::Range<usize>, fn(usize) -> usize> {
    integers().map(|x| x * x)
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

    fn from_seed(seed: &[usize], size: usize) -> Path {
        // TODO check that size >= seed.len()
        let mut path = Vec::with_capacity(size);
        let mut member = vec![false; size];
        for i in seed.iter() {
            path.push(i - 1);
            member[*i - 1] = true;
        }
        Path { path, member }
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

    let start = petgraph::graph::node_index(rng.gen_range(0, g.node_count()));
    let neighbours = g.neighbors(start).collect::<Vec<_>>();
    let next = rng.choose(&neighbours).ok_or("Node had no neighbours!")?;

    let mut path = Path::new(g.node_count());

    path.push(start.index());
    path.push(next.index());

    Ok(path)
}

fn find_hamiltonian<N, E, Ty>(
    g: &petgraph::Graph<N, E, Ty, usize>,
    seed: Option<Vec<usize>>,
) -> Result<Vec<usize>, &'static str>
where
    Ty: petgraph::EdgeType,
{
    if petgraph::algo::connected_components(&g) != 1 {
        return Err("Not a fully-connected graph");
    }

    let reverse_rate = max(100, g.node_count() / 1000);
    let backtrack_rate = max(1000, g.node_count() / 100);
    let backtrack_amount = max(5, g.node_count() / 10_000);
    let reset_rate = g.node_count() * 10; // Must be larger than num nodes
    let max_iterations = reset_rate * 5;

    let mut rng = rand::thread_rng();

    let mut path = match seed {
        Some(s) => Path::from_seed(&s, g.node_count()),
        None => setup_path(g)?,
    };

    let mut longest_path: Vec<usize> = Vec::with_capacity(g.node_count());

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
        let possible_next_nodes: Vec<_> = g.neighbors(v.into())
            .filter(|n| !path.contains(n.index()))
            .collect();

        // If there are any, choose one randomly and add it to the path
        if let Some(v) = rng.choose(&possible_next_nodes) {
            path.push(v.index());
        } else {
            // but we have a new longest path anyway, so set `longest_path`
            if path.len() > longest_path.len() {
                longest_path = path.path.clone();
            }
            // choose any neighbour, `n`, of `v` (which must already be in `path`) and reverse path from `n` (not including n) to `v`
            let previous_node = path.path[path.len() - 2];
            let possible_pivots: Vec<_> = g.neighbors(v.into())
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
            return Ok(path.iter().map(|&a| a + 1).collect());
        }

        // If we've 'timed out', fail
        if resets * reset_rate > max_iterations {
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
