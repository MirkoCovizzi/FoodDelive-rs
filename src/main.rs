use std::collections::HashMap;
use rand::seq::SliceRandom;
use std::ops::Div;

mod rider;
mod locations;

const NUM_LOCATIONS: u32 = 15;
const NUM_RIDERS: u32 = 5;
const MIN_TIP: u32 = 1;
const MAX_TIP: u32 = 10;
const MIN_T: u32 = 1;
const MAX_T: u32 = 100;

fn main() {
    let range = (MIN_TIP..MAX_TIP).collect::<Vec<u32>>();
    let mut rng = rand::thread_rng();
    let mut locations = HashMap::new();

    for i in 0..NUM_LOCATIONS {
        locations.insert(i, range.choose(&mut rng).unwrap());
    }

    let mut locations_sorted = locations.values()
        .map(|x| **x)
        .collect::<Vec<u32>>();

    locations_sorted.sort(); locations_sorted.reverse();
    println!("{:?}", locations_sorted);

    let locations_values_sum = locations.values()
        .map(|x| **x)
        .sum::<u32>();
    println!("Tot: {:?}", locations_values_sum);

    let locations_values_sum_f = locations_values_sum as f32;
    let num_riders_f = NUM_RIDERS as f32;
    let max_opt_relax = (locations_values_sum_f / num_riders_f).ceil();
    let opt_relax = max_opt_relax -
        (locations_values_sum_f - max_opt_relax) /
            (num_riders_f - 1.0);
    println!("Z* Relaxed: {:?}", opt_relax);
}
