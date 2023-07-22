use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

pub type Region = &'static str;

pub struct Cluster {
    nodes_in_region: HashMap<Region, usize>,
    latency_by_edge: HashMap<(Region, Region), Duration>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Destination {
    Leader,
    Follower(Region),
}

impl fmt::Display for Destination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Destination::Leader => write!(f, "Leader"),
            Destination::Follower(region) => write!(f, "Follower ({})", region),
        }
    }
}

impl Cluster {
    pub fn new(regions: Vec<(Region, usize)>, latency: Vec<(Region, Region, Duration)>) -> Self {
        let mut latency_by_edge = HashMap::new();
        for (src, dst, duration) in latency {
            latency_by_edge.insert((src, dst), duration);
            latency_by_edge.insert((dst, src), duration);
        }
        Self {
            nodes_in_region: HashMap::from_iter(regions),
            latency_by_edge,
        }
    }

    pub fn p51_latency_from_leader(&self, leader_region: Region) -> Duration {
        // Stack rank the followers by latency from leader.
        let mut possible_latencies = Vec::new();
        for destination in self.destinations(leader_region) {
            if let Destination::Follower(region) = destination {
                possible_latencies
                    .push(self.latency_by_edge.get(&(leader_region, region)).unwrap());
            }
        }
        possible_latencies.sort();
        // Pick the slowest of the majority, exlcuding the leader.
        let majority_minus_one = self.nodes_in_region.values().sum::<usize>() / 2;
        *possible_latencies[majority_minus_one - 1]
    }

    pub fn client_response_time(
        &self,
        client_region: Region,
        destination: Destination,
        leader_region: Region,
    ) -> Duration {
        let leader_p51 = self.p51_latency_from_leader(leader_region);
        let client_to_destination = match destination {
            Destination::Leader => *self
                .latency_by_edge
                .get(&(client_region, leader_region))
                .unwrap(),
            Destination::Follower(follower_region) => *self
                .latency_by_edge
                .get(&(client_region, follower_region))
                .unwrap(),
        };
        match destination {
            Destination::Leader => client_to_destination * 2 + leader_p51 * 2,
            Destination::Follower(follower_region) => {
                let follower_to_leader = *self
                    .latency_by_edge
                    .get(&(follower_region, leader_region))
                    .unwrap();
                client_to_destination * 2 + follower_to_leader * 2 + leader_p51 * 2
            }
        }
    }

    pub fn regions(&self) -> Vec<Region> {
        self.nodes_in_region.keys().cloned().collect()
    }

    pub fn destinations(&self, leader_region: Region) -> Vec<Destination> {
        let mut destinations = Vec::new();
        destinations.push(Destination::Leader);
        for (&region, &count) in &self.nodes_in_region {
            let mut followers_in_region = count;
            if region == leader_region {
                followers_in_region -= 1;
            }
            for _ in 0..followers_in_region {
                destinations.push(Destination::Follower(region));
            }
        }
        destinations
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use once_cell::sync::Lazy;

    static CLUSTER: Lazy<Cluster> = Lazy::new(|| {
        Cluster::new(
            vec![("A", 2), ("B", 2), ("C", 1)],
            vec![
                ("A", "A", Duration::from_millis(1)),
                ("B", "B", Duration::from_millis(2)),
                ("C", "C", Duration::from_millis(3)),
                ("A", "B", Duration::from_millis(4)),
                ("A", "C", Duration::from_millis(5)),
                ("B", "C", Duration::from_millis(6)),
            ],
        )
    });

    #[test]
    fn test_new() {
        assert_eq!(CLUSTER.nodes_in_region.get("A"), Some(&2));
        assert_eq!(CLUSTER.nodes_in_region.get("B"), Some(&2));
        assert_eq!(CLUSTER.nodes_in_region.get("C"), Some(&1));

        assert_eq!(
            CLUSTER.latency_by_edge.get(&("A", "A")),
            Some(&Duration::from_millis(1))
        );
        assert_eq!(
            CLUSTER.latency_by_edge.get(&("B", "B")),
            Some(&Duration::from_millis(2))
        );
        assert_eq!(
            CLUSTER.latency_by_edge.get(&("C", "C")),
            Some(&Duration::from_millis(3))
        );
        assert_eq!(
            CLUSTER.latency_by_edge.get(&("A", "B")),
            Some(&Duration::from_millis(4))
        );
        assert_eq!(
            CLUSTER.latency_by_edge.get(&("B", "A")),
            Some(&Duration::from_millis(4))
        );
        assert_eq!(
            CLUSTER.latency_by_edge.get(&("A", "C")),
            Some(&Duration::from_millis(5))
        );
        assert_eq!(
            CLUSTER.latency_by_edge.get(&("C", "A")),
            Some(&Duration::from_millis(5))
        );
        assert_eq!(
            CLUSTER.latency_by_edge.get(&("B", "C")),
            Some(&Duration::from_millis(6))
        );
        assert_eq!(
            CLUSTER.latency_by_edge.get(&("C", "B")),
            Some(&Duration::from_millis(6))
        );
    }

    #[test]
    fn test_p51_latency_from_leader() {
        assert_eq!(
            CLUSTER.p51_latency_from_leader("A"),
            Duration::from_millis(4)
        );
        assert_eq!(
            CLUSTER.p51_latency_from_leader("B"),
            Duration::from_millis(4)
        );
        assert_eq!(
            CLUSTER.p51_latency_from_leader("C"),
            Duration::from_millis(5)
        );
    }

    #[test]
    fn test_client_response_time() {
        assert_eq!(
            CLUSTER.client_response_time("B", Destination::Leader, "B"),
            Duration::from_millis(12)
        );
        assert_eq!(
            CLUSTER.client_response_time("A", Destination::Leader, "B"),
            Duration::from_millis(16)
        );
        assert_eq!(
            CLUSTER.client_response_time("B", Destination::Follower("B"), "B"),
            Duration::from_millis(16)
        );
        assert_eq!(
            CLUSTER.client_response_time("A", Destination::Follower("A"), "B"),
            Duration::from_millis(18)
        );
    }

    #[test]
    fn test_destinations() {
        let mut destinations = CLUSTER.destinations("C");
        destinations.sort();
        assert_eq!(
            destinations,
            vec![
                Destination::Leader,
                Destination::Follower("A"),
                Destination::Follower("A"),
                Destination::Follower("B"),
                Destination::Follower("B"),
            ]
        );
        let mut destinations = CLUSTER.destinations("A");
        destinations.sort();
        assert_eq!(
            destinations,
            vec![
                Destination::Leader,
                Destination::Follower("A"),
                Destination::Follower("B"),
                Destination::Follower("B"),
                Destination::Follower("C"),
            ]
        );
    }
}
