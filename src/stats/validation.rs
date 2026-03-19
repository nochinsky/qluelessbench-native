//! Statistical validation functions.
//!
//! Provides reliability classification, outlier detection, and statistical validity checks.

use crate::stats::descriptive::{
    calculate_coefficient_of_variation, calculate_mean, calculate_std_dev,
};
use serde::{Deserialize, Serialize};

/// Reliability classification based on coefficient of variation (CV).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reliability {
    /// Excellent reliability (CV ≤ 5%)
    Excellent,
    /// Good/Acceptable reliability (CV ≤ 10%)
    Good,
    /// Poor/Unreliable (CV > 10%)
    Poor,
    /// Unknown (insufficient data)
    Unknown,
}

impl Reliability {
    /// Get the icon representation for this reliability level.
    pub fn icon(&self) -> &'static str {
        match self {
            Reliability::Excellent => "✓✓",
            Reliability::Good => "⚠✓",
            Reliability::Poor => "✗⚠",
            Reliability::Unknown => "?",
        }
    }

    /// Get a CSS class name for this reliability level.
    pub fn css_class(&self) -> &'static str {
        match self {
            Reliability::Excellent => "reliability-excellent",
            Reliability::Good => "reliability-good",
            Reliability::Poor => "reliability-poor",
            Reliability::Unknown => "reliability-unknown",
        }
    }

    /// Check if this reliability level is acceptable for reporting.
    pub fn is_acceptable(&self) -> bool {
        matches!(self, Reliability::Excellent | Reliability::Good)
    }
}

/// Get the reliability classification for a coefficient of variation value.
///
/// # Classification
/// - CV ≤ 5%: Excellent (✓✓)
/// - CV ≤ 10%: Good (⚠✓)
/// - CV > 10%: Poor (✗⚠)
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::{get_reliability, Reliability};
/// assert_eq!(get_reliability(3.0), Reliability::Excellent);
/// assert_eq!(get_reliability(7.0), Reliability::Good);
/// assert_eq!(get_reliability(15.0), Reliability::Poor);
/// ```
pub fn get_reliability(cv: f64) -> Reliability {
    if cv <= 5.0 {
        Reliability::Excellent
    } else if cv <= 10.0 {
        Reliability::Good
    } else {
        Reliability::Poor
    }
}

/// Get the reliability icon for a coefficient of variation value.
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::get_reliability_icon;
/// assert_eq!(get_reliability_icon(3.0), "✓✓");
/// assert_eq!(get_reliability_icon(7.0), "⚠✓");
/// assert_eq!(get_reliability_icon(15.0), "✗⚠");
/// ```
pub fn get_reliability_icon(cv: f64) -> &'static str {
    get_reliability(cv).icon()
}

/// Detect outliers using the IQR (Interquartile Range) method.
///
/// Returns a vector of indices that are considered outliers.
/// An outlier is defined as a value that is:
/// - Below Q1 - 1.5 * IQR, or
/// - Above Q3 + 1.5 * IQR
///
/// where Q1 is the 25th percentile, Q3 is the 75th percentile,
/// and IQR = Q3 - Q1.
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::detect_outliers;
/// let values = vec![1.0, 2.0, 3.0, 4.0, 100.0];
/// let outliers = detect_outliers(&values);
/// assert_eq!(outliers, vec![4]);  // 100.0 is an outlier
/// ```
pub fn detect_outliers(values: &[f64]) -> Vec<usize> {
    if values.len() < 4 {
        return Vec::new();
    }

    let mut sorted: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let q1_idx = n / 4;
    let q3_idx = (3 * n) / 4;

    let q1 = sorted[q1_idx].1;
    let q3 = sorted[q3_idx].1;
    let iqr = q3 - q1;

    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;

    values
        .iter()
        .enumerate()
        .filter(|(_, &v)| v < lower_bound || v > upper_bound)
        .map(|(i, _)| i)
        .collect()
}

/// Check if a set of values is statistically valid for benchmark reporting.
///
/// Returns true if:
/// - There are at least 3 values
/// - The coefficient of variation is ≤ 10% (Good or Excellent reliability)
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::is_statistically_valid;
/// let good_values = vec![100.0, 101.0, 102.0, 100.5, 99.5];
/// assert!(is_statistically_valid(&good_values));
///
/// let bad_values = vec![100.0, 200.0, 50.0, 300.0, 25.0];
/// assert!(!is_statistically_valid(&bad_values));
/// ```
pub fn is_statistically_valid(values: &[f64]) -> bool {
    if values.len() < 3 {
        return false;
    }

    let cv = calculate_coefficient_of_variation(values);
    cv <= 10.0
}

/// Calculate statistical summary for a set of benchmark values.
#[derive(Debug, Clone)]
pub struct StatisticalSummary {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub cv: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub reliability: Reliability,
    pub outlier_count: usize,
}

