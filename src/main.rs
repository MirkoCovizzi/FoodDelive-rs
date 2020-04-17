use std::collections::HashMap;
use rand::seq::SliceRandom;

mod rider;
mod locations;

const NUM_LOCATIONS: u32 = 15;
const NUM_RIDERS: u32 = 5;
const MIN_TIP: u32 = 1;
const MAX_TIP: u32 = 10;
const MIN_T: u32 = 1;
const MAX_T: u32 = 1;

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

    let max_opt_relax = (locations_values_sum as f32 / NUM_RIDERS as f32)
        .ceil() as u32;
    let opt_relax = max_opt_relax as f32 -
        (locations_values_sum - max_opt_relax) as f32 /
            (NUM_RIDERS - 1) as f32;
    println!("Z* Relaxed: {:?}", opt_relax);
}
