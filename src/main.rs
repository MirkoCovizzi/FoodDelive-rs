extern crate simple_logger;

use std::collections::HashMap;

use log::*;
use petgraph::algo::min_spanning_tree;
use petgraph::data::FromElements;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Graph;
use rand::seq::SliceRandom;

use crate::rider::Rider;
mod rider;

const NUM_LOCATIONS: usize = 150;
const NUM_RIDERS: usize = 50;
const MIN_TIP: f64 = 1.0;
const MAX_TIP: f64 = 10.0;
const MIN_T: f64 = 1.0;
const MAX_T: f64 = 120.0;
const T: f64 = 1000.0;
const ERROR: f64 = 0.000_000_001;

fn distance(node_1: Option<&(f64, f64)>, node_2: Option<&(f64, f64)>) -> f64 {
    let node_1 = node_1.unwrap();
    let node_2 = node_2.unwrap();
    ((node_1.0 - node_2.0).powi(2) + (node_1.1 - node_2.1).powi(2)).sqrt()
}

fn cost(g: &Graph<&(f64, f64), &f64>, route: &[&usize]) -> f64 {
    let mut c = 0.0;
    for i in 0..route.len() - 1 {
        let first = route[i];
        let second = route[i + 1];
        if let Some(edge) = g.find_edge(NodeIndex::new(*first), NodeIndex::new(*second)) {
            c += **g.edge_weight(edge).unwrap();
        }
    }
    c
}

fn nearest_neighbor(g: &Graph<&(f64, f64), &f64>, route: &[&usize]) -> Vec<usize> {
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
        if let Some(best) = best_n {
            nn_route[i + 1] = best.index();
        }
    }
    nn_route[route.len() - 1] = *route[0];
    nn_route
}

struct CGATree {
    graph: Graph<(f64, f64), f64>,
    best_difference: f64,
    best_riders: HashMap<usize, Rider>,
    best_routes: HashMap<usize, Vec<usize>>,
    opt_relax: f64,
    nodes_number: u32,
    finished: bool,
}