impl StatisticalSummary {
    /// Calculate statistical summary for a set of values.
    pub fn calculate(values: &[f64]) -> Self {
        use crate::stats::descriptive::{calculate_median, calculate_percentile};

        let count = values.len();

        if count == 0 {
            return StatisticalSummary {
                count: 0,
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                cv: 0.0,
                min: 0.0,
                max: 0.0,
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
                reliability: Reliability::Unknown,
                outlier_count: 0,
            };
        }

        let mean = calculate_mean(values);
        let median = calculate_median(values);
        let std_dev = calculate_std_dev(values);
        let cv = calculate_coefficient_of_variation(values);
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let p50 = calculate_percentile(values, 50.0);
        let p95 = calculate_percentile(values, 95.0);
        let p99 = calculate_percentile(values, 99.0);
        let reliability = get_reliability(cv);
        let outlier_count = detect_outliers(values).len();

        StatisticalSummary {
            count,
            mean,
            median,
            std_dev,
            cv,
            min,
            max,
            p50,
            p95,
            p99,
            reliability,
            outlier_count,
        }
    }

    /// Check if this summary indicates statistically valid results.
    pub fn is_valid(&self) -> bool {
        self.count >= 3 && self.reliability.is_acceptable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_excellent() {
        assert_eq!(get_reliability(0.0), Reliability::Excellent);
        assert_eq!(get_reliability(3.0), Reliability::Excellent);
        assert_eq!(get_reliability(5.0), Reliability::Excellent);
    }

    #[test]
    fn test_reliability_good() {
        assert_eq!(get_reliability(5.1), Reliability::Good);
        assert_eq!(get_reliability(7.0), Reliability::Good);
        assert_eq!(get_reliability(10.0), Reliability::Good);
    }

    #[test]
    fn test_reliability_poor() {
        assert_eq!(get_reliability(10.1), Reliability::Poor);
        assert_eq!(get_reliability(15.0), Reliability::Poor);
        assert_eq!(get_reliability(50.0), Reliability::Poor);
    }

    #[test]
    fn test_reliability_icon_excellent() {
        assert_eq!(get_reliability_icon(3.0), "✓✓");
    }

    #[test]
    fn test_reliability_icon_good() {
        assert_eq!(get_reliability_icon(7.0), "⚠✓");
    }

    #[test]
    fn test_reliability_icon_poor() {
        assert_eq!(get_reliability_icon(15.0), "✗⚠");
    }

    #[test]
    fn test_reliability_is_acceptable() {
        assert!(Reliability::Excellent.is_acceptable());
        assert!(Reliability::Good.is_acceptable());
        assert!(!Reliability::Poor.is_acceptable());
        assert!(!Reliability::Unknown.is_acceptable());
    }

    #[test]
    fn test_detect_outliers_none() {
        let values = vec![100.0, 101.0, 102.0, 100.5, 99.5];
        let outliers = detect_outliers(&values);
        assert!(outliers.is_empty());
    }

    #[test]
    fn test_detect_outliers_single() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 100.0];
        let outliers = detect_outliers(&values);
        assert_eq!(outliers, vec![4]);
    }

    #[test]
    fn test_detect_outliers_multiple() {
        let values = vec![1.0, 100.0, 2.0, 3.0, 4.0, -50.0];
        let outliers = detect_outliers(&values);
        assert_eq!(outliers.len(), 2);
        assert!(outliers.contains(&1)); // 100.0
        assert!(outliers.contains(&5)); // -50.0
    }

    #[test]
    fn test_detect_outliers_few_values() {
        let values = vec![1.0, 2.0, 3.0];
        let outliers = detect_outliers(&values);
        assert!(outliers.is_empty());
    }

    #[test]
    fn test_is_statistically_valid_true() {
        let values = vec![100.0, 101.0, 102.0, 100.5, 99.5];
        assert!(is_statistically_valid(&values));
    }

    #[test]
    fn test_is_statistically_valid_few_samples() {
        let values = vec![100.0, 101.0];
        assert!(!is_statistically_valid(&values));
    }

    #[test]
    fn test_is_statistically_valid_high_cv() {
        let values = vec![100.0, 200.0, 50.0, 300.0, 25.0];
        assert!(!is_statistically_valid(&values));
    }

    #[test]
    fn test_statistical_summary() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let summary = StatisticalSummary::calculate(&values);

        assert_eq!(summary.count, 5);
        assert_eq!(summary.mean, 3.0);
        assert_eq!(summary.median, 3.0);
        assert_eq!(summary.min, 1.0);
        assert_eq!(summary.max, 5.0);
        assert_eq!(summary.p50, 3.0);
        assert!(summary.p95 > 4.0);
        assert!(summary.p99 > 4.0);
    }

    #[test]
    fn test_statistical_summary_is_valid() {
        let good_values = vec![100.0, 101.0, 102.0, 100.5, 99.5];
        let summary = StatisticalSummary::calculate(&good_values);
        assert!(summary.is_valid());

        let bad_values = vec![100.0, 200.0, 50.0];
        let summary = StatisticalSummary::calculate(&bad_values);
        assert!(!summary.is_valid());
    }

    #[test]
    fn test_css_class() {
        assert_eq!(Reliability::Excellent.css_class(), "reliability-excellent");
        assert_eq!(Reliability::Good.css_class(), "reliability-good");
        assert_eq!(Reliability::Poor.css_class(), "reliability-poor");
        assert_eq!(Reliability::Unknown.css_class(), "reliability-unknown");
    }
}
