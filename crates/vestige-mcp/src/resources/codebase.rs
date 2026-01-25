//! Codebase Resources
//!
//! codebase:// URI scheme resources for the MCP server.

use std::sync::Arc;
use tokio::sync::Mutex;

use vestige_core::{RecallInput, SearchMode, Storage};

/// Read a codebase:// resource
pub async fn read(storage: &Arc<Mutex<Storage>>, uri: &str) -> Result<String, String> {
    let path = uri.strip_prefix("codebase://").unwrap_or("");

    // Parse query parameters if present
    let (path, query) = match path.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (path, None),
    };

    match path {
        "structure" => read_structure(storage).await,
        "patterns" => read_patterns(storage, query).await,
        "decisions" => read_decisions(storage, query).await,
        _ => Err(format!("Unknown codebase resource: {}", path)),
    }
}

fn parse_codebase_param(query: Option<&str>) -> Option<String> {
    query.and_then(|q| {
        q.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k == "codebase" {
                Some(v.to_string())
            } else {
                None
            }
        })
    })
}

async fn read_structure(storage: &Arc<Mutex<Storage>>) -> Result<String, String> {
    let storage = storage.lock().await;

    // Get all pattern and decision nodes to infer structure
    // NOTE: We run separate queries because FTS5 sanitization removes OR operators
    // and wraps queries in quotes (phrase search), so "pattern OR decision" would
    // become a phrase search for "pattern decision" instead of matching either term.
    let search_terms = ["pattern", "decision", "architecture"];
    let mut all_nodes = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for term in &search_terms {
        let input = RecallInput {
            query: term.to_string(),
            limit: 100,
            min_retention: 0.0,
            search_mode: SearchMode::Keyword,
            valid_at: None,
        };

        for node in storage.recall(input).unwrap_or_default() {
            if seen_ids.insert(node.id.clone()) {
                all_nodes.push(node);
            }
        }
    }

    let nodes = all_nodes;

    // Extract unique codebases from tags
    let mut codebases: std::collections::HashSet<String> = std::collections::HashSet::new();

    for node in &nodes {
        for tag in &node.tags {
            if let Some(codebase) = tag.strip_prefix("codebase:") {
                codebases.insert(codebase.to_string());
            }
        }
    }

    let pattern_count = nodes.iter().filter(|n| n.node_type == "pattern").count();
    let decision_count = nodes.iter().filter(|n| n.node_type == "decision").count();

    let result = serde_json::json!({
        "knownCodebases": codebases.into_iter().collect::<Vec<_>>(),
        "totalPatterns": pattern_count,
        "totalDecisions": decision_count,
        "totalMemories": nodes.len(),
        "hint": "Use codebase://patterns?codebase=NAME or codebase://decisions?codebase=NAME for specific codebase context",
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

async fn read_patterns(storage: &Arc<Mutex<Storage>>, query: Option<&str>) -> Result<String, String> {
    let storage = storage.lock().await;
    let codebase = parse_codebase_param(query);

    let search_query = match &codebase {
        Some(cb) => format!("pattern codebase:{}", cb),
        None => "pattern".to_string(),
    };

    let input = RecallInput {
        query: search_query,
        limit: 50,
        min_retention: 0.0,
        search_mode: SearchMode::Keyword,
        valid_at: None,
    };

    let nodes = storage.recall(input).unwrap_or_default();

    let patterns: Vec<serde_json::Value> = nodes
        .iter()
        .filter(|n| n.node_type == "pattern")
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "content": n.content,
                "tags": n.tags,
                "retentionStrength": n.retention_strength,
                "createdAt": n.created_at.to_rfc3339(),
                "source": n.source,
            })
        })
        .collect();

    let result = serde_json::json!({
        "codebase": codebase,
        "total": patterns.len(),
        "patterns": patterns,
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

async fn read_decisions(storage: &Arc<Mutex<Storage>>, query: Option<&str>) -> Result<String, String> {
    let storage = storage.lock().await;
    let codebase = parse_codebase_param(query);

    let search_query = match &codebase {
        Some(cb) => format!("decision architecture codebase:{}", cb),
        None => "decision architecture".to_string(),
    };

    let input = RecallInput {
        query: search_query,
        limit: 50,
        min_retention: 0.0,
        search_mode: SearchMode::Keyword,
        valid_at: None,
    };

    let nodes = storage.recall(input).unwrap_or_default();

    let decisions: Vec<serde_json::Value> = nodes
        .iter()
        .filter(|n| n.node_type == "decision")
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "content": n.content,
                "tags": n.tags,
                "retentionStrength": n.retention_strength,
                "createdAt": n.created_at.to_rfc3339(),
                "source": n.source,
            })
        })
        .collect();

    let result = serde_json::json!({
        "codebase": codebase,
        "total": decisions.len(),
        "decisions": decisions,
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}
