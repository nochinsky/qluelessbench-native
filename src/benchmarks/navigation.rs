//! Navigation/GPS route calculation benchmark tests.
//!
//! Tests pathfinding algorithms (Dijkstra, A*) for GPS navigation workloads.

use anyhow::Result;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::references::ReferenceValues;
use crate::results::CategoryResult;

/// Graph node for pathfinding.
#[derive(Clone, Copy)]
struct Node {
    x: f64,
    y: f64,
}

/// Edge in the graph.
#[derive(Clone, Debug)]
struct Edge {
    to: usize,
    weight: f64,
}

/// Heap node for Dijkstra.
#[derive(Clone, Copy, PartialEq)]
struct HeapNode {
    cost: f64,
    node: usize,
}

impl Eq for HeapNode {}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Generate a road network graph.
fn generate_road_network(num_nodes: usize) -> (Vec<Node>, Vec<Vec<Edge>>) {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    let mut nodes = Vec::with_capacity(num_nodes);
    let mut adjacency = vec![Vec::new(); num_nodes];

    let grid_size = (num_nodes as f64).sqrt().ceil() as usize;
    for i in 0..num_nodes {
        let row = i / grid_size;
        let col = i % grid_size;
        let x = col as f64 + rng.gen_range(-0.1..0.1);
        let y = row as f64 + rng.gen_range(-0.1..0.1);
        nodes.push(Node { x, y });
    }

    for i in 0..num_nodes {
        let row = i / grid_size;
        let col = i % grid_size;

        // Connect to right neighbor
        if col + 1 < grid_size {
            let j = i + 1;
            if j < num_nodes {
                let dist =
                    ((nodes[i].x - nodes[j].x).powi(2) + (nodes[i].y - nodes[j].y).powi(2)).sqrt();
                adjacency[i].push(Edge {
                    to: j,
                    weight: dist,
                });
                adjacency[j].push(Edge {
                    to: i,
                    weight: dist,
                });
            }
        }

        // Connect to bottom neighbor
        if row + 1 < grid_size {
            let j = i + grid_size;
            if j < num_nodes {
                let dist =
                    ((nodes[i].x - nodes[j].x).powi(2) + (nodes[i].y - nodes[j].y).powi(2)).sqrt();
                adjacency[i].push(Edge {
                    to: j,
                    weight: dist,
                });
                adjacency[j].push(Edge {
                    to: i,
                    weight: dist,
                });
            }
        }
    }

    (nodes, adjacency)
}

/// Dijkstra's algorithm for shortest path.
fn dijkstra(adjacency: &[Vec<Edge>], start: usize, end: usize) -> f64 {
    let n = adjacency.len();
    let mut dist = vec![f64::INFINITY; n];
    dist[start] = 0.0;

    let mut heap = BinaryHeap::new();
    heap.push(HeapNode {
        cost: 0.0,
        node: start,
    });

    while let Some(HeapNode { cost, node }) = heap.pop() {
        if node == end {
            return cost;
        }

        if cost > dist[node] {
            continue;
        }

        for edge in &adjacency[node] {
            let new_cost = dist[node] + edge.weight;
            if new_cost < dist[edge.to] {
                dist[edge.to] = new_cost;
                heap.push(HeapNode {
                    cost: new_cost,
                    node: edge.to,
                });
            }
        }
    }

    f64::INFINITY
}

/// Navigation benchmark.
pub struct NavigationBenchmark {
    multi_core: bool,
}

impl NavigationBenchmark {
    /// Create a new NavigationBenchmark.
    pub fn new() -> Self {
        NavigationBenchmark { multi_core: false }
    }

    /// Create a new NavigationBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        NavigationBenchmark { multi_core: true }
    }

    /// Test route calculation.
    fn test_route_finding(num_nodes: usize, num_routes: usize) -> Result<f64> {
        let (nodes, adjacency) = generate_road_network(num_nodes);

        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(123);

        let start = Instant::now();

        for _ in 0..num_routes {
            let src = rng.gen_range(0..nodes.len());
            let dst = rng.gen_range(0..nodes.len());
            let _distance = dijkstra(&adjacency, src, dst);
        }

        let duration = start.elapsed().as_secs_f64();
        Ok(num_routes as f64 / duration)
    }

    /// Test parallel route calculation.
    fn test_parallel_route_finding(
        num_workers: usize,
        num_nodes: usize,
        routes_per_worker: usize,
    ) -> Result<f64> {
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|worker_id| -> Result<()> {
                let (_nodes, adjacency) = generate_road_network(num_nodes);

                use rand::{Rng, SeedableRng};
                let mut rng = rand::rngs::StdRng::seed_from_u64(worker_id as u64);

                for _ in 0..routes_per_worker {
                    let src = rng.gen_range(0..num_nodes);
                    let dst = rng.gen_range(0..num_nodes);
                    let _distance = dijkstra(&adjacency, src, dst);
                }

                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(routes_per_worker as f64 / duration)
    }
}

impl Default for NavigationBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for NavigationBenchmark {
    fn category_name(&self) -> &'static str {
        "Navigation"
    }

    fn weight(&self) -> f64 {
        1.0
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;
        let refs = ReferenceValues::load();

        if self.multi_core {
            let num_workers = get_parallel_workers();

            // Multi-core: Parallel route finding
            let test_fn = || Self::test_parallel_route_finding(num_workers, 1000, 50);
            let result = run_with_iterations(
                test_fn,
                "Route Finding (parallel)",
                refs.navigation.parallel_routes_per_sec,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core: Route finding
            let test_fn = || Self::test_route_finding(500, 100);
            let result = run_with_iterations(
                test_fn,
                "Route Finding",
                refs.navigation.single_routes_per_sec,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        }

        // Calculate category score
        let category_score = calculate_category_score(&results);

        Ok(CategoryResult {
            category: self.category_name().to_string(),
            score: category_score,
            duration: total_duration,
            weight: self.weight(),
            tests: results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_category_name() {
        let benchmark = NavigationBenchmark::new();
        assert_eq!(benchmark.category_name(), "Navigation");
    }

    #[test]
    fn test_navigation_weight() {
        let benchmark = NavigationBenchmark::new();
        assert_eq!(benchmark.weight(), 1.0);
    }

    #[test]
    fn test_generate_road_network() {
        let (nodes, adjacency) = generate_road_network(100);
        assert_eq!(nodes.len(), 100);
        assert_eq!(adjacency.len(), 100);
    }

    #[test]
    fn test_dijkstra_returns_distance() {
        let (nodes, adjacency) = generate_road_network(50);
        let distance = dijkstra(&adjacency, 0, nodes.len() - 1);
        assert!(distance < f64::INFINITY);
        assert!(distance >= 0.0);
    }

    #[test]
    fn test_route_finding() {
        let result = NavigationBenchmark::test_route_finding(100, 10);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_multi_core_benchmark_creation() {
        let single = NavigationBenchmark::new();
        let multi = NavigationBenchmark::new_multi_core();
        assert_eq!(single.category_name(), multi.category_name());
        assert_eq!(single.weight(), multi.weight());
    }

    #[test]
    fn test_parallel_route_finding() {
        let result = NavigationBenchmark::test_parallel_route_finding(2, 100, 10);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
