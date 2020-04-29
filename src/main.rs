extern crate simple_logger;

use std::collections::HashMap;

use log::*;
use rand::seq::SliceRandom;

use crate::rider::Rider;
use petgraph::stable_graph::NodeIndex;
use petgraph::Graph;

mod rider;

const NUM_LOCATIONS: usize = 15;
const NUM_RIDERS: usize = 5;
const MIN_TIP: u32 = 1;
const MAX_TIP: u32 = 10;
const MIN_T: f64 = 1.0;
const MAX_T: f64 = 120.0;
const T: f64 = 1000.0;

fn distance(node_1: Option<&(f64, f64)>, node_2: Option<&(f64, f64)>) -> f64 {
    let node_1 = node_1.unwrap();
    let node_2 = node_2.unwrap();
    ((node_1.0 - node_2.0).powi(2) + (node_1.1 - node_2.1).powi(2)).sqrt()
}

fn cost(g: &Graph<&(f64, f64), &f64>, route: &Vec<&usize>) -> f64 {
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

fn nearest_neighbor(g: &Graph<&(f64, f64), &f64>, route: &Vec<&usize>) -> Vec<usize> {
    let mut nn_route = vec![0; route.len()];
    nn_route[0] = *route[0];

    for i in 0..nn_route.len() {
        let mut best_n = None;
        let mut best_distance = ((MAX_T - MIN_T).powi(2) + (MAX_T - MIN_T).powi(2)).sqrt() + 1.0;
        let r_index = NodeIndex::new(nn_route[i]);
        for n in g.neighbors(r_index) {
            if !nn_route.contains(&&n.index()) {
                let edge = g.find_edge(r_index, n).unwrap();
                if **g.edge_weight(edge).unwrap() < best_distance {
                    best_n = Some(n);
                    best_distance = **g.edge_weight(edge).unwrap();
                }
            }
        }
        if best_n.is_some() {
            nn_route[i + 1] = best_n.unwrap().index();
        }
    }
    nn_route[route.len() - 1] = *route[0];
    nn_route
}

struct CGATree {
    graph: Graph<(f64, f64), f64>,
    best_difference: f64,
    best_riders: HashMap<usize, Rider>,
    nodes_number: u32,
    finished: bool,
}

impl CGATree {
    fn new(graph: Graph<(f64, f64), f64>, best_difference: f64) -> Self {
        Self {
            graph,
            best_difference,
            best_riders: HashMap::new(),
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

            let _max_one_tree_max = 0.0;
            let one_tree_bound = false;

            if riders_sorted.last().unwrap().1.orders.keys().len() >= 4
                && val < self.best_difference
            {
                for r in riders_sorted.iter() {
                    if !one_tree_bound {
                        let nodes = &r.1.orders;
                        let _max_one_tree = 0;
                        if nodes.len() >= 4 {
                            for node in nodes.keys() {
                                let mut r_nodes = vec![];
                                for nd in nodes.keys() {
                                    if nd != node {
                                        r_nodes.push(nd);
                                    }
                                }

                                let _h = self.graph.filter_map(
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
                let mut tsp_bound = false;
                if val > 0.0 && locations.is_empty() {
                    for r in riders.iter() {
                        if !tsp_bound {
                            let r_orders: Vec<&usize> = r.1.orders.keys().collect();
                            let mut route_map = HashMap::new();
                            let mut index = 0usize;
                            let h = self.graph.filter_map(
                                |n, nw| {
                                    if r_orders.contains(&&n.index()) {
                                        route_map.insert(n.index(), index);
                                        index += 1;
                                        Some(nw)
                                    } else {
                                        None
                                    }
                                },
                                |_, ew| Some(ew),
                            );
                            let mut route: Vec<&usize> =
                                r_orders.iter().map(|i| route_map.get(i).unwrap()).collect();
                            route.push(route_map.get(&NUM_LOCATIONS).unwrap());
                            let greedy_route = nearest_neighbor(&h, &route);
                            let greedy_route_ref = greedy_route.iter().map(|x| x).collect();
                            // Compute 2-opt of previous
                            if cost(&h, &greedy_route_ref) > T {
                                debug!("Bound: Route cost: {}", cost(&h, &greedy_route_ref));
                                tsp_bound = true;
                            } else {
                                // r.1.route = greedy_route;
                            }
                        }
                    }
                }
                if val > 0.0 && locations.is_empty() && !tsp_bound {
                    self.best_difference = val;
                    info!("New best difference: {}", val);
                    self.best_riders = riders.clone();
                    info!("New best partition: {:?}", riders);
                    info!("Sums: ");
                    for r in &riders {
                        info!("Rider #{}: {}", r.0, r.1.sum_tips());
                    }
                }
                if locations.len() > 0 {
                    let mut next_locations = vec![];
                    let next_tip = locations_sorted[0];
                    for l in locations_sorted.iter() {
                        if l.1 == next_tip.1 {
                            next_locations.push(l.0);
                        }
                    }

                    let mut different_sum_riders_indexes = HashMap::new();
                    for r in riders_sorted.iter() {
                        if !different_sum_riders_indexes
                            .values()
                            .collect::<Vec<&f64>>()
                            .contains(&&r.1.sum_tips())
                        {
                            different_sum_riders_indexes.insert(r.0, r.1.sum_tips());
                        }
                    }

                    for index in different_sum_riders_indexes.iter() {
                        let mut new_locations = locations.clone();
                        let mut new_riders = riders.clone();

                        let updated_new_rider = new_riders.get_mut(index.0).unwrap();
                        let near_orders = &updated_new_rider.orders;
                        let mut best_cost = f64::MAX;
                        let mut best_location = None;
                        for l in next_locations.iter() {
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
                            let greedy_route = nearest_neighbor(&h, &route);
                            let greedy_route = greedy_route.iter().map(|x| x).collect();
                            let route_cost = cost(&h, &greedy_route);
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
    let t_range: Vec<u32> = (MIN_T as u32..MAX_T as u32).collect();
    for _ in 0..NUM_LOCATIONS + 1 {
        let x = *t_range.choose(&mut rng).unwrap() as f64;
        let y = *t_range.choose(&mut rng).unwrap() as f64;
        graph.add_node((x, y));
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
