# Cognition

Rust implementation of a cognitively-grounded agent memory system, extended into a broader Agent architecture (not memory-only).

## About This Repository

This repository is a Rust-first implementation inspired by CogMem design ideas, with a practical workspace structure for building:
- Long-term memory and retrieval
- Knowledge graph activation
- LLM-driven memory extraction
- Agent runtime components (skills, runtime orchestration, CLI, storage)

Reference to original CogMem source repository:
- https://github.com/triet4p/agent-memory-cognitive

## Positioning

Cognition is not only a memory library.

In addition to cognitive memory components, the project includes Agent-oriented modules for:
- Runtime and orchestration
- Skill integration
- LLM provider abstraction
- CLI-driven operations and debugging
- Python binding pathway (planned via cognition-py)

## Tech Stack

- Language: Rust (Edition 2024)
- Workspace: Cargo Workspace (multi-crate)
- Graph engine: petgraph
- Async runtime: tokio
- Storage: SQLite via sqlx
- Serialization: serde, serde_json
- Time and IDs: chrono, uuid

## Workspace Structure

- crates/cognition-core: Shared contracts, types, model schemas, error/result abstractions
- crates/cognition-graph: Unified graph model and spreading activation logic
- crates/cognition-storage: Durable memory persistence (nodes, edges, migration)
- crates/cognition-memory: Memory extraction pipeline and prompt registry
- crates/cognition-llm: LLM integration layer (scaffold)
- crates/cognition-runtime: Agent runtime layer (scaffold)
- crates/cognition-skills: Skill subsystem (scaffold)
- crates/cognition-py: Python interoperability layer (scaffold)
- cognition-cli: Command-line entrypoint for local workflows

## Current Cognitive Model Highlights

- 2-layer node schema for retrieval + lossless context:
  - narrative_fact (gist)
  - raw_snippet (verbatim)
- Intention lifecycle support:
  - status: planning, fulfilled, abandoned
  - deadline field for prospective memory
- Expanded memory network types:
  - world, experience, opinion, habit, intention, actioneffect
- Expanded edge types for cognitive transitions:
  - entity, temporal, semantic, causal, sr_link, ao_causal, transition
- SUM-based spreading activation with edge-type multiplier

## Agent Capabilities Direction

Beyond memory, the project is structured to evolve into an end-to-end Agent system:
- Ingestion and extraction from conversational input
- Retrieval and activation-guided recall
- Runtime decision loop and tool/skill execution
- Multi-provider LLM integration
- Cross-language integration for Python ecosystems

## Getting Started

### Prerequisites

- Rust toolchain (stable)
- Cargo

### Build

Run from repository root:

```bash
cargo check --workspace
```

### Run root binary

```bash
cargo run
```

### Run CLI binary

```bash
cargo run -p cognition-cli
```

## Development Status

This codebase is under active development.

Some crates are production-oriented while others are scaffolded for upcoming Agent features.
The workspace layout is intentionally modular to support iterative evolution from memory core to full Agent runtime.

## Configuration

- Base config file: config/default.yaml
- Environment file: .env (local)

## Documentation

Project documents are available in docs, including:
- CogMem.md
- cognition-core.md
- coginition-storage.md
- Project-Structure.md
- Project-Proposal.md

## Contributing

Contributions are welcome.

Recommended flow:
1. Open an issue describing the change
2. Create a focused branch
3. Add tests for behavior changes
4. Submit a PR with clear rationale and impact notes

## License

No license file is currently declared in this repository.
Please add a LICENSE file before public distribution.
