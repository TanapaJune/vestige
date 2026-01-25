//! Memory Consolidation Module
//!
//! Implements sleep-inspired memory consolidation:
//! - Decay weak memories
//! - Promote emotional/important memories
//! - Generate embeddings
//! - Prune very weak memories (optional)

mod sleep;

pub use sleep::SleepConsolidation;
