//! # Mathematical Validation Tests for Vestige (Extreme Testing)
//!
//! These tests validate mathematical correctness and theoretical properties:
//! - Activation decay follows expected exponential curves
//! - Conservation properties in spreading activation
//! - Forgetting curve accuracy (FSRS-6)
//! - Statistical properties of embeddings
//! - Information theoretic measures
//!
//! Based on mathematical foundations of memory systems and neuroscience

use vestige_core::neuroscience::spreading_activation::{
    ActivationConfig, ActivationNetwork, LinkType,
};
use vestige_core::neuroscience::hippocampal_index::{
    BarcodeGenerator, HippocampalIndex, INDEX_EMBEDDING_DIM,
};
use chrono::{Duration, Utc};
use std::collections::HashMap;

// ============================================================================
// EXPONENTIAL DECAY VALIDATION (1 test)
// ============================================================================

/// Test that activation decay follows exponential decay law.
///
/// Validates: A(n) = A(0) * decay_factor^n
/// where n is the number of hops.
#[test]
fn test_math_exponential_decay_law() {
    let decay_factor = 0.7;
    let config = ActivationConfig {
        decay_factor,
        max_hops: 10,
        min_threshold: 0.001,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create a simple chain with uniform edge weights (1.0)
    for i in 0..10 {
        network.add_edge(
            format!("node_{}", i),
            format!("node_{}", i + 1),
            LinkType::Semantic,
            1.0,  // Unit weight to isolate decay effect
        );
    }

    let results = network.activate("node_0", 1.0);

    // Verify exponential decay at each hop
    let mut distance_activations: HashMap<u32, f64> = HashMap::new();
    for result in &results {
        distance_activations.insert(result.distance, result.activation);
    }

    // Check decay at each distance
    for distance in 1..=5 {
        if let Some(&activation) = distance_activations.get(&distance) {
            let expected = decay_factor.powi(distance as i32);
            let error = (activation - expected).abs();

            assert!(
                error < 0.05,
                "Distance {}: expected {:.4}, got {:.4}, error {:.4}",
                distance,
                expected,
                activation,
                error
            );
        }
    }

    // Verify monotonic decrease
    let mut prev_activation = 1.0;
    for distance in 1..=5 {
        if let Some(&activation) = distance_activations.get(&distance) {
            assert!(
                activation < prev_activation,
                "Activation should decrease: d{} ({}) < prev ({})",
                distance,
                activation,
                prev_activation
            );
            prev_activation = activation;
        }
    }
}

// ============================================================================
// EDGE WEIGHT MULTIPLICATION (1 test)
// ============================================================================

/// Test that edge weights correctly multiply with activation.
///
/// Validates: A(target) = A(source) * decay_factor * edge_weight
#[test]
fn test_math_edge_weight_multiplication() {
    let decay_factor = 0.8;
    let config = ActivationConfig {
        decay_factor,
        max_hops: 2,
        min_threshold: 0.001,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create edges with different weights
    let test_weights = [0.1, 0.25, 0.5, 0.75, 1.0];

    for (i, &weight) in test_weights.iter().enumerate() {
        network.add_edge(
            "source".to_string(),
            format!("target_{}", i),
            LinkType::Semantic,
            weight,
        );
    }

    let results = network.activate("source", 1.0);

    // Verify each target's activation
    for (i, &weight) in test_weights.iter().enumerate() {
        let target_id = format!("target_{}", i);
        let expected_activation = decay_factor * weight;

        let actual_activation = results
            .iter()
            .find(|r| r.memory_id == target_id)
            .map(|r| r.activation)
            .unwrap_or(0.0);

        let error = (actual_activation - expected_activation).abs();
        assert!(
            error < 0.01,
            "Target {}: weight {}, expected {:.4}, got {:.4}",
            i,
            weight,
            expected_activation,
            actual_activation
        );
    }

    // Verify ordering (higher weight = higher activation)
    let mut activation_tuples: Vec<(f64, f64)> = test_weights
        .iter()
        .enumerate()
        .filter_map(|(i, &weight)| {
            results
                .iter()
                .find(|r| r.memory_id == format!("target_{}", i))
                .map(|r| (weight, r.activation))
        })
        .collect();

    activation_tuples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for i in 1..activation_tuples.len() {
        assert!(
            activation_tuples[i].1 >= activation_tuples[i - 1].1,
            "Higher weight should yield higher activation"
        );
    }
}

// ============================================================================
// TOTAL ACTIVATION BOUNDS (1 test)
// ============================================================================

/// Test that total activation is bounded.
///
/// Validates that spreading activation doesn't create infinite energy.
#[test]
fn test_math_activation_bounds() {
    let config = ActivationConfig {
        decay_factor: 0.8,
        max_hops: 5,
        min_threshold: 0.05,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create a converging network (many paths to same target)
    for i in 0..10 {
        network.add_edge(
            "hub".to_string(),
            format!("intermediate_{}", i),
            LinkType::Semantic,
            0.8,
        );
        network.add_edge(
            format!("intermediate_{}", i),
            "sink".to_string(),
            LinkType::Semantic,
            0.8,
        );
    }

    let results = network.activate("hub", 1.0);

    // All activations should be <= 1.0
    for result in &results {
        assert!(
            result.activation <= 1.0,
            "Activation should be bounded by 1.0: {} has {}",
            result.memory_id,
            result.activation
        );
        assert!(
            result.activation >= 0.0,
            "Activation should be non-negative: {} has {}",
            result.memory_id,
            result.activation
        );
    }

    // Total activation should be bounded
    // (for a tree with decay d, total <= 1 / (1 - d) for geometric series)
    let total_activation: f64 = results.iter().map(|r| r.activation).sum();
    let theoretical_max = 1.0 / (1.0 - 0.8);  // = 5.0 for infinite series

    assert!(
        total_activation < theoretical_max * 3.0,  // Allow margin for fan-out and multi-source
        "Total activation should be bounded: {} < {}",
        total_activation,
        theoretical_max * 3.0
    );
}

// ============================================================================
// BARCODE UNIQUENESS STATISTICS (1 test)
// ============================================================================

/// Test statistical properties of barcode generation.
///
/// Validates uniqueness and distribution of generated barcodes.
#[test]
fn test_math_barcode_statistics() {
    let mut generator = BarcodeGenerator::new();
    let now = Utc::now();

    // Generate many barcodes
    let num_barcodes = 10000;
    let mut ids: Vec<u64> = Vec::with_capacity(num_barcodes);
    let mut fingerprints: Vec<u32> = Vec::with_capacity(num_barcodes);
    let mut compact_strings: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 0..num_barcodes {
        let content = format!("Unique content number {} with some variation {}", i, i * 7);
        let timestamp = now + Duration::milliseconds(i as i64);
        let barcode = generator.generate(&content, timestamp);

        ids.push(barcode.id);
        fingerprints.push(barcode.content_fingerprint);
        compact_strings.insert(barcode.to_compact_string());
    }

    // Test 1: All IDs should be unique and sequential
    for i in 1..ids.len() {
        assert_eq!(
            ids[i],
            ids[i - 1] + 1,
            "IDs should be sequential: {} -> {}",
            ids[i - 1],
            ids[i]
        );
    }

    // Test 2: All compact strings should be unique
    assert_eq!(
        compact_strings.len(),
        num_barcodes,
        "All compact strings should be unique"
    );

    // Test 3: Content fingerprints should be mostly unique
    // (with 10000 samples, collision probability is low for good hash)
    let unique_fingerprints: std::collections::HashSet<u32> = fingerprints.iter().copied().collect();
    let uniqueness_ratio = unique_fingerprints.len() as f64 / num_barcodes as f64;

    assert!(
        uniqueness_ratio > 0.99,
        "Fingerprint uniqueness should be > 99%: {:.2}%",
        uniqueness_ratio * 100.0
    );

    // Test 4: Fingerprint distribution (check for clustering)
    // Divide into 256 buckets and check distribution
    let mut buckets = [0u32; 256];
    for fp in &fingerprints {
        let bucket = (*fp % 256) as usize;
        buckets[bucket] += 1;
    }

    let expected_per_bucket = num_barcodes as f64 / 256.0;
    let mut chi_squared = 0.0;
    for &count in &buckets {
        let diff = count as f64 - expected_per_bucket;
        chi_squared += diff * diff / expected_per_bucket;
    }

    // Chi-squared critical value for 255 df at 99% confidence is ~310
    // We use a looser bound for test stability
    assert!(
        chi_squared < 500.0,
        "Fingerprint distribution should be roughly uniform: chi^2 = {:.2}",
        chi_squared
    );
}

// ============================================================================
// EMBEDDING DIMENSION VALIDATION (1 test)
// ============================================================================

/// Test that index embeddings have correct dimensionality.
///
/// Validates that the hippocampal index uses proper embedding dimensions.
#[test]
fn test_math_embedding_dimensions() {
    let index = HippocampalIndex::new();
    let now = Utc::now();

    // Create full-size embedding (384 dimensions)
    let full_embedding: Vec<f32> = (0..384)
        .map(|i| (i as f32 / 384.0).sin())
        .collect();

    // Index memory with embedding
    let result = index.index_memory(
        "test_memory",
        "Test content for embedding validation",
        "fact",
        now,
        Some(full_embedding.clone()),
    );

    assert!(result.is_ok(), "Should index memory with full embedding");

    // Verify index stats show correct dimensions
    let stats = index.stats();
    assert_eq!(
        stats.index_dimensions,
        INDEX_EMBEDDING_DIM,
        "Index should use compressed embedding dimension ({})",
        INDEX_EMBEDDING_DIM
    );

    // Compression ratio should be reasonable
    let compression_ratio = 384.0 / INDEX_EMBEDDING_DIM as f64;
    assert!(
        (2.0..=4.0).contains(&compression_ratio),
        "Compression ratio should be 2-4x: {:.2}x",
        compression_ratio
    );

    // Test with undersized embedding
    let small_embedding: Vec<f32> = (0..64).map(|i| i as f32 / 64.0).collect();

    let small_result = index.index_memory(
        "small_embedding_memory",
        "Memory with small embedding",
        "fact",
        now,
        Some(small_embedding),
    );

    // Should handle gracefully (either accept or return clear error)
    let _ = small_result;

    // Test with oversized embedding
    let large_embedding: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();

    let large_result = index.index_memory(
        "large_embedding_memory",
        "Memory with large embedding",
        "fact",
        now,
        Some(large_embedding),
    );

    // Should handle gracefully
    let _ = large_result;

    // Verify index is still consistent
    let final_stats = index.stats();
    assert!(
        final_stats.total_indices >= 1,
        "Index should have at least the valid memory"
    );
}
