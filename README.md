<p align="center">
  <img src="chillin-with-lens.png" alt="Lens" width="400" />
</p>

<h3 align="center">Craft specialized agent perspectives.</h3>

<p align="center">
  <a href="#quick-start">Quick Start</a> &bull;
  <a href="#how-it-works">How It Works</a> &bull;
  <a href="#the-store">The Store</a> &bull;
  <a href="#build-your-own">Build Your Own</a>
</p>

---

## The Idea

Your agents are generalists. They see everything but focus on nothing.

**A Lens changes that.** It gives an agent a specialized perspective — a way to see one domain deeply instead of everything shallowly.

```
 Raw World                    Through a Lens
 ─────────                    ──────────────

 ┌─────────────┐              ┌─────────────┐
 │ Figma file  │              │ 12 components│
 │ API docs    │   ┌─────┐    │ 3 patterns   │
 │ Git history │──▶│LENS │──▶│ 1 action plan│
 │ Slack msgs  │   └─────┘    │ 0 noise      │
 │ Jira board  │              └─────────────┘
 └─────────────┘
```

Design decomposition. Emotion detection. Code archaeology. Build cleanup. Each Lens synthesizes domain knowledge into a focused capability that an agent works *through*.

## Quick Start

```rust
use async_trait::async_trait;
use lens::{Lens, LensContext, LensResult, Result};

struct MyLens;

#[async_trait]
impl Lens for MyLens {
    fn id(&self) -> &str { "my-lens" }
    fn name(&self) -> &str { "My Lens" }
    fn version(&self) -> &str { "0.1.0" }

    async fn execute(&self, ctx: LensContext) -> Result<LensResult> {
        Ok(LensResult::success(serde_json::json!({ "ok": true })))
    }
}
```

```toml
[dependencies]
lens = { git = "https://github.com/fuego-wtf/lens.git" }
```

Four methods. One trait. Your agent now has a perspective.

## How It Works

```
                         ┌──────────────────────────────┐
                         │         Lens Trait            │
                         │                              │
                         │  id()      ─── identity      │
                         │  name()    ─── display        │
                         │  version() ─── semver         │
                         │  execute() ─── do the thing   │
                         └──────────────┬───────────────┘
                                        │
                    ┌───────────────────┼───────────────────┐
                    │                   │                   │
              ┌─────▼─────┐     ┌───────▼──────┐    ┌──────▼──────┐
              │   Lens     │     │ StreamingLens │    │  MCP Server  │
              │            │     │               │    │    Lens      │
              │ One-shot   │     │ Real-time     │    │ Tool-based   │
              │ execution  │     │ event stream  │    │ interface    │
              └────────────┘     └───────────────┘    └─────────────┘
```

### Three Ways to Build

| Pattern | Trait | Use When |
|---------|-------|----------|
| **One-shot** | `Lens` | Input in, result out |
| **Streaming** | `StreamingLens` | Real-time events, progress, live data |
| **MCP Server** | `McpServerLens` | Expose tools via Model Context Protocol |

### Event System

Streaming lenses communicate through `LensEvent`:

```
 Started ──▶ Progress ──▶ Data ──▶ Checkpoint ──▶ Completed
    │           │          │           │
    │      "Phase 2/8"   {json}    "Approve?"
    │        (35%)                  [wait for input]
    │
  "decomposition"
```

### Manifest

Every Lens declares itself in `lens.toml`:

```toml
[lens]
id = "figma"
name = "Figma Decomposer"
version = "0.1.0"
description = "Breaks Figma designs into implementable components"

[permissions]
network = true
filesystem = false

[mcp_server]
entry = "target/release/figma"
runtime = "rust"
```

## The Store

Browse the Optics — our collection of ready-made Lenses:

```
store/
├── figma/               Figma design ──▶ component specs
├── vibe/                Voice input  ──▶ emotion signals
├── graphic-generator/   Prompts      ──▶ visual assets
├── unbuilder/           Built apps   ──▶ clean components
├── terminal/            Shell cmds   ──▶ validated execution
└── your-lens/           ???          ──▶ ???
```

| Lens | What It Sees | What It Produces |
|------|-------------|-----------------|
| **Figma** | Design files, tokens, layers | Component specs, pattern matches, Linear tasks |
| **Vibe** | Voice audio stream | Real-time emotion classification |
| **Graphic Generator** | Text prompts | Visual assets |
| **Unbuilder** | Built applications | Clean, reusable components |
| **Terminal** | Shell commands | Validated, safe execution |

## Build Your Own

### 1. Implement the trait

```rust
#[async_trait]
impl Lens for CodeArchaeologist {
    fn id(&self) -> &str { "code-archaeologist" }
    fn name(&self) -> &str { "Code Archaeologist" }
    fn version(&self) -> &str { "0.1.0" }

    async fn execute(&self, ctx: LensContext) -> Result<LensResult> {
        let repo = ctx.input.get("repo").unwrap();
        let findings = self.dig(repo).await?;
        Ok(LensResult::success(findings))
    }
}
```

### 2. Add streaming (optional)

```rust
#[async_trait]
impl StreamingLens for CodeArchaeologist {
    async fn execute_streaming(
        &self, ctx: LensContext
    ) -> Result<(LensResult, LensEventStream)> {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            tx.send(LensEvent::started("code-archaeologist", "excavation")).await;
            // ... dig through history, emit findings ...
            tx.send(LensEvent::data("code-archaeologist", "fossil", json!({
                "file": "auth.rs",
                "age": "847 days unchanged",
                "risk": "high"
            }))).await;
        });

        Ok((LensResult::success(json!({"status": "digging"})), Box::pin(rx)))
    }
}
```

### 3. Ship it

```toml
# lens.toml
[lens]
id = "code-archaeologist"
name = "Code Archaeologist"
version = "0.1.0"
```

### Feature Flags

```toml
# Just the trait (for Lens authors)
lens = { git = "https://github.com/fuego-wtf/lens.git" }

# Trait + discovery + dynamic loading (for host apps)
lens = { git = "https://github.com/fuego-wtf/lens.git", features = ["runtime"] }
```

The `runtime` feature adds:
- `LensDiscovery` — scan for installed Lenses
- `LensLoader` — dynamically load `.dylib`/`.so` at runtime
- `export_lens!` macro — FFI entry point for compiled Lenses

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Host Application                       │
│                                                             │
│   LensDiscovery ──▶ LensLoader ──▶ Lens::execute()         │
│        │                │               │                   │
│   ~/.lenses/        libloading      LensEvent stream        │
│   (installed)       (dynamic)       to UI / agent           │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│                    lens.toml + .dylib                        │
│                                                             │
│   [lens]              ┌──────────────────┐                  │
│   id = "figma"        │  create_lens()   │ ◀── FFI entry   │
│   name = "..."        │  -> Box<dyn Lens>│                  │
│   version = "0.1.0"   └──────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

## License

MIT — [Fuego Labs](https://fuego.wtf)
