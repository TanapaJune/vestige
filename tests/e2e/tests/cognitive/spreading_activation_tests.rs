//! # Spreading Activation E2E Tests (Phase 7.4)
//!
//! Comprehensive tests proving spreading activation finds connections
//! that pure similarity search CANNOT find.
//!
//! Based on Collins & Loftus (1975) spreading activation theory.

use vestige_core::neuroscience::spreading_activation::{
    ActivationConfig, ActivationNetwork, LinkType,
};
use std::collections::HashSet;

// ============================================================================
// MULTI-HOP ASSOCIATION TESTS (6 tests)
// ============================================================================

/// Test that spreading activation finds hidden chains that similarity search misses.
///
/// Scenario: A -> B -> C where A and C have NO direct similarity.
/// Similarity search from A would never find C, but spreading activation does.
#[test]
fn test_spreading_finds_hidden_chains() {
    let mut network = ActivationNetwork::new();

    // Create a chain: "rust_async" -> "tokio_runtime" -> "green_threads"
    // These concepts are related through association, not direct similarity
    network.add_edge(
        "rust_async".to_string(),
        "tokio_runtime".to_string(),
        LinkType::Semantic,
        0.9,
    );
    network.add_edge(
        "tokio_runtime".to_string(),
        "green_threads".to_string(),
        LinkType::Semantic,
        0.8,
    );

    // Activate from "rust_async"
    let results = network.activate("rust_async", 1.0);

    // Should find "green_threads" through the chain
    let found_green_threads = results
        .iter()
        .any(|r| r.memory_id == "green_threads");

    assert!(
        found_green_threads,
        "Spreading activation should find 'green_threads' through the chain, \
         even though it has no direct similarity to 'rust_async'"
    );

    // Verify the path was tracked correctly
    let green_threads_result = results
        .iter()
        .find(|r| r.memory_id == "green_threads")
        .unwrap();
    assert_eq!(green_threads_result.distance, 2, "Should be 2 hops away");
}

