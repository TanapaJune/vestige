//! Search Module
//!
//! Provides high-performance search capabilities:
//! - Vector search using HNSW (USearch)
//! - Keyword search using BM25/FTS5
//! - Hybrid search with RRF fusion
//! - Temporal-aware search
//! - Reranking for precision (GOD TIER 2026)

mod hybrid;
mod keyword;
mod reranker;
mod temporal;
mod vector;

pub use vector::{
    VectorIndex, VectorIndexConfig, VectorIndexStats, VectorSearchError, DEFAULT_CONNECTIVITY,
    DEFAULT_DIMENSIONS,
};

pub use keyword::{sanitize_fts5_query, KeywordSearcher};

pub use hybrid::{linear_combination, reciprocal_rank_fusion, HybridSearchConfig, HybridSearcher};

pub use temporal::TemporalSearcher;

// GOD TIER 2026: Reranking for +15-20% precision
pub use reranker::{
    Reranker, RerankerConfig, RerankerError, RerankedResult,
    DEFAULT_RERANK_COUNT, DEFAULT_RETRIEVAL_COUNT,
};
