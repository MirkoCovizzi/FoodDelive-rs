extern crate simple_logger;

use std::collections::HashMap;

use rand::seq::SliceRandom;

use crate::rider::Rider;
use log::*;

mod rider;

const NUM_LOCATIONS: u32 = 15;
const NUM_RIDERS: u32 = 5;
const MIN_TIP: u32 = 1;
const MAX_TIP: u32 = 10;

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

            let mut locations_sorted = locations.values().collect::<Vec<_>>();
            locations_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            locations_sorted.reverse();

            let mut riders_sorted = riders.values().collect::<Vec<_>>();
            riders_sorted.sort();

            let mut sum_riders = 0.0;
            for r in riders_sorted[..riders_sorted.len() - 1].iter() {
                sum_riders += r.sum_tips();
            }

            let locations_values_sum = locations.values().sum::<f64>();

            let sum_tot = sum_riders + locations_values_sum;

            let max_rider = riders_sorted.last().unwrap().sum_tips();

            let val = max_rider - (sum_tot / (NUM_RIDERS - 1) as f64);

            let _max_one_tree_max = 0.0;
            let _one_tree_bound = false;

            if riders_sorted.last().unwrap().orders.keys().len() >= 4 && val < self.best_difference
            {
                info!("First branch!");
            }
        }
    }
}

fn main() {
    simple_logger::init().expect("Can't initialize logging.");

    let range = (MIN_TIP..MAX_TIP).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();
    let mut locations = HashMap::new();
    let mut riders = HashMap::new();

    for i in 0..NUM_LOCATIONS {
        locations.insert(i, *range.choose(&mut rng).unwrap() as f64);
    }

    for i in 0..NUM_RIDERS {
        let mut rider = Rider::new(i);
        rider.add_tip(NUM_LOCATIONS, 10.0);
        riders.insert(i, rider);
    }

    let mut locations_sorted = locations.values().collect::<Vec<_>>();

    locations_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    locations_sorted.reverse();
    info!("{:?}", locations_sorted);

    let locations_values_sum = locations.values().sum::<f64>();
    info!("Tot: {:?}", locations_values_sum);

    let num_riders_f = NUM_RIDERS as f64;
    let max_opt_relax = (locations_values_sum / num_riders_f).ceil();
    let opt_relax = max_opt_relax - (locations_values_sum - max_opt_relax) / (num_riders_f - 1.0);
    info!("Z* Relaxed: {:?}", opt_relax);

    let mut cga_tree = CgaTree::new(locations_values_sum);
    cga_tree.search(locations, riders);
}
