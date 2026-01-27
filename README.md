# Vestige

**Memory that fades like yours does.**

[![GitHub stars](https://img.shields.io/github/stars/samvallad33/vestige?style=social)](https://github.com/samvallad33/vestige)
[![Release](https://img.shields.io/github/v/release/samvallad33/vestige)](https://github.com/samvallad33/vestige/releases/latest)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![MCP Compatible](https://img.shields.io/badge/MCP-compatible-green)](https://modelcontextprotocol.io)

> The only MCP memory server built on cognitive science. FSRS-6 spaced repetition, spreading activation, synaptic tagging—all running 100% local.

---

## Quick Start

### 1. Download

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/samvallad33/vestige/releases/latest/download/vestige-mcp-aarch64-apple-darwin.tar.gz | tar -xz
sudo mv vestige-mcp vestige vestige-restore /usr/local/bin/
```

<details>
<summary>Other platforms</summary>

**macOS (Intel):**
```bash
curl -L https://github.com/samvallad33/vestige/releases/latest/download/vestige-mcp-x86_64-apple-darwin.tar.gz | tar -xz
sudo mv vestige-mcp vestige vestige-restore /usr/local/bin/
```

**Linux:**
```bash
curl -L https://github.com/samvallad33/vestige/releases/latest/download/vestige-mcp-x86_64-unknown-linux-gnu.tar.gz | tar -xz
sudo mv vestige-mcp vestige vestige-restore /usr/local/bin/
```

**Windows:** Download from [Releases](https://github.com/samvallad33/vestige/releases/latest)

**Build from source:**
```bash
git clone https://github.com/samvallad33/vestige && cd vestige
cargo build --release
sudo cp target/release/vestige-mcp /usr/local/bin/
```
</details>

### 2. Connect to Claude

```bash
claude mcp add vestige vestige-mcp -s user
```

### 3. Restart & Test

Restart Claude, then:

> "Remember that I prefer TypeScript over JavaScript"

New session:

> "What are my coding preferences?"

**It remembers.**

---

## Why Vestige?

| Problem | Solution |
|---------|----------|
| AI forgets between sessions | Persistent memory with intelligent retrieval |
| RAG dumps irrelevant context | **Prediction Error Gating** auto-decides CREATE/UPDATE/SUPERSEDE |
| Memory bloat eats tokens | **FSRS-6 decay** naturally fades unused memories |
| No idea what AI "knows" | `search` tool lets you query anytime |
| Privacy concerns | **100% local** after initial setup |

---

## Tools

| Tool | Description |
|------|-------------|
| `search` | Unified search (keyword + semantic + hybrid) |
| `smart_ingest` | Intelligent ingestion with duplicate detection |
| `ingest` | Simple memory storage |
| `memory` | Get, delete, or check memory state |
| `codebase` | Remember patterns and architectural decisions |
| `intention` | Set reminders and future triggers |
| `promote_memory` | Mark memory as helpful (strengthens) |
| `demote_memory` | Mark memory as wrong (weakens) |

---

## Make Claude Use Vestige Automatically

Add this to your `CLAUDE.md`:

```markdown
## Vestige Memory System

At the start of every conversation, check Vestige for context:
1. Recall user preferences and instructions
2. Recall relevant project context
3. Operate in proactive memory mode - save important info without being asked
```

### Trigger Words

| User Says | Claude Does |
|-----------|-------------|
| "Remember this" | `smart_ingest` immediately |
| "I prefer..." / "I always..." | Save as preference |
| "Remind me..." | Create `intention` |
| "This is important" | `smart_ingest` + `promote_memory` |

[Full CLAUDE.md templates →](docs/CLAUDE-SETUP.md)

---

## Troubleshooting

<details>
<summary>"Command not found" after installation</summary>

Ensure `vestige-mcp` is in PATH:
```bash
which vestige-mcp
```

Or use full path in Claude config:
```bash
claude mcp add vestige /full/path/to/vestige-mcp -s user
```
</details>

<details>
<summary>Model cache location</summary>

The embedding model (~130MB) is cached in platform-specific directories:
- **macOS**: `~/Library/Caches/com.vestige.core/fastembed`
- **Linux**: `~/.cache/vestige/fastembed`
- **Windows**: `%LOCALAPPDATA%\vestige\cache\fastembed`

Override with: `export FASTEMBED_CACHE_PATH=/custom/path`
</details>

<details>
<summary>Model download fails</summary>

First run requires internet (~130MB). If behind proxy:
```bash
export HTTPS_PROXY=your-proxy:port
```
</details>

[More troubleshooting →](docs/FAQ.md#troubleshooting)

---

## Documentation

| Document | Contents |
|----------|----------|
| [FAQ](docs/FAQ.md) | 30+ answers to common questions |
| [How It Works](docs/SCIENCE.md) | FSRS-6, dual-strength memory, the neuroscience |
| [Storage Modes](docs/STORAGE.md) | Global, per-project, multi-Claude setup |
| [CLAUDE.md Setup](docs/CLAUDE-SETUP.md) | Templates for proactive memory use |
| [Configuration](docs/CONFIGURATION.md) | CLI commands, environment variables |
| [Changelog](CHANGELOG.md) | Version history |

---

## CLI Commands

```bash
vestige stats              # Memory statistics
vestige health             # System health check
vestige consolidate        # Run memory maintenance
vestige restore <file>     # Restore from backup
```

---

## Contributing

Issues and PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT OR Apache-2.0 (dual-licensed)

---

<p align="center">
  <i>Built by <a href="https://github.com/samvallad33">@samvallad33</a></i>
</p>
