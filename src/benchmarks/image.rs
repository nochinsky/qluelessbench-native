//! Image processing benchmark tests.
//!
//! Tests resize, blur/sharpen filters, and format conversion.

use anyhow::Result;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
use rayon::prelude::*;
use std::io::Cursor;
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::results::CategoryResult;

/// Image processing benchmark.
pub struct ImageProcessingBenchmark {
    /// If true, run tests in parallel (multi-core mode)
    multi_core: bool,
}

impl ImageProcessingBenchmark {
    /// Create a new ImageProcessingBenchmark.
    pub fn new() -> Self {
        ImageProcessingBenchmark { multi_core: false }
    }

    /// Create a new ImageProcessingBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        ImageProcessingBenchmark { multi_core: true }
    }

    /// Generate a test image.
    fn generate_test_image(width: u32, height: u32) -> DynamicImage {
        let mut img = ImageBuffer::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = ((x + y) % 256) as u8;
            let g = ((x * 2) % 256) as u8;
            let b = ((y * 2) % 256) as u8;
            *pixel = Rgb([r, g, b]);
        }
        DynamicImage::ImageRgb8(img)
    }

    /// Test resize operations.
    fn test_resize(img: &DynamicImage) -> Result<f64> {
        let start = Instant::now();
        let _resized = img.resize(512, 512, image::imageops::FilterType::Lanczos3);
        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test blur filter.
    fn test_blur(img: &DynamicImage) -> Result<f64> {
        let start = Instant::now();
        let _blurred = img.blur(5.0);
        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test sharpen filter.
    fn test_sharpen(img: &DynamicImage) -> Result<f64> {
        let start = Instant::now();
        let _sharpened = img.filter3x3(&[0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0]);
        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test format conversion (PNG to JPEG).
    fn test_format_conversion(img: &DynamicImage) -> Result<f64> {
        let start = Instant::now();
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Jpeg)?;
        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test parallel resize operations.
    fn test_parallel_resize(images: &[DynamicImage]) -> Result<f64> {
        let start = Instant::now();

        images.par_iter().try_for_each(|img| -> Result<()> {
            let _resized = img.resize(512, 512, image::imageops::FilterType::Lanczos3);
            Ok(())
        })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(images.len() as f64 / duration)
    }

    /// Test parallel blur filter.
    fn test_parallel_blur(images: &[DynamicImage]) -> Result<f64> {
        let start = Instant::now();

        images.par_iter().try_for_each(|img| -> Result<()> {
            let _blurred = img.blur(5.0);
            Ok(())
        })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(images.len() as f64 / duration)
    }

    /// Test parallel sharpen filter.
    fn test_parallel_sharpen(images: &[DynamicImage]) -> Result<f64> {
        let start = Instant::now();

        images.par_iter().try_for_each(|img| -> Result<()> {
            let _sharpened = img.filter3x3(&[0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0]);
            Ok(())
        })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(images.len() as f64 / duration)
    }

    /// Test parallel format conversion.
    fn test_parallel_format_conversion(images: &[DynamicImage]) -> Result<f64> {
        let start = Instant::now();

        images.par_iter().try_for_each(|img| -> Result<()> {
            let mut buffer = Cursor::new(Vec::new());
            img.write_to(&mut buffer, ImageFormat::Jpeg)?;
            Ok(())
        })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(images.len() as f64 / duration)
    }

    /// Generate multiple test images for parallel processing.
    /// Throughput model: Each worker processes a full-size image.
    /// N workers process N images = N× the work.
    fn generate_test_images(width: u32, height: u32, num_images: usize) -> Vec<DynamicImage> {
        (0..num_images)
            .map(|_| Self::generate_test_image(width, height))
            .collect()
    }
}

impl Default for ImageProcessingBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for ImageProcessingBenchmark {
    fn category_name(&self) -> &'static str {
        "Image Processing"
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Generate test image once (2048x2048)
        let test_image = Self::generate_test_image(2048, 2048);

        // Reference values (operations per second - calibrated)
        // Same reference values used for both single-core and multi-core modes
        let (resize_ref, blur_ref, sharpen_ref, format_ref) = (20.0, 10.0, 50.0, 50.0);

        if self.multi_core {
            // Multi-core: process multiple full-size images in parallel (throughput model)
            // N workers process N images of the same size = N× the work
            let num_workers = get_parallel_workers();
            let images = Self::generate_test_images(2048, 2048, num_workers);

            let test_fn = || Self::test_parallel_resize(&images);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Resize ({} images)", num_workers),
                resize_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_blur(&images);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Blur ({} images)", num_workers),
                blur_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_sharpen(&images);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Sharpen ({} images)", num_workers),
                sharpen_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_format_conversion(&images);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Format Conversion ({} images)", num_workers),
                format_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core tests
            // Test 1: Resize
            let test_fn = || Self::test_resize(&test_image);
            let result = run_with_iterations(
                test_fn,
                "Resize Operations",
                resize_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 2: Blur
            let test_fn = || Self::test_blur(&test_image);
            let result = run_with_iterations(
                test_fn,
                "Blur Filter",
                blur_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 3: Sharpen
            let test_fn = || Self::test_sharpen(&test_image);
            let result = run_with_iterations(
                test_fn,
                "Sharpen Filter",
                sharpen_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 4: Format Conversion
            let test_fn = || Self::test_format_conversion(&test_image);
            let result = run_with_iterations(
                test_fn,
                "Format Conversion",
                format_ref,
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
    fn test_image_category_name() {
        let benchmark = ImageProcessingBenchmark::new();
        assert_eq!(benchmark.category_name(), "Image Processing");
    }

    #[test]
    fn test_generate_test_image() {
        let img = ImageProcessingBenchmark::generate_test_image(100, 100);
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);
    }

    #[test]
    fn test_generate_test_images() {
        let images = ImageProcessingBenchmark::generate_test_images(100, 100, 4);
        assert_eq!(images.len(), 4);
        for img in &images {
            assert_eq!(img.width(), 100);
            assert_eq!(img.height(), 100);
        }
    }

    #[test]
    fn test_resize() {
        let img = ImageProcessingBenchmark::generate_test_image(1024, 1024);
        let result = ImageProcessingBenchmark::test_resize(&img);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_blur() {
        let img = ImageProcessingBenchmark::generate_test_image(512, 512);
        let result = ImageProcessingBenchmark::test_blur(&img);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_sharpen() {
        let img = ImageProcessingBenchmark::generate_test_image(512, 512);
        let result = ImageProcessingBenchmark::test_sharpen(&img);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_format_conversion() {
        let img = ImageProcessingBenchmark::generate_test_image(256, 256);
        let result = ImageProcessingBenchmark::test_format_conversion(&img);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_multi_core_benchmark_creation() {
        let single = ImageProcessingBenchmark::new();
        let multi = ImageProcessingBenchmark::new_multi_core();
        assert_eq!(single.category_name(), multi.category_name());
    }

    #[test]
    fn test_parallel_resize() {
        let images = ImageProcessingBenchmark::generate_test_images(512, 512, 4);
        let result = ImageProcessingBenchmark::test_parallel_resize(&images);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_blur() {
        let images = ImageProcessingBenchmark::generate_test_images(256, 256, 4);
        let result = ImageProcessingBenchmark::test_parallel_blur(&images);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_sharpen() {
        let images = ImageProcessingBenchmark::generate_test_images(256, 256, 4);
        let result = ImageProcessingBenchmark::test_parallel_sharpen(&images);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_format_conversion() {
        let images = ImageProcessingBenchmark::generate_test_images(256, 256, 4);
        let result = ImageProcessingBenchmark::test_parallel_format_conversion(&images);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
