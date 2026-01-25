# Vestige MCP Server

A bleeding-edge Rust MCP (Model Context Protocol) server for Vestige - providing Claude and other AI assistants with long-term memory capabilities.

## Features

- **FSRS-6 Algorithm**: State-of-the-art spaced repetition (21 parameters, personalized decay)
- **Dual-Strength Memory Model**: Based on Bjork & Bjork 1992 cognitive science research
- **Local Semantic Embeddings**: nomic-embed-text-v1.5 (768d) via fastembed v5 (no external API)
- **HNSW Vector Search**: USearch-based, 20x faster than FAISS
- **Hybrid Search**: BM25 + semantic with RRF fusion
- **Codebase Memory**: Remember patterns, decisions, and context

## Installation

```bash
cd /path/to/vestige/crates/vestige-mcp
cargo build --release
```

Binary will be at `target/release/vestige-mcp`

## Claude Desktop Configuration

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "vestige": {
      "command": "/path/to/vestige-mcp"
    }
  }
}
```

## Available Tools

### Core Memory

| Tool | Description |
|------|-------------|
| `ingest` | Add new knowledge to memory |
| `recall` | Search and retrieve memories |
| `semantic_search` | Find conceptually similar content |
| `hybrid_search` | Combined keyword + semantic search |
| `get_knowledge` | Retrieve a specific memory by ID |
| `delete_knowledge` | Delete a memory |
| `mark_reviewed` | Review with FSRS rating (1-4) |

### Statistics & Maintenance

| Tool | Description |
|------|-------------|
| `get_stats` | Memory system statistics |
| `health_check` | System health status |
| `run_consolidation` | Apply decay, generate embeddings |

### Codebase Tools

| Tool | Description |
|------|-------------|
| `remember_pattern` | Remember code patterns |
| `remember_decision` | Remember architectural decisions |
| `get_codebase_context` | Get patterns and decisions |

## Available Resources

### Memory Resources

| URI | Description |
|-----|-------------|
| `memory://stats` | Current statistics |
| `memory://recent?n=10` | Recent memories |
| `memory://decaying` | Low retention memories |
| `memory://due` | Memories due for review |

### Codebase Resources

| URI | Description |
|-----|-------------|
| `codebase://structure` | Known codebases |
| `codebase://patterns` | Remembered patterns |
| `codebase://decisions` | Architectural decisions |

## Example Usage (with Claude)

```
User: Remember that we decided to use FSRS-6 instead of SM-2 because it's 20-30% more efficient.

Claude: [calls remember_decision]
I've recorded that architectural decision.

User: What decisions have we made about algorithms?

Claude: [calls get_codebase_context]
I found 1 decision:
- We decided to use FSRS-6 instead of SM-2 because it's 20-30% more efficient.
```

## Data Storage

- Database: `~/Library/Application Support/com.vestige.mcp/vestige-mcp.db` (macOS)
- Uses SQLite with FTS5 for full-text search
- Vector embeddings stored in separate table

## Protocol

- JSON-RPC 2.0 over stdio
- MCP Protocol Version: 2024-11-05
- Logging to stderr (stdout reserved for JSON-RPC)

## License

MIT
