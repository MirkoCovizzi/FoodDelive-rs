extern crate simple_logger;

use std::collections::HashMap;

use log::*;
use rand::seq::SliceRandom;

use crate::rider::Rider;
use petgraph::Graph;

mod rider;

const NUM_LOCATIONS: u32 = 15;
const NUM_RIDERS: u32 = 5;
const MIN_TIP: u32 = 1;
const MAX_TIP: u32 = 10;
const MIN_T: u32 = 1;
const MAX_T: u32 = 120;

struct CgaTree {
    best_difference: f64,
    nodes_number: u32,
    finished: bool,
}

impl CgaTree {
    fn new(best_difference: f64) -> Self {
        Self {
            best_difference,
            nodes_number: 0,
            finished: false,
        }
    }

    fn search(&mut self, locations: HashMap<u32, f64>, riders: HashMap<u32, Rider>) {
        if !self.finished {
            self.nodes_number += 1;

            let mut locations_sorted: Vec<&f64> = locations.values().collect();
            locations_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            locations_sorted.reverse();

            let mut riders_sorted: Vec<&Rider> = riders.values().collect();
            riders_sorted.sort();

            let mut sum_riders = 0.0;
            for r in riders_sorted[..riders_sorted.len() - 1].iter() {
                sum_riders += r.sum_tips();
            }

            let locations_values_sum: f64 = locations.values().sum();

            let sum_tot = sum_riders + locations_values_sum;

            let max_rider = riders_sorted.last().unwrap().sum_tips();

            let val = max_rider - (sum_tot / (NUM_RIDERS as f64 - 1.0));

            let _max_one_tree_max = 0.0;
            let one_tree_bound = false;

            if riders_sorted.last().unwrap().orders.keys().len() >= 4 && val < self.best_difference
            {
                for r in riders_sorted {
                    if !one_tree_bound {
                        let nodes = r.orders.keys();
                        let _max_one_tree = 0;
                        if nodes.len() >= 4 {
                            for node in nodes.clone() {
                                let mut r_nodes = vec![];
                                for nd in nodes.clone() {
                                    if nd != node {
                                        r_nodes.push(nd);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if val < self.best_difference && !one_tree_bound {
                let tsp_bound = false;
                if val > 0.0 && locations.is_empty() {
                    for _r in riders {
                        if !tsp_bound {}
                    }
                }
                if val > 0.0 && locations.is_empty() && !tsp_bound {
                    self.best_difference = val;
                }
            }
        }
    }
}

fn distance(node_1: Option<&(u32, f64, f64)>, node_2: Option<&(u32, f64, f64)>) -> (u32, f64, f64) {
    let node_1 = node_1.unwrap();
    let node_2 = node_2.unwrap();
    let x_1 = node_1.1;
    let x_2 = node_2.1;
    let y_1 = node_1.2;
    let y_2 = node_2.2;
    (0, 0.0, ((x_1 - x_2).powi(2) + (y_1 - y_2).powi(2)).sqrt())
}

fn main() {
    simple_logger::init().expect("Can't initialize logging.");
    let mut rng = rand::thread_rng();

    let mut graph = Graph::<(u32, f64, f64), (u32, f64, f64)>::new();
    let t_range: Vec<u32> = (MIN_T..MAX_T).collect();
    for i in 0..NUM_LOCATIONS + 1 {
        let x = *t_range.choose(&mut rng).unwrap() as f64;
        let y = *t_range.choose(&mut rng).unwrap() as f64;
        let tup = (i, x, y);
        graph.add_node(tup);
    }
    for i in graph.node_indices() {
        for j in graph.node_indices() {
            if i != j {
                graph.add_edge(i, j, distance(graph.node_weight(i), graph.node_weight(j)));
            }
        }
    }

    let tip_range: Vec<u32> = (MIN_TIP..MAX_TIP).collect();
    let mut locations = HashMap::new();
    let mut riders = HashMap::new();

    for i in 0..NUM_LOCATIONS {
        locations.insert(i, *tip_range.choose(&mut rng).unwrap() as f64);
    }

    for i in 0..NUM_RIDERS {
        let mut rider = Rider::new(i);
        rider.add_tip(NUM_LOCATIONS, 0.0);
        riders.insert(i, rider);
    }

    let mut locations_sorted: Vec<&f64> = locations.values().collect();

    locations_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    locations_sorted.reverse();
    info!("{:?}", locations_sorted);

    let locations_values_sum: f64 = locations.values().sum();
    info!("Tot: {:?}", locations_values_sum);

    let max_opt_relax = (locations_values_sum / NUM_RIDERS as f64).ceil();
    let opt_relax =
        max_opt_relax - (locations_values_sum - max_opt_relax) / (NUM_RIDERS as f64 - 1.0);
    info!("Z* Relaxed: {:?}", opt_relax);

    let mut cga_tree = CgaTree::new(locations_values_sum);
    cga_tree.search(locations, riders);
}
