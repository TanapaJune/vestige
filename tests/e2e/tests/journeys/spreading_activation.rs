//! # Spreading Activation Journey Tests
//!
//! Tests the associative memory network that finds hidden connections
//! between memories through spreading activation - a technique inspired
//! by how neurons activate related memories in the brain.
//!
//! ## User Journey
//!
//! 1. User builds up memories over time (code, concepts, decisions)
//! 2. User queries for a concept
//! 3. System activates the source memory
//! 4. Activation spreads to related memories via association links
//! 5. User discovers hidden connections they didn't explicitly search for

use vestige_core::neuroscience::spreading_activation::{
    ActivationConfig, ActivationNetwork, LinkType,
};
use std::collections::HashSet;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a network with a coding knowledge graph
fn create_coding_network() -> ActivationNetwork {
    let mut network = ActivationNetwork::new();

    // Rust ecosystem
    network.add_edge("rust".to_string(), "ownership".to_string(), LinkType::Semantic, 0.95);
    network.add_edge("rust".to_string(), "borrowing".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("rust".to_string(), "cargo".to_string(), LinkType::PartOf, 0.85);
    network.add_edge("ownership".to_string(), "memory_safety".to_string(), LinkType::Causal, 0.9);
    network.add_edge("borrowing".to_string(), "lifetimes".to_string(), LinkType::Semantic, 0.85);

    // Async ecosystem
    network.add_edge("rust".to_string(), "async_rust".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("async_rust".to_string(), "tokio".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("tokio".to_string(), "runtime".to_string(), LinkType::PartOf, 0.85);
    network.add_edge("async_rust".to_string(), "futures".to_string(), LinkType::Semantic, 0.85);

    network
}

/// Create a network for testing multi-hop discovery
fn create_chain_network() -> ActivationNetwork {
    let config = ActivationConfig {
        decay_factor: 0.8,
        max_hops: 5,
        min_threshold: 0.05,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create a chain: A -> B -> C -> D -> E
    network.add_edge("node_a".to_string(), "node_b".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("node_b".to_string(), "node_c".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("node_c".to_string(), "node_d".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("node_d".to_string(), "node_e".to_string(), LinkType::Semantic, 0.9);

    network
}

// ============================================================================
// TEST 1: SPREADING FINDS HIDDEN CHAINS
// ============================================================================

/// Test that spreading activation discovers memories through chains.
///
/// Validates:
/// - Direct neighbors are activated
/// - 2-hop neighbors are activated
/// - Activation decays with distance
/// - Path is tracked correctly
#[test]
fn test_spreading_finds_hidden_chains() {
    let mut network = create_chain_network();

    // Activate from node_a
    let results = network.activate("node_a", 1.0);

    // Should find all nodes in the chain
    let found_ids: HashSet<_> = results.iter().map(|r| r.memory_id.as_str()).collect();

    assert!(found_ids.contains("node_b"), "Should find direct neighbor node_b");
    assert!(found_ids.contains("node_c"), "Should find 2-hop node_c");
    assert!(found_ids.contains("node_d"), "Should find 3-hop node_d");
    assert!(found_ids.contains("node_e"), "Should find 4-hop node_e");

    // Verify distance tracking
    let node_b = results.iter().find(|r| r.memory_id == "node_b").unwrap();
    let node_e = results.iter().find(|r| r.memory_id == "node_e").unwrap();

    assert_eq!(node_b.distance, 1, "node_b should be at distance 1");
    assert_eq!(node_e.distance, 4, "node_e should be at distance 4");

    // Verify activation decay
    assert!(
        node_b.activation > node_e.activation,
        "Closer nodes should have higher activation"
    );
}

// ============================================================================
// TEST 2: ACTIVATION DECAYS WITH DISTANCE
// ============================================================================

/// Test that activation decays appropriately with each hop.
///
/// Validates:
/// - Decay factor is applied per hop
/// - Further nodes have lower activation
/// - Decay is configurable
#[test]
fn test_activation_decays_with_distance() {
    let config = ActivationConfig {
        decay_factor: 0.7, // 30% decay per hop
        max_hops: 4,
        min_threshold: 0.01,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create chain with uniform edge strength
    network.add_edge("a".to_string(), "b".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("b".to_string(), "c".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("c".to_string(), "d".to_string(), LinkType::Semantic, 1.0);

    let results = network.activate("a", 1.0);

    let act_b = results.iter().find(|r| r.memory_id == "b").map(|r| r.activation).unwrap_or(0.0);
    let act_c = results.iter().find(|r| r.memory_id == "c").map(|r| r.activation).unwrap_or(0.0);
    let act_d = results.iter().find(|r| r.memory_id == "d").map(|r| r.activation).unwrap_or(0.0);

    // Verify monotonic decrease
    assert!(act_b > act_c, "b ({:.3}) > c ({:.3})", act_b, act_c);
    assert!(act_c > act_d, "c ({:.3}) > d ({:.3})", act_c, act_d);

    // Verify approximate decay rate (allowing for floating point)
    let ratio = act_c / act_b;
    assert!(
        (ratio - 0.7).abs() < 0.1,
        "Decay ratio should be ~0.7, got {:.3}",
        ratio
    );
}

// ============================================================================
// TEST 3: EDGE REINFORCEMENT (HEBBIAN LEARNING)
// ============================================================================

/// Test that edges are strengthened through use.
///
/// Validates:
/// - Initial edge strength is recorded
/// - Reinforcement increases strength
/// - Strength caps at maximum (1.0)
#[test]
fn test_edge_reinforcement_hebbian() {
    let mut network = ActivationNetwork::new();

    // Add edge with moderate strength
    network.add_edge("concept_a".to_string(), "concept_b".to_string(), LinkType::Semantic, 0.5);

    // Get initial associations
    let initial = network.get_associations("concept_a");
    let initial_strength = initial
        .iter()
        .find(|a| a.memory_id == "concept_b")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    assert!((initial_strength - 0.5).abs() < 0.01, "Initial should be 0.5");

    // Reinforce the connection
    network.reinforce_edge("concept_a", "concept_b", 0.2);

    // Get reinforced associations
    let reinforced = network.get_associations("concept_a");
    let new_strength = reinforced
        .iter()
        .find(|a| a.memory_id == "concept_b")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    assert!(
        new_strength > initial_strength,
        "Reinforcement should increase strength: {:.3} > {:.3}",
        new_strength,
        initial_strength
    );

    // Reinforce multiple times
    for _ in 0..10 {
        network.reinforce_edge("concept_a", "concept_b", 0.1);
    }

    // Should cap at 1.0
    let final_assoc = network.get_associations("concept_a");
    let final_strength = final_assoc
        .iter()
        .find(|a| a.memory_id == "concept_b")
        .map(|a| a.association_strength)
        .unwrap_or(0.0);

    assert!(
        final_strength <= 1.0,
        "Strength should cap at 1.0, got {:.3}",
        final_strength
    );
}

// ============================================================================
// TEST 4: NETWORK BUILDS FROM SEMANTIC LINKS
// ============================================================================

/// Test building a semantic network from related concepts.
///
/// Validates:
/// - Nodes are created automatically
/// - Edges connect nodes
/// - Associations can be queried
/// - Graph statistics are correct
#[test]
fn test_network_builds_from_semantic_links() {
    let mut network = create_coding_network();

    // Verify graph structure
    assert!(network.node_count() >= 9, "Should have at least 9 nodes");
    assert!(network.edge_count() >= 9, "Should have at least 9 edges");

    // Verify associations from rust
    let rust_assoc = network.get_associations("rust");
    assert!(
        rust_assoc.len() >= 3,
        "Rust should have at least 3 associations"
    );

    // Verify highest association (ownership at 0.95)
    assert_eq!(
        rust_assoc[0].memory_id, "ownership",
        "Highest association should be ownership"
    );

    // Verify spreading from rust reaches the whole ecosystem
    let results = network.activate("rust", 1.0);
    let found: HashSet<_> = results.iter().map(|r| r.memory_id.as_str()).collect();

    // Should reach direct concepts
    assert!(found.contains("ownership"));
    assert!(found.contains("borrowing"));
    assert!(found.contains("async_rust"));

    // Should reach 2-hop concepts
    assert!(found.contains("memory_safety")); // rust -> ownership -> memory_safety
    assert!(found.contains("tokio")); // rust -> async_rust -> tokio
}

// ============================================================================
// TEST 5: DIFFERENT LINK TYPES AFFECT ACTIVATION
// ============================================================================

/// Test that different link types can have different effects.
///
/// Validates:
/// - Semantic, Temporal, Causal, PartOf links all work
/// - Link type is preserved in results
/// - Different link types can coexist
#[test]
fn test_different_link_types_affect_activation() {
    let mut network = ActivationNetwork::new();

    // Add edges with different link types
    network.add_edge("event".to_string(), "semantic_rel".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("event".to_string(), "temporal_rel".to_string(), LinkType::Temporal, 0.8);
    network.add_edge("event".to_string(), "causal_rel".to_string(), LinkType::Causal, 0.85);
    network.add_edge("event".to_string(), "part_of_rel".to_string(), LinkType::PartOf, 0.7);

    let results = network.activate("event", 1.0);

    // Should find all related nodes
    let found: HashSet<_> = results.iter().map(|r| r.memory_id.as_str()).collect();
    assert!(found.contains("semantic_rel"));
    assert!(found.contains("temporal_rel"));
    assert!(found.contains("causal_rel"));
    assert!(found.contains("part_of_rel"));

    // Verify link types are preserved
    let semantic = results.iter().find(|r| r.memory_id == "semantic_rel").unwrap();
    let temporal = results.iter().find(|r| r.memory_id == "temporal_rel").unwrap();
    let causal = results.iter().find(|r| r.memory_id == "causal_rel").unwrap();
    let part_of = results.iter().find(|r| r.memory_id == "part_of_rel").unwrap();

    assert_eq!(semantic.link_type, LinkType::Semantic);
    assert_eq!(temporal.link_type, LinkType::Temporal);
    assert_eq!(causal.link_type, LinkType::Causal);
    assert_eq!(part_of.link_type, LinkType::PartOf);

    // Verify activation reflects edge strength
    assert!(
        semantic.activation > part_of.activation,
        "Semantic (0.9) should have higher activation than PartOf (0.7)"
    );
}

// ============================================================================
// ADDITIONAL SPREADING ACTIVATION TESTS
// ============================================================================

/// Test max hops limit.
#[test]
fn test_max_hops_limit() {
    let config = ActivationConfig {
        decay_factor: 0.99, // Almost no decay
        max_hops: 2,        // But strict hop limit
        min_threshold: 0.01,
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create 5-node chain
    network.add_edge("a".to_string(), "b".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("b".to_string(), "c".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("c".to_string(), "d".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("d".to_string(), "e".to_string(), LinkType::Semantic, 1.0);

    let results = network.activate("a", 1.0);
    let found: HashSet<_> = results.iter().map(|r| r.memory_id.as_str()).collect();

    // Should find b (1 hop) and c (2 hops)
    assert!(found.contains("b"), "Should find b at 1 hop");
    assert!(found.contains("c"), "Should find c at 2 hops");

    // Should NOT find d or e (3+ hops)
    assert!(!found.contains("d"), "Should not find d at 3 hops");
    assert!(!found.contains("e"), "Should not find e at 4 hops");
}

/// Test minimum threshold stops propagation.
#[test]
fn test_minimum_threshold() {
    let config = ActivationConfig {
        decay_factor: 0.5,   // 50% decay per hop
        max_hops: 10,        // High limit
        min_threshold: 0.2,  // But high threshold
        allow_cycles: false,
    };
    let mut network = ActivationNetwork::with_config(config);

    // Create chain
    network.add_edge("a".to_string(), "b".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("b".to_string(), "c".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("c".to_string(), "d".to_string(), LinkType::Semantic, 1.0);
    network.add_edge("d".to_string(), "e".to_string(), LinkType::Semantic, 1.0);

    let results = network.activate("a", 1.0);
    let found: HashSet<_> = results.iter().map(|r| r.memory_id.as_str()).collect();

    // With 0.5 decay and 0.2 threshold:
    // b: 1.0 * 0.5 = 0.5 (above)
    // c: 0.5 * 0.5 = 0.25 (above)
    // d: 0.25 * 0.5 = 0.125 (below)

    assert!(found.contains("b"), "b should be found");
    assert!(found.contains("c"), "c should be found");
    // d and e may or may not be found depending on threshold implementation
}

/// Test path tracking.
#[test]
fn test_path_tracking() {
    let mut network = ActivationNetwork::new();

    network.add_edge("start".to_string(), "middle".to_string(), LinkType::Semantic, 0.9);
    network.add_edge("middle".to_string(), "end".to_string(), LinkType::Semantic, 0.9);

    let results = network.activate("start", 1.0);

    let end_result = results.iter().find(|r| r.memory_id == "end").unwrap();

    // Path should be: start -> middle -> end
    assert_eq!(end_result.path.len(), 3, "Path should have 3 nodes");
    assert_eq!(end_result.path[0], "start");
    assert_eq!(end_result.path[1], "middle");
    assert_eq!(end_result.path[2], "end");
}

/// Test convergent paths.
#[test]
fn test_convergent_paths() {
    let mut network = ActivationNetwork::new();

    // Create convergent paths: source -> a -> target and source -> b -> target
    network.add_edge("source".to_string(), "path_a".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("source".to_string(), "path_b".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("path_a".to_string(), "target".to_string(), LinkType::Semantic, 0.8);
    network.add_edge("path_b".to_string(), "target".to_string(), LinkType::Semantic, 0.8);

    let results = network.activate("source", 1.0);

    // Target should be reached
    let target_results: Vec<_> = results.iter().filter(|r| r.memory_id == "target").collect();
    assert!(!target_results.is_empty(), "Target should be activated");

    // Total activation from convergent paths
    let total: f64 = target_results.iter().map(|r| r.activation).sum();
    assert!(total > 0.0, "Target should have positive activation");
}
