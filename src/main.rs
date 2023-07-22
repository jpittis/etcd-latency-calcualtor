mod cluster;

use cluster::{Cluster, Destination, Region};
use std::time::Duration;

fn main() {
    let five_node_single_region =
        Cluster::new(vec![("A", 5)], vec![("A", "A", Duration::from_millis(5))]);

    let nine_node_multi_region = Cluster::new(
        vec![("A", 3), ("B", 3), ("C", 3)],
        vec![
            ("A", "A", Duration::from_millis(5)),
            ("B", "B", Duration::from_millis(5)),
            ("C", "C", Duration::from_millis(5)),
            ("B", "C", Duration::from_millis(20)),
            ("A", "B", Duration::from_millis(60)),
            ("C", "A", Duration::from_millis(60)),
        ],
    );

    println!("## Usage");
    println!("cargo run > README.md");

    calculate("Five Node Single Region", &five_node_single_region);
    calculate("Nine Node Multi Region", &nine_node_multi_region);
}

fn calculate(title: &str, cluster: &Cluster) {
    println!("## {}", title);
    let unique = unique_paths(cluster);
    let all = all_paths(cluster);
    let leader: Vec<Path> = all
        .iter()
        .cloned()
        .filter(|(_, destination, _, _)| *destination == Destination::Leader)
        .collect();
    let local: Vec<Path> = all
        .iter()
        .cloned()
        .filter(
            |(client_region, destination, leader_region, _)| match destination {
                Destination::Leader => client_region == leader_region,
                Destination::Follower(region) => client_region == region,
            },
        )
        .collect();
    let all = all_paths(cluster);

    println!("Client Region | Destination | Leader Region | Response Time |");
    println!("------------- | ----------- | ------------- | ------------- |");
    print_paths(&unique);
    println!();
    println!("Strategy | Min Response Time | Max Response Time | Mean Response Time");
    println!("-------- | ----------------- | ----------------- | ------------------");
    print_avg_paths("Leader", &leader);
    print_avg_paths("Local", &local);
    print_avg_paths("Global", &all);
    println!();
}

type Path = (Region, Destination, Region, Duration);

fn all_paths(cluster: &Cluster) -> Vec<Path> {
    let mut paths = Vec::new();
    for client_region in cluster.regions() {
        for leader_region in cluster.regions() {
            for destination in cluster.destinations(leader_region) {
                let duration =
                    cluster.client_response_time(client_region, destination, leader_region);
                paths.push((client_region, destination, leader_region, duration))
            }
        }
    }
    paths
}

fn avg_paths(paths: &Vec<Path>) -> (Duration, Duration, Duration) {
    let sum: Duration = paths.iter().map(|(_, _, _, duration)| duration).sum();
    let min: Duration = *paths
        .iter()
        .map(|(_, _, _, duration)| duration)
        .min()
        .unwrap();
    let max: Duration = *paths
        .iter()
        .map(|(_, _, _, duration)| duration)
        .max()
        .unwrap();
    let avg = sum / paths.len() as u32;
    (min, max, avg)
}

fn unique_paths(cluster: &Cluster) -> Vec<Path> {
    let mut paths = all_paths(cluster);
    paths.sort();
    paths.dedup();
    paths
}

fn print_paths(paths: &Vec<Path>) {
    for path in paths {
        println!(
            "{} | {} | {} | {:?}",
            path.0,
            path.1,
            path.2,
            round_duration_ms(path.3)
        );
    }
}

fn print_avg_paths(strategy: &str, paths: &Vec<Path>) {
    let (min, max, avg) = avg_paths(paths);
    println!(
        "{} | {:?} | {:?} | {:?}",
        strategy,
        round_duration_ms(min),
        round_duration_ms(max),
        round_duration_ms(avg),
    );
}

fn round_duration_ms(duration: Duration) -> Duration {
    Duration::from_millis(duration.as_millis() as u64)
}
