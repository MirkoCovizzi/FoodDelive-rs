use std::collections::HashMap;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Rider {
    orders: HashMap<u32, u32>,
    route: Option<Vec<u32>>
}

impl Rider {
    pub fn new() -> Rider {
        Rider {
            orders: HashMap::new(),
            route: None
        }
    }

    pub fn add_tip(&mut self, location: u32, tip: u32) {
        self.orders.insert(location, tip);
    }

    pub fn sum_tips(&self) -> u32 {
        self.orders.keys()
            .into_iter()
            .map(|&k| self.orders.get(&k).unwrap())
            .sum::<u32>()
    }
}

impl PartialOrd for Rider {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sum_tips().partial_cmp(&other.sum_tips())
    }
}

impl PartialEq for Rider {
    fn eq(&self, other: &Self) -> bool {
        self.sum_tips().eq(&other.sum_tips())
    }
}

impl std::fmt::Display for Rider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.orders)
    }
}

#[cfg(test)]
mod tests {
    use super::Rider;

    #[test]
    fn sum_tips() {
        let mut rider = Rider::new();
        rider.add_tip(0, 9);
        rider.add_tip(1, 10);

        assert_eq!(rider.sum_tips(), 19);
    }

    #[test]
    fn sum_tips_override_tip() {
        let mut rider = Rider::new();
        rider.add_tip(0, 9);
        rider.add_tip(0, 8);
        rider.add_tip(1, 10);

        assert_eq!(rider.sum_tips(), 18);
    }

    #[test]
    fn partial_ord() {
        let mut rider_1 = Rider::new();
        rider_1.add_tip(0, 9);
        rider_1.add_tip(1, 10);

        let mut rider_2 = Rider::new();
        rider_2.add_tip(0, 5);
        rider_2.add_tip(1, 10);

        assert_eq!(rider_2 < rider_1, true);
        assert_eq!(rider_1 > rider_2, true);
        assert_eq!(rider_1 == rider_2, false);
    }
}