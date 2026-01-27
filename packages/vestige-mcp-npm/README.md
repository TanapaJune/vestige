# @vestige/mcp

Vestige MCP Server - A synthetic hippocampus for AI assistants.

Built on 130 years of cognitive science research, Vestige provides biologically-inspired memory that decays, strengthens, and consolidates like the human mind.

## Installation

```bash
npm install -g vestige-mcp-server
```

This automatically downloads the correct binary for your platform (macOS, Linux, Windows) from GitHub releases.

### What gets installed

| Command | Description |
|---------|-------------|
| `vestige-mcp` | MCP server for Claude integration |
| `vestige` | CLI for stats, health checks, and maintenance |

### Verify installation

```bash
vestige health
```

## Usage with Claude Code

```bash
claude mcp add vestige vestige-mcp -s user
```

Then restart Claude.

## Usage with Claude Desktop

Add to your Claude Desktop configuration:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "vestige": {
      "command": "vestige-mcp"
    }
  }
}
```

## CLI Commands

```bash
vestige stats          # Memory statistics
vestige stats --states # Cognitive state distribution
vestige health         # System health check
vestige consolidate    # Run memory maintenance cycle
```

## Features

- **FSRS-6 Algorithm**: State-of-the-art spaced repetition for optimal memory retention
- **Dual-Strength Memory**: Bjork & Bjork (1992) - Storage + Retrieval strength model
- **Synaptic Tagging**: Memories become important retroactively (Frey & Morris 1997)
- **Semantic Search**: Local embeddings via nomic-embed-text-v1.5 (768 dimensions)
- **Local-First**: All data stays on your machine - no cloud, no API costs

## Storage & Memory

Vestige uses SQLite for storage. Your memories are stored on **disk**, not in RAM.

- **Database limit**: 216TB (SQLite theoretical max)
- **RAM usage**: ~64MB cache (configurable)
- **Typical usage**: 1 million memories ≈ 1-2GB on disk

You'll never run out of space. A heavy user creating 100 memories/day would use ~1.5GB after 10 years.

## Embeddings

On first use, Vestige downloads the nomic-embed-text-v1.5 model (~130MB). This is a one-time download and all subsequent operations are fully offline.

The model is stored in `.fastembed_cache/` in your working directory, or you can set a global location:

```bash
export FASTEMBED_CACHE_PATH="$HOME/.fastembed_cache"
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VESTIGE_DATA_DIR` | Data storage directory | `~/.vestige` |
| `VESTIGE_LOG_LEVEL` | Log verbosity | `info` |
| `FASTEMBED_CACHE_PATH` | Embeddings model location | `./.fastembed_cache` |

## Troubleshooting

### "Could not attach to MCP server vestige"

1. Verify binary exists: `which vestige-mcp`
2. Test directly: `vestige-mcp` (should wait for stdio input)
3. Check Claude logs: `~/Library/Logs/Claude/` (macOS)

### "vestige: command not found"

Reinstall the package:
```bash
npm install -g vestige-mcp-server
```

### Embeddings not downloading

The model downloads on first `ingest` or `search` operation. If Claude can't connect to the MCP server, no memory operations happen and no model downloads.

Fix the MCP connection first, then the model will download automatically.

## Supported Platforms

| Platform | Architecture |
|----------|--------------|
| macOS | ARM64 (Apple Silicon) |
| Linux | x86_64 |
| Windows | x86_64 |

## License

MIT
