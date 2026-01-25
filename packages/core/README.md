# Vestige

[![npm version](https://img.shields.io/npm/v/vestige-mcp.svg)](https://www.npmjs.com/package/vestige-mcp)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-blue.svg)](https://modelcontextprotocol.io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Git Blame for AI Thoughts** - Memory that decays, strengthens, and discovers connections like the human mind.

![Vestige Demo](./docs/assets/hero-demo.gif)

## Why Vestige?

| Feature | Vestige | Mem0 | Zep | Letta |
|---------|--------|------|-----|-------|
| FSRS-5 spaced repetition | Yes | No | No | No |
| Dual-strength memory | Yes | No | No | No |
| Sentiment-weighted retention | Yes | No | Yes | No |
| Local-first (no cloud) | Yes | No | No | No |
| Git context capture | Yes | No | No | No |
| Semantic connections | Yes | Limited | Yes | Yes |
| Free & open source | Yes | Freemium | Freemium | Yes |

## Quickstart

```bash
# Install
npx vestige-mcp init

# Add to Claude Desktop config
# ~/.config/claude/claude_desktop_config.json (Mac/Linux)
# %APPDATA%\Claude\claude_desktop_config.json (Windows)
{
  "mcpServers": {
    "vestige": {
      "command": "npx",
      "args": ["vestige-mcp"]
    }
  }
}

# Restart Claude Desktop - done!
```

## Key Concepts

### Cognitive Science Foundation

Vestige implements proven memory science:

- **FSRS-5**: State-of-the-art spaced repetition algorithm (powers Anki's 100M+ users)
- **Dual-Strength Memory**: Separate storage and retrieval strength (Bjork & Bjork, 1992)
- **Ebbinghaus Decay**: Memories fade naturally without reinforcement using `R = e^(-t/S)`
- **Sentiment Weighting**: Emotional memories decay slower via AFINN-165 lexicon analysis

### Developer Features

- **Git-Blame for Thoughts**: Every memory captures git branch, commit hash, and changed files
- **REM Cycle**: Background connection discovery between unrelated memories
- **Shadow Self**: Queue unsolved problems for future inspiration when new knowledge arrives

## MCP Tools

| Tool | Description |
|------|-------------|
| `ingest` | Store knowledge with metadata (source, people, tags, git context) |
| `recall` | Search memories by query with relevance ranking |
| `get_knowledge` | Retrieve specific memory by ID |
| `get_related` | Find connected nodes via graph traversal |
| `mark_reviewed` | Reinforce a memory (triggers spaced repetition) |
| `remember_person` | Add/update person in your network |
| `get_person` | Retrieve person details and relationship health |
| `daily_brief` | Get summary of memory state and review queue |
| `health_check` | Check database health with recommendations |
| `backup` | Create timestamped database backup |

## MCP Resources

| Resource | URI | Description |
|----------|-----|-------------|
| Recent memories | `memory://knowledge/recent` | Last 20 stored memories |
| Decaying memories | `memory://knowledge/decaying` | Memories below 50% retention |
| People network | `memory://people/network` | Your relationship graph |
| System context | `memory://context` | Active window, git branch, clipboard |

## CLI Commands

```bash
# Memory
vestige stats              # Quick overview
vestige recall "query"     # Search memories
vestige review             # Show due for review

# Ingestion
vestige eat <url|path>     # Ingest documentation

# REM Cycle
vestige dream              # Discover connections
vestige dream --dry-run    # Preview only

# Shadow Self
vestige problem "desc"     # Log unsolved problem
vestige problems           # List open problems
vestige solve <id> "fix"   # Mark solved

# Context
vestige context            # Show current context
vestige watch              # Start context daemon

# Maintenance
vestige backup             # Create backup
vestige optimize           # Vacuum and reindex
vestige decay              # Apply memory decay
```

## Configuration

Create `~/.vestige/config.json`:

```json
{
  "fsrs": {
    "desiredRetention": 0.9,
    "maxStability": 365
  },
  "rem": {
    "enabled": true,
    "maxAnalyze": 50,
    "minStrength": 0.3
  },
  "decay": {
    "sentimentBoost": 2.0
  }
}
```

### Database Locations

| File | Path |
|------|------|
| Main database | `~/.vestige/vestige.db` |
| Shadow Self | `~/.vestige/shadow.db` |
| Backups | `~/.vestige/backups/` |
| Context | `~/.vestige/context.json` |

## How It Works

### Memory Decay

```
Retention = e^(-days/stability)

New memory:     S=1.0  -> 37% after 1 day
Reviewed once:  S=2.5  -> 67% after 1 day
Reviewed 3x:    S=15.6 -> 94% after 1 day
Emotional:      S x 1.85 boost
```

### REM Cycle Connections

The REM cycle discovers hidden relationships:

| Connection Type | Trigger | Strength |
|----------------|---------|----------|
| `entity_shared` | Same people mentioned | 0.5 + (count * 0.2) |
| `concept_overlap` | 2+ shared concepts | 0.4 + (count * 0.15) |
| `keyword_similarity` | Jaccard > 15% | similarity * 2 |
| `temporal_proximity` | Same day + overlap | 0.3 |

## Documentation

- [API Reference](./docs/api.md) - Full TypeScript API documentation
- [Configuration](./docs/configuration.md) - All config options
- [Architecture](./docs/architecture.md) - System design and data flow
- [Cognitive Science](./docs/cognitive-science.md) - The research behind Vestige

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

MIT - see [LICENSE](./LICENSE)

---

**Vestige**: The only AI memory system built on 130 years of cognitive science research.
