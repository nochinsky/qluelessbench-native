//! Statistical analysis functions for benchmark results.
//!
//! This module provides statistical functions for analyzing benchmark data,
//! including median, percentile, coefficient of variation, and outlier detection.

pub mod descriptive;
pub mod validation;

pub use descriptive::{
    calculate_coefficient_of_variation, calculate_geometric_mean, calculate_mean, calculate_median,
    calculate_percentile, calculate_std_dev,
};

pub use validation::{
    detect_outliers, get_reliability, get_reliability_icon, is_statistically_valid, Reliability,
    StatisticalSummary,
};
