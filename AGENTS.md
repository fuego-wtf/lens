# Agent Collaboration Policy

This document defines how AI agents collaborate with the `lens` codebase.

## Repository Context

- **Purpose**: Open-source Lens trait standard for specialized agent perspectives
- **Tech Stack**: Rust, async-trait, serde, tokio
- **Integration Points**: Desktop (lens runtime), Store lenses (`store/`)

## Agent Guidelines

### Code Changes

1. **No new components** unless explicitly requested by user
   - Extend existing trait patterns (`Lens`, `StreamingLens`, `McpServerLens`)
   - Reuse event system from `src/event.rs`
   - Follow manifest structure in `lens.toml`

2. **Wiring-first approach**
   - Lens trait defines clear execution contracts
   - No hidden behavior or implicit state
   - Event streams are explicit and typed

3. **Source-anchored modifications**
   - All lens capabilities traceable to domain synthesis
   - Document trait semantics in rustdoc
   - Maintain backward compatibility for store lenses

### Three Trait Patterns

| Pattern | Trait | Use When |
|---------|-------|----------|
| **One-shot** | `Lens` | Input in, result out |
| **Streaming** | `StreamingLens` | Real-time events, progress, live data |
| **MCP Server** | `McpServerLens` | Expose tools via Model Context Protocol |

### Architecture Constraints

- **Open-source**: Public API stability is critical
- **Trait stability**: Breaking changes require major version bump
- **Feature flags**: `runtime` feature for host apps, base for lens authors

### Event System

Follow existing event flow:
```
Started ──▶ Progress ──▶ Data ──▶ Checkpoint ──▶ Completed
```

### Cross-Repo References

- Desktop: Uses `runtime` feature for lens loading
- Store: Lenses at `store/` implement these traits
- Marketplace: Optics marketplace distributes lenses

## Constraints

- Do not break trait signatures without major version bump
- Do not add required trait methods (use default implementations)
- Do not modify event types without migration plan
- Do not add runtime dependencies to base trait
