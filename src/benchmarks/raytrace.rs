//! Ray tracing benchmark tests.
//!
//! Implements a simple path tracer for CPU ray tracing performance.

use anyhow::Result;
use rayon::prelude::*;
use std::time::Instant;

use crate::benchmarks::base::{calculate_category_score, run_with_iterations, BaseBenchmark};
use crate::results::CategoryResult;

/// Simple vector type.
#[derive(Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    fn dot(self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn normalize(self) -> Vec3 {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        let len = len.max(f64::EPSILON);
        Vec3::new(self.x / len, self.y / len, self.z / len)
    }

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/// Sphere type.
struct Sphere {
    center: Vec3,
    radius: f64,
}

impl Sphere {
    fn intersect(&self, origin: Vec3, dir: Vec3) -> Option<f64> {
        let oc = origin.sub(self.center);
        let b = oc.dot(dir);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - c;

        if discriminant > 0.0 {
            let t = -b - discriminant.sqrt();
            if t > 0.001 {
                return Some(t);
            }
        }
        None
    }
}

/// Trace a single ray.
fn trace_ray(origin: Vec3, dir: Vec3, spheres: &[Sphere]) -> f64 {
    let mut closest_t = f64::INFINITY;
    let mut hit_sphere = false;

    for sphere in spheres {
        if let Some(t) = sphere.intersect(origin, dir) {
            if t < closest_t {
                closest_t = t;
                hit_sphere = true;
            }
        }
    }

    if hit_sphere {
        let hit_point = Vec3::new(
            origin.x + dir.x * closest_t,
            origin.y + dir.y * closest_t,
            origin.z + dir.z * closest_t,
        );
        ((hit_point.y + 1.0) * 0.5).clamp(0.0, 1.0)
    } else {
        0.5 * (dir.y + 1.0)
    }
}

/// Render a single row of pixels.
fn render_row(row: usize, width: usize, height: usize, spheres: &[Sphere]) -> Vec<f64> {
    let mut pixels = Vec::with_capacity(width);
    let aspect_ratio = width as f64 / height as f64;

    for col in 0..width {
        let u = (2.0 * col as f64 / width as f64 - 1.0) * aspect_ratio;
        let v = 1.0 - 2.0 * row as f64 / height as f64;
        let dir = Vec3::new(u, v, -1.0).normalize();
        let origin = Vec3::new(0.0, 0.0, 0.0);

        let color = trace_ray(origin, dir, spheres);
        pixels.push(color);
    }
    pixels
}

/// Ray tracing benchmark.
pub struct RayTracingBenchmark {
    multi_core: bool,
}

impl RayTracingBenchmark {
    /// Create a new RayTracingBenchmark.
    pub fn new() -> Self {
        RayTracingBenchmark { multi_core: false }
    }

    /// Create a new RayTracingBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        RayTracingBenchmark { multi_core: true }
    }

    /// Test ray tracing performance (single-core, sequential).
    fn test_ray_trace_sequential(width: usize, height: usize) -> Result<f64> {
        let spheres = vec![
            Sphere {
                center: Vec3::new(0.0, 0.0, -3.0),
                radius: 0.5,
            },
            Sphere {
                center: Vec3::new(0.5, 0.2, -2.5),
                radius: 0.3,
            },
            Sphere {
                center: Vec3::new(-0.5, -0.3, -2.0),
                radius: 0.4,
            },
        ];

        let start = Instant::now();

        let _pixels: Vec<Vec<f64>> = (0..height)
            .map(|row| render_row(row, width, height, &spheres))
            .collect();

        let duration = start.elapsed().as_secs_f64();
        let total_pixels = width * height;

        Ok(total_pixels as f64 / 1_000_000.0 / duration)
    }

    /// Test ray tracing performance (multi-core, parallel).
    fn test_ray_trace(width: usize, height: usize) -> Result<f64> {
        let spheres = vec![
            Sphere {
                center: Vec3::new(0.0, 0.0, -3.0),
                radius: 0.5,
            },
            Sphere {
                center: Vec3::new(0.5, 0.2, -2.5),
                radius: 0.3,
            },
            Sphere {
                center: Vec3::new(-0.5, -0.3, -2.0),
                radius: 0.4,
            },
        ];

        let start = Instant::now();

        let _pixels: Vec<Vec<f64>> = (0..height)
            .into_par_iter()
            .map(|row| render_row(row, width, height, &spheres))
            .collect();

        let duration = start.elapsed().as_secs_f64();
        let total_pixels = width * height;

        Ok(total_pixels as f64 / 1_000_000.0 / duration)
    }
}

impl Default for RayTracingBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for RayTracingBenchmark {
    fn category_name(&self) -> &'static str {
        "Ray Tracing"
    }

    fn weight(&self) -> f64 {
        1.5
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Reference values (megapixels/second)
        let single_core_ref = 2.0; // 2 MP/s for single-core
        let multi_core_ref = 20.0; // 20 MP/s for multi-core (10x)

        if self.multi_core {
            // Multi-core: Higher resolution with parallel rendering
            let test_fn = || Self::test_ray_trace(1024, 1024);
            let result = run_with_iterations(
                test_fn,
                "Ray Tracing (1024x1024)",
                multi_core_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core: Lower resolution with sequential rendering
            let test_fn = || Self::test_ray_trace_sequential(512, 512);
            let result = run_with_iterations(
                test_fn,
                "Ray Tracing (512x512)",
                single_core_ref,
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
    fn test_raytrace_category_name() {
        let benchmark = RayTracingBenchmark::new();
        assert_eq!(benchmark.category_name(), "Ray Tracing");
    }

    #[test]
    fn test_raytrace_weight() {
        let benchmark = RayTracingBenchmark::new();
        assert_eq!(benchmark.weight(), 1.5);
    }

    #[test]
    fn test_vec3_creation() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vec3_dot_product() {
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(v1.dot(v2), 0.0);
    }

    #[test]
    fn test_sphere_intersection() {
        let sphere = Sphere {
            center: Vec3::new(0.0, 0.0, -3.0),
            radius: 1.0,
        };
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, -1.0);
        let result = sphere.intersect(origin, dir);
        assert!(result.is_some());
    }

    #[test]
    fn test_trace_ray() {
        let sphere = Sphere {
            center: Vec3::new(0.0, 0.0, -3.0),
            radius: 1.0,
        };
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, -1.0);
        let color = trace_ray(origin, dir, &[sphere]);
        assert!(color >= 0.0 && color <= 1.0);
    }

    #[test]
    fn test_render_row() {
        let sphere = Sphere {
            center: Vec3::new(0.0, 0.0, -3.0),
            radius: 1.0,
        };
        let pixels = render_row(0, 64, 64, &[sphere]);
        assert_eq!(pixels.len(), 64);
    }

    #[test]
    fn test_multi_core_benchmark_creation() {
        let single = RayTracingBenchmark::new();
        let multi = RayTracingBenchmark::new_multi_core();
        assert_eq!(single.category_name(), multi.category_name());
        assert_eq!(single.weight(), multi.weight());
    }

    #[test]
    fn test_ray_trace_sequential() {
        let result = RayTracingBenchmark::test_ray_trace_sequential(64, 64);
        assert!(result.is_ok());
        assert!(result.unwrap() >= 0.0);
    }

    #[test]
    fn test_ray_trace() {
        let result = RayTracingBenchmark::test_ray_trace(64, 64);
        assert!(result.is_ok());
        assert!(result.unwrap() >= 0.0);
    }
}