impl CGATree {
    fn new(graph: Graph<(f64, f64), f64>, best_difference: f64, opt_relax: f64) -> Self {
        Self {
            graph,
            best_difference,
            best_riders: HashMap::new(),
            best_routes: HashMap::new(),
            opt_relax,
            nodes_number: 0,
            finished: false,
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn search(&mut self, locations: HashMap<usize, f64>, riders: HashMap<usize, Rider>) {
        if !self.finished {
            self.nodes_number += 1;

            let mut locations_sorted = locations.iter().collect::<Vec<_>>();
            locations_sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
            locations_sorted.reverse();

            let mut riders_sorted = riders.iter().collect::<Vec<_>>();
            riders_sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

            let mut sum_riders = 0.0;
            for r in riders_sorted[..riders_sorted.len() - 1].iter() {
                sum_riders += r.1.sum_tips();
            }

            let locations_values_sum = locations.values().sum::<f64>();
            let sum_tot = sum_riders + locations_values_sum;
            let max_rider = riders_sorted.last().unwrap().1.sum_tips();
            let val = max_rider - (sum_tot / (NUM_RIDERS as f64 - 1.0));

            let mut one_tree_bound = false;

            if riders_sorted.last().unwrap().1.orders.keys().len() >= 4
                && val < self.best_difference
            {
                for r in riders_sorted.iter() {
                    if !one_tree_bound {
                        let nodes = &r.1.orders;
                        let mut min_one_tree = f64::MAX;
                        if nodes.len() >= 4 {
                            for node in nodes.keys() {
                                let mut r_nodes = vec![];
                                for nd in nodes.keys() {
                                    if nd != node {
                                        r_nodes.push(nd);
                                    }
                                }
                                r_nodes.insert(0, &NUM_LOCATIONS);
                                r_nodes.push(&NUM_LOCATIONS);
                                let h = self.graph.filter_map(
                                    |n, nw| {
                                        if r_nodes.contains(&&n.index()) {
                                            Some(nw)
                                        } else {
                                            None
                                        }
                                    },
                                    |_, ew| Some(ew),
                                );
                                let mst = min_spanning_tree(&h);
                                let mst = Graph::<_, _>::from_elements(mst);
                                let mut sum_edges = 0.0;
                                for m in mst.edge_references() {
                                    sum_edges += **m.weight();
                                }
                                let mut best_edge_weight = f64::MAX;
                                let node_edges = self.graph.edges(NodeIndex::new(*node));
                                for e in node_edges {
                                    if r_nodes.contains(&&e.target().index()) {
                                        let w = *e.weight();
                                        if w < best_edge_weight {
                                            best_edge_weight = w;
                                        }
                                    }
                                }
                                sum_edges += best_edge_weight;
                                if sum_edges < min_one_tree {
                                    min_one_tree = sum_edges;
                                }
                            }
                        }
                        if min_one_tree > T {
                            one_tree_bound = true;
                            debug!("1-Tree bound!");
                        }
                    }
                }
            }

            if val < self.best_difference && !one_tree_bound {
                let mut riders_routes = HashMap::new();
                let mut tsp_bound = false;
                if val > 0.0 && locations.is_empty() {
                    debug!("Testing NN...");
                    for r in riders.iter() {
                        if !tsp_bound {
                            let mut r_orders = r.1.orders.keys().collect::<Vec<_>>();
                            r_orders.insert(0, &NUM_LOCATIONS);
                            r_orders.push(&NUM_LOCATIONS);
                            let h = self.graph.filter_map(
                                |_, nw| Some(nw),
                                |e, ew| {
                                    if let Some((n1, n2)) = self.graph.edge_endpoints(e) {
                                        if r_orders.contains(&&n1.index())
                                            && r_orders.contains(&&n2.index())
                                        {
                                            return Some(ew);
                                        }
                                    }
                                    None
                                },
                            );
                            let greedy_route = nearest_neighbor(&h, &r_orders);
                            let greedy_route_ref =
                                greedy_route.iter().map(|x| x).collect::<Vec<_>>();
                            if cost(&h, &greedy_route_ref) > T {
                                debug!("Bound: Route cost: {}", cost(&h, &greedy_route_ref));
                                tsp_bound = true;
                            } else {
                                riders_routes.insert(*r.0, greedy_route);
                            }
                        }
                    }
                }
                if val > 0.0 && locations.is_empty() && !tsp_bound {
                    self.best_difference = val;
                    info!("New best: {} (Relaxed optimum: {})", val, self.opt_relax);
                    self.best_riders = riders.clone();
                    self.best_routes = riders_routes.clone();
                    info!("New best partition: {:?}", riders);
                    info!("Sums: ");
                    for r in &riders {
                        info!("Rider #{}: {}", r.0, r.1.sum_tips());
                        let mut r_orders = r.1.orders.keys().collect::<Vec<_>>();
                        r_orders.insert(0, &NUM_LOCATIONS);
                        r_orders.push(&NUM_LOCATIONS);
                        let h = self.graph.filter_map(
                            |_, nw| Some(nw),
                            |e, ew| {
                                if let Some((n1, n2)) = self.graph.edge_endpoints(e) {
                                    if r_orders.contains(&&n1.index())
                                        && r_orders.contains(&&n2.index())
                                    {
                                        return Some(ew);
                                    }
                                }
                                None
                            },
                        );
                        info!("Route: {:?}", riders_routes.get(r.0).unwrap());
                        let route_ref = riders_routes
                            .get(r.0)
                            .unwrap()
                            .iter()
                            .map(|x| x)
                            .collect::<Vec<_>>();
                        info!("TSP: {}", cost(&h, &route_ref));
                    }

                    if val - self.opt_relax == 0.0 {
                        self.finished = true;
                    }
                }
                if !locations.is_empty() {
                    let mut next_locations = vec![];
                    let next_tip = locations_sorted[0];
                    for l in locations_sorted.iter() {
                        if (l.1 - next_tip.1).abs() < ERROR {
                            next_locations.push(l.0);
                        }
                    }

                    let mut different_sum_riders_indexes = HashMap::new();
                    for r in riders_sorted.iter() {
                        if !different_sum_riders_indexes
                            .values()
                            .any(|x| (x - r.1.sum_tips()).abs() < ERROR)
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
                            updated_orders.extend(near_orders.keys());
                            updated_orders.push(*l);
                            let h = self.graph.filter_map(
                                |_, nw| Some(nw),
                                |e, ew| {
                                    if let Some((n1, n2)) = self.graph.edge_endpoints(e) {
                                        if updated_orders.contains(&&n1.index())
                                            && updated_orders.contains(&&n2.index())
                                        {
                                            return Some(ew);
                                        }
                                    }
                                    None
                                },
                            );
                            let greedy_route = nearest_neighbor(&h, &updated_orders);
                            let greedy_route_ref =
                                greedy_route.iter().map(|x| x).collect::<Vec<_>>();
                            let route_cost = cost(&h, &greedy_route_ref);
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

    let mut graph = Graph::<_, _>::new();
    let t_range = (MIN_T as u32..MAX_T as u32).collect::<Vec<_>>();
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

    let tip_range = (MIN_TIP as u32..MAX_TIP as u32).collect::<Vec<_>>();
    let mut locations = HashMap::new();
    let mut riders = HashMap::new();

    for i in 0..NUM_LOCATIONS {
        locations.insert(i, *tip_range.choose(&mut rng).unwrap() as f64);
    }
    info!("Locations: {:?}", locations);

    for i in 0..NUM_RIDERS {
        riders.insert(i, Rider::new());
    }

    let mut locations_sorted = locations.values().collect::<Vec<_>>();
    locations_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    locations_sorted.reverse();
    info!("Locations Sorted: {:?}", locations_sorted);

    let locations_values_sum = locations.values().sum::<f64>();
    info!("Tot: {:?}", locations_values_sum);

    let max_opt_relax = (locations_values_sum / NUM_RIDERS as f64).ceil();
    let opt_relax =
        max_opt_relax - (locations_values_sum - max_opt_relax) / (NUM_RIDERS as f64 - 1.0);
    info!("Z* Relaxed: {:?}", opt_relax);

    let mut cga_tree = CGATree::new(graph, locations_values_sum, opt_relax);
    cga_tree.search(locations, riders);

    dbg!(cga_tree.nodes_number);
}
