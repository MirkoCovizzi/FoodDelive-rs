use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Rider {
    id: u32,
    pub orders: HashMap<u32, f64>,
    pub route: Option<Vec<u32>>,
}

impl Rider {
    pub fn new(id: u32) -> Rider {
        Rider {
            id,
            orders: HashMap::new(),
            route: None,
        }
    }

    pub fn add_tip(&mut self, location: u32, tip: f64) {
        self.orders.insert(location, tip);
    }

    pub fn sum_tips(&self) -> f64 {
        self.orders
            .keys()
            .map(|&k| self.orders.get(&k).unwrap())
            .sum()
    }
}

impl Ord for Rider {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sum_tips().partial_cmp(&other.sum_tips()).unwrap()
    }
}

impl Eq for Rider {}

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
        let mut rider = Rider::new(0);
        rider.add_tip(0, 9.0);
        rider.add_tip(1, 10.0);

        assert_eq!(rider.sum_tips(), 19.0);
    }

    #[test]
    fn sum_tips_override_tip() {
        let mut rider = Rider::new(0);
        rider.add_tip(0, 9.0);
        rider.add_tip(0, 8.0);
        rider.add_tip(1, 10.0);

        assert_eq!(rider.sum_tips(), 18.0);
    }

    #[test]
    fn partial_ord() {
        let mut rider_1 = Rider::new(0);
        rider_1.add_tip(0, 9.0);
        rider_1.add_tip(1, 10.0);

        let mut rider_2 = Rider::new(1);
        rider_2.add_tip(0, 5.0);
        rider_2.add_tip(1, 10.0);

        assert_eq!(rider_2 < rider_1, true);
        assert_eq!(rider_1 > rider_2, true);
        assert_eq!(rider_1 == rider_2, false);
    }
}
