extern crate simple_logger;

use std::collections::HashMap;

use log::*;
use rand::seq::SliceRandom;

use crate::rider::Rider;
use petgraph::stable_graph::{EdgeIndex, NodeIndex};
use petgraph::{Directed, Graph};
use std::borrow::Borrow;

mod rider;

const NUM_LOCATIONS: usize = 15;
const NUM_RIDERS: usize = 5;
const MIN_TIP: u32 = 1;
const MAX_TIP: u32 = 10;
const MIN_T: u32 = 1;
const MAX_T: u32 = 120;

fn distance(node_1: Option<&(f64, f64)>, node_2: Option<&(f64, f64)>) -> f64 {
    let node_1 = node_1.unwrap();
    let node_2 = node_2.unwrap();
    ((node_1.0 - node_2.0).powi(2) + (node_1.1 - node_2.1).powi(2)).sqrt()
}

fn cost(g: Graph<&(f64, f64), &f64>, route: Vec<&usize>) -> f64 {
    let mut c = 0.0;
    for i in 0..route.len() - 1 {
        let first = route[i];
        let second = route[i + 1];
        let edge = g
            .find_edge(NodeIndex::new(*first), NodeIndex::new(*second))
            .unwrap();
        c += **g.edge_weight(edge).unwrap();
    }
    c
}

struct CGATree {
    graph: Graph<(f64, f64), f64>,
    best_difference: f64,
    nodes_number: u32,
    finished: bool,
}

impl CGATree {
    fn new(graph: Graph<(f64, f64), f64>, best_difference: f64) -> Self {
        Self {
            graph,
            best_difference,
            nodes_number: 0,
            finished: false,
        }
    }

    fn search(&mut self, locations: HashMap<usize, f64>, riders: HashMap<usize, Rider>) {
        if !self.finished {
            self.nodes_number += 1;

            let mut locations_sorted: Vec<(&usize, &f64)> = locations.iter().collect();
            locations_sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
            locations_sorted.reverse();

            let mut riders_sorted: Vec<(&usize, &Rider)> = riders.iter().collect();
            riders_sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

            let mut sum_riders = 0.0;
            for r in riders_sorted[..riders_sorted.len() - 1].iter() {
                sum_riders += r.1.sum_tips();
            }

            let locations_values_sum: f64 = locations.values().sum();

            let sum_tot = sum_riders + locations_values_sum;

            let max_rider = riders_sorted.last().unwrap().1.sum_tips();

            let val = max_rider - (sum_tot / (NUM_RIDERS as f64 - 1.0));

            let max_one_tree_max = 0.0;
            let one_tree_bound = false;

            if riders_sorted.last().unwrap().1.orders.keys().len() >= 4
                && val < self.best_difference
            {
                for r in &riders_sorted {
                    if !one_tree_bound {
                        let nodes = &r.1.orders;
                        let max_one_tree = 0;
                        if nodes.len() >= 4 {
                            for node in nodes.keys() {
                                let mut r_nodes = vec![];
                                for nd in nodes.keys() {
                                    if nd != node {
                                        r_nodes.push(nd);
                                    }
                                }

                                let h = self.graph.filter_map(
                                    |n, _| {
                                        if r_nodes.contains(&&n.index()) {
                                            Some(n)
                                        } else {
                                            None
                                        }
                                    },
                                    |e, _| Some(e),
                                );
                            }
                        }
                    }
                }
            }

            if val < self.best_difference && !one_tree_bound {
                let tsp_bound = false;
                if val > 0.0 && locations.is_empty() {
                    for r in &riders {
                        if !tsp_bound {
                            let h = self.graph.filter_map(
                                |n, _| {
                                    if r.1.orders.contains_key(&n.index()) {
                                        Some(n)
                                    } else {
                                        None
                                    }
                                },
                                |e, _| Some(e),
                            );
                            let route: Vec<&usize> = r.1.orders.keys().collect();
                            // Compute greedy route
                            // Compute 2-opt of previous
                        }
                    }
                }
                if val > 0.0 && locations.is_empty() && !tsp_bound {
                    self.best_difference = val;
                }
                if locations.len() > 0 {
                    let mut next_locations = vec![];
                    let next_tip = locations_sorted[0];
                    for l in locations_sorted {
                        if l.1 == next_tip.1 {
                            next_locations.push(l.0);
                        }
                    }

                    let mut different_sum_riders_indexes = HashMap::new();
                    for r in &riders_sorted {
                        if !different_sum_riders_indexes
                            .values()
                            .collect::<Vec<&f64>>()
                            .contains(&&r.1.sum_tips())
                        {
                            different_sum_riders_indexes.insert(r.0, r.1.sum_tips());
                        }
                    }

                    for index in different_sum_riders_indexes {
                        let mut new_locations = locations.clone();
                        let mut new_riders = riders.clone();

                        let updated_new_rider = new_riders.get_mut(index.0).unwrap();
                        let near_orders = &updated_new_rider.orders;
                        let mut best_cost = f64::MAX;
                        let mut best_location = None;
                        for l in &next_locations {
                            let mut updated_orders = vec![];
                            updated_orders.extend(near_orders.keys().into_iter());
                            updated_orders.push(*l);
                            let mut route_map = HashMap::new();
                            let mut index = 0usize;
                            let h = self.graph.filter_map(
                                |n, nw| {
                                    if updated_orders.contains(&&n.index()) {
                                        route_map.insert(n.index(), index);
                                        index += 1;
                                        Some(nw)
                                    } else {
                                        None
                                    }
                                },
                                |_, ew| Some(ew),
                            );
                            let mut route: Vec<&usize> = updated_orders
                                .iter()
                                .map(|i| route_map.get(i).unwrap())
                                .collect();
                            route.push(route_map.get(&NUM_LOCATIONS).unwrap());
                            // NN algorithm
                            let route_cost = cost(h, route);
                            if route_cost < best_cost {
                                best_cost = route_cost;
                                best_location = Some(l);
                            }
                        }

                        updated_new_rider.add_tip(**best_location.unwrap(), *next_tip.1);

                        new_locations.remove(best_location.unwrap());

                        self.search(new_locations, new_riders);
                    }
                }
            }
        }
    }
}

