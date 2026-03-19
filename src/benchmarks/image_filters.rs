//! Image filters benchmark tests.
//!
//! Tests image processing filters: blur, sharpen, edge detection.

use anyhow::Result;
use image::{GrayImage, ImageBuffer, Luma};
use rayon::prelude::*;
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::results::CategoryResult;

/// Image filters benchmark.
pub struct ImageFiltersBenchmark {
    multi_core: bool,
}

impl ImageFiltersBenchmark {
    /// Create a new ImageFiltersBenchmark.
    pub fn new() -> Self {
        ImageFiltersBenchmark { multi_core: false }
    }

    /// Create a new ImageFiltersBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        ImageFiltersBenchmark { multi_core: true }
    }

    /// Generate a test image with a pattern.
    fn generate_test_image(width: u32, height: u32) -> GrayImage {
        ImageBuffer::from_fn(width, height, |x, y| {
            let pattern = ((x as i32 + y as i32) % 256) as u8;
            Luma([pattern])
        })
    }

    /// Apply Gaussian blur filter.
    fn apply_blur(img: &GrayImage, sigma: f32) -> GrayImage {
        let kernel_size = (sigma * 6.0).ceil() as usize | 1; // Ensure odd
        let half = kernel_size / 2;

        // Create Gaussian kernel
        let kernel: Vec<f32> = (0..kernel_size)
            .map(|i| {
                let x = (i as i32 - half as i32) as f32;
                (-x * x / (2.0 * sigma * sigma)).exp()
            })
            .collect();

        let kernel_sum: f32 = kernel.iter().sum();
        let kernel: Vec<f32> = kernel.iter().map(|&k| k / kernel_sum).collect();

        let (width, height) = img.dimensions();
        let mut result = GrayImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0f32;
                let mut weight_sum = 0.0f32;

                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let nx =
                            (x as i32 + kx as i32 - half as i32).clamp(0, width as i32 - 1) as u32;
                        let ny =
                            (y as i32 + ky as i32 - half as i32).clamp(0, height as i32 - 1) as u32;
                        let weight = kernel[kx] * kernel[ky];
                        sum += img.get_pixel(nx, ny)[0] as f32 * weight;
                        weight_sum += weight;
                    }
                }

                result.put_pixel(x, y, Luma([(sum / weight_sum) as u8]));
            }
        }

        result
    }

    /// Apply Sobel edge detection.
    fn apply_edge_detect(img: &GrayImage) -> GrayImage {
        let (width, height) = img.dimensions();
        let mut result = GrayImage::new(width, height);

        // Sobel kernels
        let sobel_x = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
        let sobel_y = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut gx = 0i32;
                let mut gy = 0i32;

                for ky in 0..3usize {
                    for kx in 0..3usize {
                        let pixel = img.get_pixel(x + kx as u32 - 1, y + ky as u32 - 1)[0] as i32;
                        gx += pixel * sobel_x[ky][kx];
                        gy += pixel * sobel_y[ky][kx];
                    }
                }

                let magnitude = ((gx * gx + gy * gy) as f32).sqrt().min(255.0) as u8;
                result.put_pixel(x, y, Luma([magnitude]));
            }
        }

        result
    }

    /// Apply sharpen filter.
    fn apply_sharpen(img: &GrayImage) -> GrayImage {
        let (width, height) = img.dimensions();
        let mut result = GrayImage::new(width, height);

        // Sharpen kernel
        let kernel = [[0, -1, 0], [-1, 5, -1], [0, -1, 0]];

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut sum = 0i32;

                for (ky, row) in kernel.iter().enumerate() {
                    for (kx, &k) in row.iter().enumerate() {
                        let pixel = img.get_pixel(x + kx as u32 - 1, y + ky as u32 - 1)[0] as i32;
                        sum += pixel * k;
                    }
                }

                result.put_pixel(x, y, Luma([sum.clamp(0, 255) as u8]));
            }
        }

        result
    }

    /// Test blur performance.
    fn test_blur(width: u32, height: u32, sigma: f32) -> Result<f64> {
        let img = Self::generate_test_image(width, height);

        let start = Instant::now();
        let _blurred = Self::apply_blur(&img, sigma);
        let duration = start.elapsed().as_secs_f64();

        Ok((width * height) as f64 / 1_000_000.0 / duration)
    }

    /// Test edge detection performance.
    fn test_edge_detect(width: u32, height: u32) -> Result<f64> {
        let img = Self::generate_test_image(width, height);

        let start = Instant::now();
        let _edges = Self::apply_edge_detect(&img);
        let duration = start.elapsed().as_secs_f64();

        Ok((width * height) as f64 / 1_000_000.0 / duration)
    }

    /// Test sharpen performance.
    fn test_sharpen(width: u32, height: u32) -> Result<f64> {
        let img = Self::generate_test_image(width, height);

        let start = Instant::now();
        let _sharpened = Self::apply_sharpen(&img);
        let duration = start.elapsed().as_secs_f64();

        Ok((width * height) as f64 / 1_000_000.0 / duration)
    }

    /// Test parallel filter application.
    fn test_parallel_filters(num_images: usize, width: u32, height: u32) -> Result<f64> {
        let start = Instant::now();

        (0..num_images)
            .into_par_iter()
            .try_for_each(|i| -> Result<()> {
                let img = Self::generate_test_image(width, height);
                match i % 3 {
                    0 => {
                        let _ = Self::apply_blur(&img, 2.0);
                    }
                    1 => {
                        let _ = Self::apply_edge_detect(&img);
                    }
                    _ => {
                        let _ = Self::apply_sharpen(&img);
                    }
                }
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        Ok((num_images * width as usize * height as usize) as f64 / 1_000_000.0 / duration)
    }
}

impl Default for ImageFiltersBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for ImageFiltersBenchmark {
    fn category_name(&self) -> &'static str {
        "Image Filters"
    }

    fn weight(&self) -> f64 {
        1.3
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Reference values (megapixels/second)
        let blur_ref = 10.0;
        let edge_ref = 20.0;
        let sharpen_ref = 15.0;
        let parallel_ref = 50.0;

        if self.multi_core {
            let num_workers = get_parallel_workers();

            // Multi-core: Parallel filters
            let test_fn = || Self::test_parallel_filters(num_workers, 512, 512);
            let result = run_with_iterations(
                test_fn,
                "Image Filters (parallel)",
                parallel_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core: Individual filters
            let test_fn = || Self::test_blur(1024, 1024, 2.0);
            let result = run_with_iterations(
                test_fn,
                "Gaussian Blur",
                blur_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_edge_detect(1024, 1024);
            let result = run_with_iterations(
                test_fn,
                "Edge Detection",
                edge_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_sharpen(1024, 1024);
            let result =
                run_with_iterations(test_fn, "Sharpen", sharpen_ref, iterations, warmup, timeout);
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
