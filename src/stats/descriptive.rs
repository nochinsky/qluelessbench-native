//! Descriptive statistics functions.
//!
//! Provides median, percentile, mean, standard deviation, and coefficient of variation.

/// Calculate the median of a slice of values.
///
/// Returns 0.0 if the slice is empty.
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::descriptive::calculate_median;
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// assert_eq!(calculate_median(&values), 3.0);
/// ```
pub fn calculate_median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted: Vec<f64> = values.to_vec();
    sorted.retain(|x| !x.is_nan());
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let mid = n / 2;

    if n % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Calculate the p-th percentile of a slice of values.
///
/// Percentile should be between 0.0 and 100.0.
/// Common values: p95 (95.0), p99 (99.0)
///
/// Returns 0.0 if the slice is empty.
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::descriptive::calculate_percentile;
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// assert_eq!(calculate_percentile(&values, 50.0), 3.0);
/// assert_eq!(calculate_percentile(&values, 95.0), 4.8);
/// ```
pub fn calculate_percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let percentile = percentile.clamp(0.0, 100.0) / 100.0;

    let mut sorted: Vec<f64> = values.to_vec();
    sorted.retain(|x| !x.is_nan());
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if sorted.is_empty() {
        return 0.0;
    }

    let n = sorted.len() as f64;
    let rank = percentile * (n - 1.0);
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let fraction = rank - lower as f64;

    if lower == upper {
        sorted[lower]
    } else {
        sorted[lower] + fraction * (sorted[upper] - sorted[lower])
    }
}

/// Calculate the arithmetic mean of a slice of values.
///
/// Returns 0.0 if the slice is empty.
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::descriptive::calculate_mean;
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// assert_eq!(calculate_mean(&values), 3.0);
/// ```
pub fn calculate_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calculate the sample standard deviation of a slice of values.
///
/// Returns 0.0 if the slice has fewer than 2 elements.
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::descriptive::calculate_std_dev;
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let std = calculate_std_dev(&values);
/// assert!((std - 1.58).abs() < 0.01);
/// ```
pub fn calculate_std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let mean = calculate_mean(values);
    let variance: f64 =
        values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Calculate the coefficient of variation (CV) as a percentage.
///
/// CV = (standard deviation / mean) * 100
///
/// Returns 0.0 if the slice has fewer than 2 elements or if the mean is 0.
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::descriptive::calculate_coefficient_of_variation;
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let cv = calculate_coefficient_of_variation(&values);
/// assert!((cv - 52.7).abs() < 0.1);
/// ```
pub fn calculate_coefficient_of_variation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let mean = calculate_mean(values);
    if mean == 0.0 {
        return 0.0;
    }

    let std_dev = calculate_std_dev(values);
    (std_dev / mean.abs()) * 100.0
}

/// Calculate the geometric mean of a slice of positive values.
///
/// Returns 0.0 if the slice is empty or contains non-positive values.
///
/// # Examples
///
/// ```
/// # use qluelessbench_native::stats::descriptive::calculate_geometric_mean;
/// let values = vec![1.0, 2.0, 4.0, 8.0];
/// let geo_mean = calculate_geometric_mean(&values);
/// assert!((geo_mean - 2.83).abs() < 0.01);
/// ```
pub fn calculate_geometric_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    // Check for non-positive values
    if values.iter().any(|&x| x <= 0.0) {
        return 0.0;
    }

    let log_sum: f64 = values.iter().map(|x| x.ln()).sum();
    (log_sum / values.len() as f64).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_empty() {
        assert_eq!(calculate_median(&[]), 0.0);
    }

    #[test]
    fn test_median_odd() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_median(&values), 3.0);
    }

    #[test]
    fn test_median_even() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(calculate_median(&values), 2.5);
    }

    #[test]
    fn test_median_unsorted() {
        let values = vec![5.0, 1.0, 3.0, 2.0, 4.0];
        assert_eq!(calculate_median(&values), 3.0);
    }

    #[test]
    fn test_percentile_empty() {
        assert_eq!(calculate_percentile(&[], 50.0), 0.0);
    }

    #[test]
    fn test_percentile_p50() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 50.0), 3.0);
    }

    #[test]
    fn test_percentile_p95() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 95.0), 4.8);
    }

    #[test]
    fn test_percentile_p99() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 99.0), 4.96);
    }

    #[test]
    fn test_percentile_p0() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 0.0), 1.0);
    }

    #[test]
    fn test_percentile_p100() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 100.0), 5.0);
    }

    #[test]
    fn test_mean_empty() {
        assert_eq!(calculate_mean(&[]), 0.0);
    }

    #[test]
    fn test_mean_simple() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_mean(&values), 3.0);
    }

    #[test]
    fn test_std_dev_empty() {
        assert_eq!(calculate_std_dev(&[]), 0.0);
    }

    #[test]
    fn test_std_dev_single() {
        assert_eq!(calculate_std_dev(&[5.0]), 0.0);
    }

    #[test]
    fn test_std_dev_simple() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let std = calculate_std_dev(&values);
        assert!((std - 1.581).abs() < 0.001);
    }

    #[test]
    fn test_cv_empty() {
        assert_eq!(calculate_coefficient_of_variation(&[]), 0.0);
    }

    #[test]
    fn test_cv_single() {
        assert_eq!(calculate_coefficient_of_variation(&[5.0]), 0.0);
    }

    #[test]
    fn test_cv_simple() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cv = calculate_coefficient_of_variation(&values);
        assert!((cv - 52.704).abs() < 0.001);
    }

    #[test]
    fn test_geometric_mean_empty() {
        assert_eq!(calculate_geometric_mean(&[]), 0.0);
    }

    #[test]
    fn test_geometric_mean_with_zero() {
        let values = vec![1.0, 2.0, 0.0, 4.0];
        assert_eq!(calculate_geometric_mean(&values), 0.0);
    }

    #[test]
    fn test_geometric_mean_with_negative() {
        let values = vec![1.0, 2.0, -3.0, 4.0];
        assert_eq!(calculate_geometric_mean(&values), 0.0);
    }

    #[test]
    fn test_geometric_mean_simple() {
        let values = vec![1.0, 2.0, 4.0, 8.0];
        let geo_mean = calculate_geometric_mean(&values);
        assert!((geo_mean - 2.828).abs() < 0.001);
    }
}
