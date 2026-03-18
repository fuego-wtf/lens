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

---

## Authority Boundaries (Task 69)

**Purpose**: Define clear ownership boundaries between Desktop, Sync, Lens, Backyard, and ID.

### System Authority Matrix

| System | Owns | Authority | API Family |
|--------|------|-----------|------------|
| **Desktop** | Lens runtime host, UI, user interactions | Executes lenses locally | Internal |
| **Sync** | MCP config sync, cross-tool state | Distributes lens configs | Internal |
| **Lens (crate)** | Trait definitions, event system | Open-source standard | Public |
| **Backyard** | Admin BFF, deployment APIs, thread state | Backend authority | Admin + Public |
| **ID** | Auth, billing, OAuth, device registry | Identity authority | Public |

### API Family Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                    PUBLIC APIs (api.graphyn.xyz)                │
│                                                                 │
│  • ID Service: /auth/*, /billing/*, /oauth/*                   │
│  • Backyard: /threads/*, /agents/* (runtime entry points)      │
│  • Lens: Open-source trait (published to crates.io)            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    ADMIN APIs (admin.graphyn.xyz)               │
│                                                                 │
│  • Backyard Admin: /admin/*, deployment control                │
│  • Desktop → Admin BFF → Backend services                      │
│  • Deployment operations (T-DEPLOY-*)                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    INTERNAL (no external access)                │
│                                                                 │
│  • Desktop: Local lens execution, file access                  │
│  • Sync: MCP config distribution                               │
│  • Vault: Secret management (see vault/AGENTS.md)              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Responsibility Boundaries

#### Desktop (Tauri/React)
- **Owns**: Local lens execution, UI/UX, file tree, terminal
- **Does NOT own**: Backend state, deployment, auth decisions
- **API Access**: Public APIs only (via ID/Backyard public endpoints)
- **Admin Access**: Via Admin BFF proxy only

#### Sync (`packages/sync`)
- **Owns**: MCP config sync, toolchain distribution
- **Does NOT own**: Lens trait definitions, marketplace
- **API Access**: Internal (desktop plugin)

#### Lens (`lens/` crate)
- **Owns**: `Lens`, `StreamingLens`, `McpServerLens` traits
- **Does NOT own**: Lens implementations, execution runtime
- **API Access**: Public (open-source, crates.io)

#### Backyard (Go/Encore)
- **Owns**: Thread state, agent deployment, admin BFF
- **Does NOT own**: Auth, billing, device registry
- **API Access**: Public (`/threads/*`, `/agents/*`) + Admin (`/admin/*`)
- **Deployment Control**: Admin-only (T-DEPLOY-001 boundary)

#### ID Service (Hono)
- **Owns**: Auth, billing, OAuth, device registry
- **Does NOT own**: Thread state, deployment operations
- **API Access**: Public (`/auth/*`, `/billing/*`, `/oauth/*`)

### Deployment Boundary (T-DEPLOY-001)

**Rule**: Deployment operations are admin-only.

| Operation | API Family | Route |
|----------|-----------|-------|
| Deploy agent | Admin | `POST /admin/deploy` |
| Check deployment status | Admin | `GET /admin/deploy/{id}` |
| Runtime entry (public) | Public | `GET /api/agents/{id}` |
| Thread access (public) | Public | `GET /api/threads/{id}` |

**Desktop Flow**:
```
Desktop → Admin BFF → /admin/deploy → Letta/Binary
Desktop → Public API → /api/agents/{id} → Runtime entry
```

### Cross-System Contracts

| From | To | Contract | Location |
|------|-----|----------|----------|
| Desktop | Backyard | Admin BFF proxy | `backyard/admin-bff` |
| Desktop | ID | OAuth flow | `id/oauth` |
| Sync | Desktop | MCP config | `packages/sync` |
| Lens | Desktop | Trait implementation | `lens/src/lib.rs` |
| Backyard | ID | Device validation | `backyard/admin/device.go` |

### Constraints

- Do not break trait signatures without major version bump
- Do not add required trait methods (use default implementations)
- Do not modify event types without migration plan
- Do not add runtime dependencies to base trait