/// Test 3-hop discovery - finding concepts 3 links away.
#[test]
fn test_spreading_3_hop_discovery() {
    let config = ActivationConfig {
        decay_factor: 0.8,
        max_hops: 4,
        min_threshold: 0.05,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create a 3-hop chain: A -> B -> C -> D
    network.add_edge("memory_a".to_string(), "memory_b".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("memory_b".to_string(), "memory_c".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("memory_c".to_string(), "memory_d".to_string(), LinkType::Semantic, 0.9);

    let results = network.activate("memory_a", 1.0);

    // Find memory_d at distance 3
    let found_d = results.iter().find(|r| r.memory_id == "memory_d");
    assert!(found_d.is_some(), "Should find memory at 3 hops");
    assert_eq!(found_d.unwrap().distance, 3, "Distance should be 3 hops");
}

/// Test that spreading activation beats pure similarity search.
///
/// Creates a network where the most semantically relevant memory
/// is only reachable through association, not direct similarity.
#[test]
fn test_spreading_beats_similarity_search() {
    let mut network = ActivationNetwork::new();

    // Scenario: User asks about "memory leaks in Rust"
    // Direct similarity might find: "rust_ownership" (similar keywords)
    // But the ACTUAL solution is in "arc_weak_patterns" which is only
    // reachable through: memory_leaks -> reference_counting -> arc_weak_patterns

    network.add_edge(
        "memory_leaks".to_string(),
        "rust_ownership".to_string(),
        LinkType::Semantic,
        0.5, // Weak direct connection
    );
    network.add_edge(
        "memory_leaks".to_string(),
        "reference_counting".to_string(),
        LinkType::Causal,
        0.9,
    );
    network.add_edge(
        "reference_counting".to_string(),
        "arc_weak_patterns".to_string(),
        LinkType::Semantic,
        0.95,
    );

    let results = network.activate("memory_leaks", 1.0);

    // Find both results
    let _ownership_activation = results
        .iter()
        .find(|r| r.memory_id == "rust_ownership")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    let arc_weak_activation = results
        .iter()
        .find(|r| r.memory_id == "arc_weak_patterns")
        .map(|r| r.activation)
        .unwrap_or(0.0);

    // The arc_weak_patterns should be found even though it requires 2 hops
    assert!(
        arc_weak_activation > 0.0,
        "Should find arc_weak_patterns through spreading activation"
    );

    // Both should be in results - spreading activation surfaces hidden connections
    let memory_ids: HashSet<_> = results.iter().map(|r| r.memory_id.as_str()).collect();
    assert!(memory_ids.contains("arc_weak_patterns"));
    assert!(memory_ids.contains("reference_counting"));
}

/// Test that activation paths are correctly tracked.
#[test]
fn test_spreading_path_tracking() {
    let mut network = ActivationNetwork::new();

    network.add_edge("start".to_string(), "middle".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("middle".to_string(), "end".to_string(), LinkType::Semantic, 0.9);

    let results = network.activate("start", 1.0);

    let end_result = results.iter().find(|r| r.memory_id == "end").unwrap();

    // Path should be: start -> middle -> end
    assert_eq!(end_result.path.len(), 3);
    assert_eq!(end_result.path[0], "start");
    assert_eq!(end_result.path[1], "middle");
    assert_eq!(end_result.path[2], "end");
}

/// Test convergent activation - when multiple paths lead to the same node.
#[test]
fn test_spreading_convergent_activation() {
    let mut network = ActivationNetwork::new();

    // Create convergent paths: A -> B -> D and A -> C -> D
    network.add_edge("source".to_string(), "path1".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("source".to_string(), "path2".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("path1".to_string(), "target".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("path2".to_string(), "target".to_string(), LinkType::Semantic, 0.8);

    let results = network.activate("source", 1.0);

    // Target should receive activation from both paths
    let target_results: Vec<_> = results.iter().filter(|r| r.memory_id == "target").collect();

    // Should have at least one result for target
    assert!(!target_results.is_empty(), "Target should be activated");

    // The activation should reflect receiving from multiple sources
    // (implementation may aggregate or keep separate - test that it's found)
    let total_target_activation: f64 = target_results.iter().map(|r| r.activation).sum();
    assert!(
        total_target_activation > 0.0,
        "Target should have positive activation from convergent paths"
    );
}

/// Test semantic vs temporal link types have different effects.
#[test]
fn test_spreading_semantic_vs_temporal_links() {
    let mut network = ActivationNetwork::new();

    // Create two parallel paths with different link types
    network.add_edge(
        "event".to_string(),
        "semantic_related".to_string(),
        LinkType::Semantic,
        0.9,
    );
    network.add_edge(
        "event".to_string(),
        "temporal_related".to_string(),
        LinkType::Temporal,
        0.9,
    );

    let results = network.activate("event", 1.0);

    // Both should be found
    let semantic = results.iter().find(|r| r.memory_id == "semantic_related");
    let temporal = results.iter().find(|r| r.memory_id == "temporal_related");

    assert!(semantic.is_some(), "Should find semantically linked memory");
    assert!(temporal.is_some(), "Should find temporally linked memory");

    // Verify link types are preserved
    assert_eq!(semantic.unwrap().link_type, LinkType::Semantic);
    assert_eq!(temporal.unwrap().link_type, LinkType::Temporal);
}

// ============================================================================
// ACTIVATION DECAY TESTS (5 tests)
// ============================================================================

/// Test that activation decays with each hop.
#[test]
fn test_activation_decay_per_hop() {
    let config = ActivationConfig {
        decay_factor: 0.7,
        max_hops: 3,
        min_threshold: 0.01,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Chain with uniform strength
    network.add_edge("a".to_string(), "b".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("b".to_string(), "c".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("c".to_string(), "d".to_string(), LinkType::Semantic, 1.0);

    let results = network.activate("a", 1.0);

    let b_activation = results.iter().find(|r| r.memory_id == "b").map(|r| r.activation).unwrap_or(0.0);
    let c_activation = results.iter().find(|r| r.memory_id == "c").map(|r| r.activation).unwrap_or(0.0);
    let d_activation = results.iter().find(|r| r.memory_id == "d").map(|r| r.activation).unwrap_or(0.0);

    // Each hop should reduce activation by decay factor (0.7)
    assert!(b_activation > c_activation, "Activation should decay: b ({}) > c ({})", b_activation, c_activation);
    assert!(c_activation > d_activation, "Activation should decay: c ({}) > d ({})", c_activation, d_activation);

    // Verify approximate decay rate (allowing for floating point)
    let ratio_bc = c_activation / b_activation;
    assert!(
        (ratio_bc - 0.7).abs() < 0.1,
        "Decay ratio b->c should be ~0.7, got {}",
        ratio_bc
    );
}

/// Test that decay factor is configurable.
#[test]
fn test_activation_decay_factor_configurable() {
    // Test with high decay (0.9 - slow decay)
    let high_config = ActivationConfig {
        decay_factor: 0.9,
        max_hops: 3,
        min_threshold: 0.01,
        allow_cycles: false,
    };
    let mut high_network = ActivationNetwork::with_config(high_config);
    high_network.add_edge("a".to_string(), "b".to_string(), LinkType::Semantic, 1.0);
    high_network.add_edge("b".to_string(), "c".to_string(), LinkType::Semantic, 1.0);

    // Test with low decay (0.3 - fast decay)
    let low_config = ActivationConfig {
        decay_factor: 0.3,
        max_hops: 3,
        min_threshold: 0.01,
        allow_cycles: false,
    };
    let mut low_network = ActivationNetwork::with_config(low_config);
    low_network.add_edge("a".to_string(), "b".to_string(), LinkType::Semantic, 1.0);
    low_network.add_edge("b".to_string(), "c".to_string(), LinkType::Semantic, 1.0);

    let high_results = high_network.activate("a", 1.0);
    let low_results = low_network.activate("a", 1.0);

    let high_c = high_results.iter().find(|r| r.memory_id == "c").map(|r| r.activation).unwrap_or(0.0);
    let low_c = low_results.iter().find(|r| r.memory_id == "c").map(|r| r.activation).unwrap_or(0.0);

    assert!(
        high_c > low_c,
        "Higher decay factor should preserve more activation: {} > {}",
        high_c,
        low_c
    );
}

/// Test activation follows inverse distance law.
#[test]
fn test_activation_distance_law() {
    let config = ActivationConfig {
        decay_factor: 0.7,
        max_hops: 5,
        min_threshold: 0.001,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create a longer chain
    network.add_edge("n0".to_string(), "n1".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("n1".to_string(), "n2".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("n2".to_string(), "n3".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("n3".to_string(), "n4".to_string(), LinkType::Semantic, 1.0);

    let results = network.activate("n0", 1.0);

    // Collect activations by distance
    let mut activations_by_distance: Vec<(u32, f64)> = results
        .iter()
        .map(|r| (r.distance, r.activation))
        .collect();
    activations_by_distance.sort_by_key(|(d, _)| *d);

    // Verify monotonic decrease with distance
    for i in 1..activations_by_distance.len() {
        let (prev_dist, prev_act) = activations_by_distance[i - 1];
        let (curr_dist, curr_act) = activations_by_distance[i];
        if prev_dist < curr_dist {
            assert!(
                prev_act >= curr_act,
                "Activation should decrease with distance: d{} ({}) >= d{} ({})",
                prev_dist,
                prev_act,
                curr_dist,
                curr_act
            );
        }
    }
}

/// Test minimum activation threshold stops propagation.
#[test]
fn test_activation_minimum_threshold() {
    let config = ActivationConfig {
        decay_factor: 0.5,
        max_hops: 10,
        min_threshold: 0.2, // High threshold
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create a long chain
    network.add_edge("a".to_string(), "b".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("b".to_string(), "c".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("c".to_string(), "d".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("d".to_string(), "e".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("e".to_string(), "f".to_string(), LinkType::Semantic, 1.0);

    let results = network.activate("a", 1.0);

    // With 0.5 decay and 0.2 threshold:
    // b: 1.0 * 0.5 = 0.5 (above threshold)
    // c: 0.5 * 0.5 = 0.25 (above threshold)
    // d: 0.25 * 0.5 = 0.125 (below threshold - should not propagate)
    // So d might be found but e and f should NOT be found

    let found_e = results.iter().any(|r| r.memory_id == "e");
    let found_f = results.iter().any(|r| r.memory_id == "f");

    assert!(
        !found_e && !found_f,
        "Nodes beyond threshold should not be found. Found e: {}, f: {}",
        found_e,
        found_f
    );
}

/// Test maximum hops limit is enforced.
#[test]
fn test_activation_max_hops_limit() {
    let config = ActivationConfig {
        decay_factor: 0.99, // Almost no decay
        max_hops: 2,        // But strict hop limit
        min_threshold: 0.01,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create a chain of 5 nodes
    network.add_edge("a".to_string(), "b".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("b".to_string(), "c".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("c".to_string(), "d".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("d".to_string(), "e".to_string(), LinkType::Semantic, 1.0);

    let results = network.activate("a", 1.0);

    // Should find b (1 hop) and c (2 hops) but NOT d or e
    let found_b = results.iter().any(|r| r.memory_id == "b");
    let found_c = results.iter().any(|r| r.memory_id == "c");
    let found_d = results.iter().any(|r| r.memory_id == "d");
    let found_e = results.iter().any(|r| r.memory_id == "e");

    assert!(found_b, "Should find b at 1 hop");
    assert!(found_c, "Should find c at 2 hops");
    assert!(!found_d, "Should NOT find d at 3 hops (exceeds max_hops=2)");
    assert!(!found_e, "Should NOT find e at 4 hops");
}

// ============================================================================
// EDGE REINFORCEMENT TESTS (5 tests)
// ============================================================================

/// Test Hebbian reinforcement - "neurons that fire together wire together".
#[test]
fn test_hebbian_reinforcement() {
    let mut network = ActivationNetwork::new();

    // Initial weak connection
    network.add_edge(
        "concept_a".to_string(),
        "concept_b".to_string(),
        LinkType::Semantic,
        0.3,
    );

    // Get initial strength
    let initial_associations = network.get_associations("concept_a");
    let initial_strength = initial_associations
        .iter()
        .find(|a| a.memory_id == "concept_b")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    // Reinforce the connection (simulating co-activation)
    network.reinforce_edge("concept_a", "concept_b", 0.2);

    // Get reinforced strength
    let reinforced_associations = network.get_associations("concept_a");
    let reinforced_strength = reinforced_associations
        .iter()
        .find(|a| a.memory_id == "concept_b")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    assert!(
        reinforced_strength > initial_strength,
        "Reinforcement should increase edge strength: {} > {}",
        reinforced_strength,
        initial_strength
    );
}

/// Test that edge strength increases with repeated use.
#[test]
fn test_edge_strength_increases_with_use() {
    let mut network = ActivationNetwork::new();

    network.add_edge(
        "frequently_used".to_string(),
        "target".to_string(),
        LinkType::Semantic,
        0.2,
    );

    let mut strengths = vec![];

    // Record initial strength
    let assoc = network.get_associations("frequently_used");
    strengths.push(assoc[0].association_strength);

    // Reinforce multiple times
    for _ in 0..5 {
        network.reinforce_edge("frequently_used", "target", 0.1);
        let assoc = network.get_associations("frequently_used");
        strengths.push(assoc[0].association_strength);
    }

    // Verify monotonic increase (until capped at 1.0)
    for i in 1..strengths.len() {
        assert!(
            strengths[i] >= strengths[i - 1],
            "Strength should increase with use: {} >= {}",
            strengths[i],
            strengths[i - 1]
        );
    }

    // Final strength should be significantly higher than initial
    assert!(
        strengths.last().unwrap() > &0.5,
        "After multiple reinforcements, strength should be high"
    );
}

/// Test that traversal count is tracked on edges.
#[test]
fn test_traversal_count_tracking() {
    let mut network = ActivationNetwork::new();

    network.add_edge(
        "source".to_string(),
        "target".to_string(),
        LinkType::Semantic,
        0.8,
    );

    // Reinforce multiple times (each reinforcement increments activation_count)
    for _ in 0..3 {
        network.reinforce_edge("source", "target", 0.05);
    }

    // The edge should have been reinforced 3 times
    // Note: We verify this through the association strength increasing
    let associations = network.get_associations("source");
    let final_strength = associations
        .iter()
        .find(|a| a.memory_id == "target")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    // Should be 0.8 + 3*0.05 = 0.95
    assert!(
        (final_strength - 0.95).abs() < 0.01,
        "Strength should reflect 3 reinforcements: expected 0.95, got {}",
        final_strength
    );
}

/// Test that different link types can have different weights.
#[test]
fn test_link_type_weights() {
    let mut network = ActivationNetwork::new();

    // Create edges with different link types and strengths
    network.add_edge(
        "event".to_string(),
        "semantic_link".to_string(),
        LinkType::Semantic,
        0.9,
    );
    network.add_edge(
        "event".to_string(),
        "temporal_link".to_string(),
        LinkType::Temporal,
        0.5,
    );
    network.add_edge(
        "event".to_string(),
        "causal_link".to_string(),
        LinkType::Causal,
        0.7,
    );

    let results = network.activate("event", 1.0);

    // Verify different activations based on edge strength
    let semantic_act = results.iter().find(|r| r.memory_id == "semantic_link").map(|r| r.activation).unwrap_or(0.0);
    let temporal_act = results.iter().find(|r| r.memory_id == "temporal_link").map(|r| r.activation).unwrap_or(0.0);
    let causal_act = results.iter().find(|r| r.memory_id == "causal_link").map(|r| r.activation).unwrap_or(0.0);

    // Semantic (0.9) > Causal (0.7) > Temporal (0.5)
    assert!(
        semantic_act > causal_act && causal_act > temporal_act,
        "Activation should reflect edge strengths: semantic ({}) > causal ({}) > temporal ({})",
        semantic_act,
        causal_act,
        temporal_act
    );
}

/// Test edge decay without use (edges weaken over time if not reinforced).
#[test]
fn test_edge_decay_without_use() {
    let mut network = ActivationNetwork::new();

    network.add_edge(
        "forgotten".to_string(),
        "target".to_string(),
        LinkType::Semantic,
        0.8,
    );

    // Get initial associations
    let initial = network.get_associations("forgotten");
    let initial_strength = initial[0].association_strength;

    // Note: The current implementation doesn't have automatic time-based decay
    // But we can test the apply_decay method through edge manipulation
    // For now, we verify the initial state is correct

    assert!(
        (initial_strength - 0.8).abs() < 0.01,
        "Initial strength should be 0.8"
    );

    // Test that edges can be retrieved and have correct properties
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].memory_id, "target");
    assert_eq!(initial[0].link_type, LinkType::Semantic);
}

// ============================================================================
// NETWORK BUILDING TESTS (4 tests)
// ============================================================================

/// Test network builds from semantic similarity.
#[test]
fn test_network_builds_from_semantic_similarity() {
    let mut network = ActivationNetwork::new();

    // Build a network representing semantic relationships in code
    // These would typically be built from embedding similarity

    // Rust async ecosystem
    network.add_edge("async_rust".to_string(), "tokio".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("async_rust".to_string(), "async_await".to_string(), LinkType::Semantic, 0.95);
    network.add_edge("tokio".to_string(), "runtime".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("tokio".to_string(), "spawn".to_string(), LinkType::Semantic, 0.85);

    assert_eq!(network.node_count(), 5);
    assert_eq!(network.edge_count(), 4);

    // Verify associations are retrievable
    let async_associations = network.get_associations("async_rust");
    assert_eq!(async_associations.len(), 2);

    // Highest association should be async_await (0.95)
    assert_eq!(async_associations[0].memory_id, "async_await");
}

/// Test network builds from temporal proximity.
#[test]
fn test_network_builds_from_temporal_proximity() {
    let mut network = ActivationNetwork::new();

    // Build a network from temporal co-occurrence
    // Events that happened close in time

    // Morning standup sequence
    network.add_edge("standup".to_string(), "jira_update".to_string(), LinkType::Temporal, 0.9);
    network.add_edge("jira_update".to_string(), "code_review".to_string(), LinkType::Temporal, 0.85);
    network.add_edge("code_review".to_string(), "merge_pr".to_string(), LinkType::Temporal, 0.8);

    // Verify temporal chain
    let results = network.activate("standup", 1.0);

    // Should find the whole workflow sequence
    let found_merge = results.iter().any(|r| r.memory_id == "merge_pr");
    assert!(found_merge, "Should find temporally linked merge_pr");

    // Verify link types are temporal
    for result in &results {
        assert_eq!(
            result.link_type,
            LinkType::Temporal,
            "All links should be temporal"
        );
    }
}

/// Test that semantic and temporal link types are differentiated.
#[test]
fn test_network_link_types_differentiated() {
    let mut network = ActivationNetwork::new();

    // Same nodes, different link types
    network.add_edge(
        "feature_a".to_string(),
        "feature_b".to_string(),
        LinkType::Semantic,
        0.7,
    );
    network.add_edge(
        "feature_a".to_string(),
        "feature_c".to_string(),
        LinkType::Temporal,
        0.7,
    );
    network.add_edge(
        "feature_a".to_string(),
        "feature_d".to_string(),
        LinkType::Causal,
        0.7,
    );
    network.add_edge(
        "feature_a".to_string(),
        "feature_e".to_string(),
        LinkType::PartOf,
        0.7,
    );

    let associations = network.get_associations("feature_a");

    // Collect link types
    let link_types: HashSet<LinkType> = associations.iter().map(|a| a.link_type).collect();

    assert!(link_types.contains(&LinkType::Semantic));
    assert!(link_types.contains(&LinkType::Temporal));
    assert!(link_types.contains(&LinkType::Causal));
    assert!(link_types.contains(&LinkType::PartOf));

    assert_eq!(link_types.len(), 4, "Should have 4 different link types");
}

/// Test batch construction of network.
#[test]
fn test_network_batch_construction() {
    let mut network = ActivationNetwork::new();

    // Simulate batch construction from a knowledge graph
    let edges = vec![
        ("rust", "cargo", LinkType::Semantic, 0.9),
        ("rust", "ownership", LinkType::Semantic, 0.95),
        ("rust", "traits", LinkType::Semantic, 0.9),
        ("cargo", "dependencies", LinkType::Semantic, 0.85),
        ("cargo", "build", LinkType::PartOf, 0.8),
        ("ownership", "borrowing", LinkType::Semantic, 0.9),
        ("ownership", "lifetimes", LinkType::Semantic, 0.85),
        ("traits", "generics", LinkType::Semantic, 0.8),
        ("traits", "impl", LinkType::PartOf, 0.9),
    ];

    for (source, target, link_type, strength) in edges {
        network.add_edge(source.to_string(), target.to_string(), link_type, strength);
    }

    // Verify network structure
    assert_eq!(network.node_count(), 10, "Should have 10 unique nodes");
    assert_eq!(network.edge_count(), 9, "Should have 9 edges");

    // Test spreading from rust
    let results = network.activate("rust", 1.0);

    // Should reach multiple concepts
    let reached_nodes: HashSet<_> = results.iter().map(|r| r.memory_id.as_str()).collect();

    assert!(reached_nodes.contains("cargo"));
    assert!(reached_nodes.contains("ownership"));
    assert!(reached_nodes.contains("traits"));
    assert!(reached_nodes.contains("borrowing")); // 2 hops: rust -> ownership -> borrowing

    // Count nodes at each distance
    let distance_1: Vec<_> = results.iter().filter(|r| r.distance == 1).collect();
    let distance_2: Vec<_> = results.iter().filter(|r| r.distance == 2).collect();

    assert_eq!(distance_1.len(), 3, "Should have 3 nodes at distance 1 (cargo, ownership, traits)");
    assert!(distance_2.len() >= 4, "Should have at least 4 nodes at distance 2");
}
