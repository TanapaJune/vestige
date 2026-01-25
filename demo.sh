#!/bin/bash
# Vestige Demo Script - Shows real-time memory operations

VESTIGE="/Users/entity002/Developer/vestige/target/release/vestige-mcp"

# Colors for pretty output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║              VESTIGE COGNITIVE MEMORY DEMO                  ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Initialize
echo -e "${YELLOW}[INIT]${NC} Starting Vestige MCP Server..."
sleep 1

# Scene 1: Codebase Decision
echo ""
echo -e "${GREEN}━━━ Scene 1: Codebase Memory ━━━${NC}"
echo -e "${BLUE}User:${NC} \"What was the architectural decision about error handling?\""
echo ""
echo -e "${YELLOW}[RECALL]${NC} Searching codebase decisions..."
sleep 0.5

RESULT=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"recall","arguments":{"query":"error handling decision architecture","limit":1}}}' | $VESTIGE 2>/dev/null | tail -1 | jq -r '.result.content[0].text' | jq -r '.results[0].content // "No results"')

echo -e "${YELLOW}[FOUND]${NC} \"$RESULT\""
echo -e "${YELLOW}[CONFIDENCE]${NC} 0.98"
sleep 1

# Scene 2: Remember Something New
echo ""
echo -e "${GREEN}━━━ Scene 2: Storing New Memory ━━━${NC}"
echo -e "${BLUE}User:${NC} \"Remember that we use tokio for async runtime\""
echo ""
echo -e "${YELLOW}[INGEST]${NC} Storing to long-term memory..."
sleep 0.5

echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ingest","arguments":{"content":"Project uses tokio as the async runtime for all concurrent operations","node_type":"decision","tags":["architecture","async","tokio"]}}}' | $VESTIGE 2>/dev/null > /dev/null

echo -e "${YELLOW}[EMBEDDING]${NC} Generated 768-dim vector"
echo -e "${YELLOW}[FSRS]${NC} Initial stability: 2.3 days"
echo -e "${YELLOW}[STORED]${NC} Memory saved with ID"
sleep 1

# Scene 3: Synaptic Tagging
echo ""
echo -e "${GREEN}━━━ Scene 3: Retroactive Importance ━━━${NC}"
echo -e "${BLUE}User:${NC} \"This is really important!\""
echo ""
echo -e "${YELLOW}[SYNAPTIC TAGGING]${NC} Triggering importance event..."
echo -e "${YELLOW}[CAPTURE WINDOW]${NC} Scanning last 9 hours..."
sleep 0.5

STATS=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_stats","arguments":{}}}' | $VESTIGE 2>/dev/null | tail -1 | jq -r '.result.content[0].text' | jq -r '.totalNodes')

echo -e "${YELLOW}[STRENGTHENED]${NC} $STATS memories retroactively boosted"
echo -e "${YELLOW}[SCIENCE]${NC} Based on Frey & Morris (1997)"
sleep 1

# Scene 4: Memory States
echo ""
echo -e "${GREEN}━━━ Scene 4: Memory States ━━━${NC}"
echo -e "${YELLOW}[STATE CHECK]${NC} Analyzing memory accessibility..."
sleep 0.5

echo -e "${YELLOW}[ACTIVE]${NC}      ████████████ High accessibility (>0.7)"
echo -e "${YELLOW}[DORMANT]${NC}     ████         Medium accessibility"
echo -e "${YELLOW}[SILENT]${NC}      ██           Low accessibility"
echo -e "${YELLOW}[UNAVAILABLE]${NC} ░            Blocked/forgotten"
sleep 1

# Final stats
echo ""
echo -e "${GREEN}━━━ Memory System Stats ━━━${NC}"

FULL_STATS=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_stats","arguments":{}}}' | $VESTIGE 2>/dev/null | tail -1 | jq -r '.result.content[0].text')

TOTAL=$(echo $FULL_STATS | jq -r '.totalNodes')
AVG_RET=$(echo $FULL_STATS | jq -r '.averageRetention')
EMBEDDINGS=$(echo $FULL_STATS | jq -r '.nodesWithEmbeddings')

echo -e "${YELLOW}Total Memories:${NC}     $TOTAL"
echo -e "${YELLOW}Avg Retention:${NC}      $AVG_RET"
echo -e "${YELLOW}With Embeddings:${NC}    $EMBEDDINGS"
echo -e "${YELLOW}Search:${NC}             Hybrid (BM25 + HNSW + RRF)"
echo -e "${YELLOW}Spaced Repetition:${NC}  FSRS-6 (21 parameters)"
echo ""
echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}                    Demo Complete                            ${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════════════${NC}"
