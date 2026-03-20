//! Benchmark category implementations.
//!
//! This module contains all the benchmark category implementations:
//! - File I/O
//! - Compression
//! - Image Processing
//! - Text Processing
//! - Database
//! - Mathematical
//! - Archive
//! - Memory
//! - Concurrent
//! - Cryptography
//! - Ray Tracing
//! - ML Inference
//! - Navigation
//! - Image Filters

pub mod archive;
pub mod base;
pub mod compression;
pub mod concurrent;
pub mod crypto;
pub mod database;
pub mod fileio;
pub mod image;
pub mod image_filters;
pub mod math;
pub mod memory;
pub mod ml;
pub mod navigation;
pub mod raytrace;
pub mod text;

pub use archive::ArchiveBenchmark;
pub use base::{BaseBenchmark, BenchmarkEntry, WorkloadScale};
pub use compression::CompressionBenchmark;
pub use concurrent::ConcurrentBenchmark;
pub use crypto::CryptoBenchmark;
pub use database::DatabaseBenchmark;
pub use fileio::FileIOBenchmark;
pub use image::ImageProcessingBenchmark;
pub use image_filters::ImageFiltersBenchmark;
pub use math::MathematicalBenchmark;
pub use memory::MemoryBenchmark;
pub use ml::MLInferenceBenchmark;
pub use navigation::NavigationBenchmark;
pub use raytrace::RayTracingBenchmark;
pub use text::TextProcessingBenchmark;