fn main() {
    simple_logger::init().expect("Can't initialize logging.");
    let mut rng = rand::thread_rng();

    let mut graph: Graph<(f64, f64), f64> = Graph::new();
    let t_range: Vec<u32> = (MIN_T..MAX_T).collect();
    for _ in 0..NUM_LOCATIONS + 1 {
        let x = *t_range.choose(&mut rng).unwrap() as f64;
        let y = *t_range.choose(&mut rng).unwrap() as f64;
        let tup = (x, y);
        graph.add_node(tup);
    }
    for i in graph.node_indices() {
        for j in graph.node_indices() {
            if i != j {
                graph.add_edge(i, j, distance(graph.node_weight(i), graph.node_weight(j)));
            }
        }
    }

    dbg!(&graph);

    let tip_range: Vec<u32> = (MIN_TIP..MAX_TIP).collect();
    let mut locations = HashMap::new();
    let mut riders = HashMap::new();

    for i in 0..NUM_LOCATIONS {
        locations.insert(i, *tip_range.choose(&mut rng).unwrap() as f64);
    }
    info!("Locations: {:?}", locations);

    for i in 0..NUM_RIDERS {
        let mut rider = Rider::new(i);
        rider.add_tip(NUM_LOCATIONS, 0.0);
        riders.insert(i, rider);
    }

    let mut locations_sorted: Vec<&f64> = locations.values().collect();
    locations_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    locations_sorted.reverse();
    info!("Locations Sorted: {:?}", locations_sorted);

    let locations_values_sum: f64 = locations.values().sum();
    info!("Tot: {:?}", locations_values_sum);

    let max_opt_relax = (locations_values_sum / NUM_RIDERS as f64).ceil();
    let opt_relax =
        max_opt_relax - (locations_values_sum - max_opt_relax) / (NUM_RIDERS as f64 - 1.0);
    info!("Z* Relaxed: {:?}", opt_relax);

    let mut cga_tree = CGATree::new(graph, locations_values_sum);
    cga_tree.search(locations, riders);
}
