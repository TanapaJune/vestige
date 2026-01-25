# @vestige/mcp

Vestige MCP Server - A synthetic hippocampus for AI assistants.

Built on 130 years of cognitive science research, Vestige provides biologically-inspired memory that decays, strengthens, and consolidates like the human mind.

## Installation

```bash
npm install -g @vestige/mcp
```

## Usage with Claude Desktop

Add to your Claude Desktop configuration:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "vestige": {
      "command": "vestige-mcp",
      "args": ["--project", "."]
    }
  }
}
```

## Features

- **FSRS-6 Algorithm**: State-of-the-art spaced repetition for optimal memory retention
- **Dual-Strength Memory**: Bjork & Bjork (1992) - Storage + Retrieval strength model
- **Sleep Consolidation**: Bio-inspired memory optimization cycles
- **Semantic Search**: Local embeddings for intelligent memory retrieval
- **Local-First**: All data stays on your machine

## MCP Tools

| Tool | Description |
|------|-------------|
| `remember` | Store a new memory with importance scoring |
| `recall` | Retrieve memories by semantic similarity |
| `search` | Full-text search across all memories |
| `consolidate` | Trigger memory consolidation (sleep cycle) |
| `get_context` | Get relevant context for current project |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VESTIGE_DATA_DIR` | Data storage directory | `~/.vestige` |
| `VESTIGE_LOG_LEVEL` | Log verbosity | `info` |

## License

MIT
