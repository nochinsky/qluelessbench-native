//! Machine Learning inference benchmark tests.
//!
//! Tests neural network forward pass performance using ndarray.

use anyhow::Result;
use ndarray::{Array1, Array2};
use rayon::prelude::*;
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::results::CategoryResult;

/// ML inference benchmark.
pub struct MLInferenceBenchmark {
    multi_core: bool,
}

impl MLInferenceBenchmark {
    /// Create a new MLInferenceBenchmark.
    pub fn new() -> Self {
        MLInferenceBenchmark { multi_core: false }
    }

    /// Create a new MLInferenceBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        MLInferenceBenchmark { multi_core: true }
    }

    /// Simple neural network forward pass.
    fn test_nn_forward(input_size: usize, hidden_size: usize, output_size: usize) -> Result<f64> {
        // Create random weights (in a real benchmark, these would be pre-trained)
        let w1 = Array2::<f64>::from_shape_fn((hidden_size, input_size), |(_i, _j)| 0.01);
        let b1 = Array1::<f64>::zeros(hidden_size);
        let w2 = Array2::<f64>::from_shape_fn((output_size, hidden_size), |(_i, _j)| 0.01);
        let b2 = Array1::<f64>::zeros(output_size);

        // Create random input
        let input = Array1::<f64>::from_shape_fn(input_size, |i| (i % 100) as f64 / 100.0);

        let start = Instant::now();

        // Forward pass: input -> hidden -> output
        // Hidden layer with ReLU
        let mut hidden = w1.dot(&input) + &b1;
        hidden.mapv_inplace(|x| x.max(0.0)); // ReLU

        // Output layer with softmax
        let mut output = w2.dot(&hidden) + &b2;
        let max_val = output.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_sum: f64 = output.iter().map(|&x| (x - max_val).exp()).sum();
        output.mapv_inplace(|x| (x - max_val).exp() / exp_sum);

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test batch inference (multiple inputs at once).
    fn test_batch_inference(
        batch_size: usize,
        input_size: usize,
        hidden_size: usize,
        output_size: usize,
    ) -> Result<f64> {
        // Create weights
        let w1 = Array2::<f64>::from_shape_fn((hidden_size, input_size), |(_i, _j)| 0.01);
        let b1 = Array1::<f64>::zeros(hidden_size);
        let w2 = Array2::<f64>::from_shape_fn((output_size, hidden_size), |(_i, _j)| 0.01);
        let b2 = Array1::<f64>::zeros(output_size);

        // Create batch of inputs
        let inputs: Vec<Array1<f64>> = (0..batch_size)
            .map(|i| Array1::from_shape_fn(input_size, |j| ((i + j) % 100) as f64 / 100.0))
            .collect();

        let start = Instant::now();

        // Process all inputs
        for input in &inputs {
            let mut hidden = w1.dot(input) + &b1;
            hidden.mapv_inplace(|x| x.max(0.0));
            let mut _output = w2.dot(&hidden) + &b2;
        }

        let duration = start.elapsed().as_secs_f64();
        Ok(batch_size as f64 / duration)
    }

    /// Test parallel batch inference.
    fn test_parallel_batch_inference(
        num_batches: usize,
        batch_size: usize,
        input_size: usize,
        hidden_size: usize,
        output_size: usize,
    ) -> Result<f64> {
        let start = Instant::now();

        (0..num_batches)
            .into_par_iter()
            .try_for_each(|_i| -> Result<()> {
                let _ =
                    Self::test_batch_inference(batch_size, input_size, hidden_size, output_size)?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        Ok((num_batches * batch_size) as f64 / duration)
    }
}

impl Default for MLInferenceBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for MLInferenceBenchmark {
    fn category_name(&self) -> &'static str {
        "ML Inference"
    }

    fn weight(&self) -> f64 {
        1.3
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Reference values (inferences per second)
        let single_ref = 5000.0;
        let batch_ref = 500.0;
        let parallel_ref = 5000.0;

        if self.multi_core {
            let num_workers = get_parallel_workers();

            // Multi-core: Parallel batch inference
            let test_fn = || Self::test_parallel_batch_inference(num_workers, 32, 784, 128, 10);
            let result = run_with_iterations(
                test_fn,
                "ML Inference (parallel batches)",
                parallel_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core: Single inference
            let test_fn = || Self::test_nn_forward(784, 128, 10);
            let result = run_with_iterations(
                test_fn,
                "NN Forward Pass",
                single_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Single-core: Batch inference
            let test_fn = || Self::test_batch_inference(32, 784, 128, 10);
            let result = run_with_iterations(
                test_fn,
                "Batch Inference (32)",
                batch_ref,
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
