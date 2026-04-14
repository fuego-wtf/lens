# Developing a Lens

A step-by-step guide to building, testing, and shipping a Graphyn Lens.

---

## What is a Lens?

A Lens gives an agent a specialized perspective. Instead of seeing everything shallowly, an agent working *through* a Lens sees one domain deeply.

A Lens is:
- A Rust crate implementing the `Lens` trait (or `StreamingLens` / `McpServerLens`)
- A `lens.toml` manifest declaring metadata, surface, shortcuts, and permissions
- Optionally, a frontend component for rendering in Graphyn Desktop

---

## Prerequisites

- Rust toolchain (1.75+)
- Familiarity with `async_trait` and `serde`
- Graphyn Desktop (for testing surfaces and installation)

---

## 1. Scaffold Your Lens

Create a new directory in `store/` (or anywhere on your machine):

```bash
mkdir store/my-lens && cd store/my-lens
cargo init --lib
```

Add the `lens` dependency:

```toml
# Cargo.toml
[dependencies]
lens = { git = "https://github.com/fuego-wtf/lens.git" }
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

---

## 2. Write the Manifest

Create `lens.toml` in your lens root. This is how Graphyn discovers and configures your lens.

### Minimal manifest

```toml
[lens]
id = "my-lens"
name = "My Lens"
version = "0.1.0"
description = "What this lens does in one sentence"
```

### Full manifest (all options)

```toml
[lens]
id = "my-lens"
name = "My Lens"
version = "0.1.0"
description = "What this lens does in one sentence"
authors = ["Your Name"]
manifest_version = 2
min_framework_version = "0.5.0"

# Surface type: where this lens renders
# Options: "pane" (default), "pack", "tray", "desktop_app"
surface = "pane"
# Multi-surface support (optional — lens can render on multiple surfaces)
surfaces = ["pane", "pack"]

# Capabilities this lens provides
capabilities = ["semantic_search", "code_analysis"]

# Global keyboard shortcuts
[[shortcuts]]
id = "open"
action = "launch"
combo = "CommandOrControl+Shift+M"
global = true
description = "Open My Lens"

# Security & permissions
[security]
sandbox = "restricted"  # "restricted" | "network" | "full"
permissions = [
  "network:api.example.com",
  "fs:read:~/Documents",
]

# MCP server entry point (if this lens exposes tools)
[mcp_server]
entry = "target/release/my-lens"
runtime = "rust"

# Custom message types for Desktop rendering
[[message_types]]
key = "analysis_result"
component = "components/AnalysisResult.tsx"
description = "Displays analysis results with charts"
```

### Surface types explained

| Surface | Where it renders | When to use |
|---------|-----------------|-------------|
| `pane` | Full tab in the editor area | Default. Thread views, editors, dashboards. |
| `pack` | Sidebar panel (left or right dock) | Always-visible tools. File explorers, status panels. |
| `tray` | Ephemeral OS-level popup window | Quick-capture, notifications, transient input. |
| `desktop_app` | Standalone OS window | Full applications that need their own space. |

See `docs/SURFACE_SPEC_V1.md` for the complete surface architecture.

---

## 3. Implement the Trait

Choose your pattern based on what your lens does:

### Pattern A: One-shot (most common)

Input in, result out. Use for analysis, transformation, generation.

```rust
// src/lib.rs
use async_trait::async_trait;
use lens::{Lens, LensContext, LensResult, Result};

pub struct MyLens;

#[async_trait]
impl Lens for MyLens {
    fn id(&self) -> &str { "my-lens" }
    fn name(&self) -> &str { "My Lens" }
    fn version(&self) -> &str { "0.1.0" }

    async fn execute(&self, ctx: LensContext) -> Result<LensResult> {
        // Access input data
        let query = ctx.input
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // Do your domain-specific work
        let result = analyze(query).await?;

        Ok(LensResult::success(serde_json::json!({
            "findings": result,
            "query": query,
        })))
    }
}

async fn analyze(query: &str) -> Result<Vec<String>> {
    // Your domain logic here
    Ok(vec![format!("Found something for: {}", query)])
}
```

### Pattern B: Streaming

Real-time events, progress updates, live data. Use for long-running operations.

```rust
use async_trait::async_trait;
use lens::{StreamingLens, LensContext, LensResult, LensEvent, LensEventStream, Result};
use tokio::sync::mpsc;

pub struct MyStreamingLens;

#[async_trait]
impl StreamingLens for MyStreamingLens {
    async fn execute_streaming(
        &self,
        ctx: LensContext,
    ) -> Result<(LensResult, LensEventStream)> {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let _ = tx.send(LensEvent::started("my-lens", "processing")).await;

            for i in 0..10 {
                let _ = tx.send(LensEvent::progress(
                    "my-lens",
                    "processing",
                    (i + 1) as f64 / 10.0,
                )).await;

                // Simulate work
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                let _ = tx.send(LensEvent::data(
                    "my-lens",
                    "result_chunk",
                    serde_json::json!({ "chunk": i }),
                )).await;
            }

            let _ = tx.send(LensEvent::completed("my-lens", "processing")).await;
        });

        Ok((
            LensResult::success(serde_json::json!({"status": "streaming"})),
            Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)),
        ))
    }
}
```

Event lifecycle:
```
Started ──> Progress ──> Data ──> Checkpoint ──> Completed
                          |           |
                       {json}     "Approve?"
                                 [wait for input]
```

### Pattern C: MCP Server

Expose tools via Model Context Protocol. Use when agents need to call your lens.

```rust
use async_trait::async_trait;
use lens::{
    McpServerLens, McpTool, McpToolBuilder, McpToolResponse, McpContent,
    LensContext, LensResult, Result,
};

pub struct MyMcpLens;

#[async_trait]
impl McpServerLens for MyMcpLens {
    fn tools(&self) -> Vec<McpTool> {
        vec![
            McpToolBuilder::new("search", "Search the knowledge base")
                .string_param("query", "Search query", true)
                .number_param("limit", "Max results", false)
                .build(),
        ]
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
        _ctx: LensContext,
    ) -> Result<McpToolResponse> {
        match name {
            "search" => {
                let query = arguments["query"].as_str().unwrap_or("");
                let results = do_search(query).await?;
                Ok(McpToolResponse::success(vec![
                    McpContent::text(format!("Found {} results", results.len())),
                ]))
            }
            _ => Ok(McpToolResponse::error(format!("Unknown tool: {}", name))),
        }
    }
}
```

---

## 4. Tray Surface (Optional)

If your lens declares `surface = "tray"`, it renders as an ephemeral OS-level popup. Graphyn Desktop handles the window lifecycle — you provide the UI component.

### How tray surfaces work

1. Your `lens.toml` declares `surface = "tray"`
2. Desktop creates a window labeled `lens-surface-{your-lens-id}`
3. Your frontend component renders inside that window
4. Global shortcuts (from `[[shortcuts]]` with `global = true`) toggle the tray

### Example: Quick lens (reference implementation)

```toml
# lens.toml
[lens]
id = "quick"
name = "Quick"
version = "0.1.0"
surface = "tray"
surfaces = ["tray"]

[[shortcuts]]
id = "launcher"
action = "launch"
combo = "CommandOrControl+Alt+Space"
global = true
```

The Desktop generic `LensSurfacePane` reads the lens ID from the window label and renders the appropriate component. The handoff flow uses `lens_submit_handoff` (not a lens-specific command) — any tray lens can submit handoffs through the same generic API.

### Tray Tauri commands (generic, not lens-specific)

| Command | Description |
|---------|-------------|
| `show_lens_surface { lensId }` | Show/create the tray window for a lens |
| `hide_lens_surface { lensId }` | Hide the tray window |
| `lens_submit_handoff { request }` | Submit a handoff from any lens surface |

The `request` payload includes `lensId` so the backend knows which lens originated it.

---

## 5. Shortcuts

Declare global keyboard shortcuts in `lens.toml`:

```toml
[[shortcuts]]
id = "open"            # Unique ID within this lens
action = "launch"      # What happens when pressed
combo = "CommandOrControl+Shift+M"  # Accelerator string
global = true          # true = OS-level, false = window-scoped
description = "Open My Lens"
```

**How it works at runtime:**
1. On Desktop startup, the TS layer reads all installed lens manifests
2. For each shortcut with `global = true`, it calls `register_lens_shortcut(combo, lensId)`
3. When pressed, Rust emits `lens:shortcut_triggered` with the lens ID
4. The TS layer checks `manifest.lens.surface` — if `"tray"`, opens the tray surface; otherwise opens a pane

Shortcuts are automatically unregistered when a lens is uninstalled.

---

## 6. Security & Permissions

### Sandbox levels

| Level | Access | Use when |
|-------|--------|----------|
| `restricted` | No fs, no network | Pure computation, safe by default |
| `network` | HTTP/WebSocket only | APIs, external services |
| `full` | Full system access | Requires explicit user approval |

### Declaring permissions

```toml
[security]
sandbox = "network"
permissions = [
  "network:api.github.com",
  "network:api.linear.app",
  "fs:read:~/.config/my-tool",
]
```

Permissions are shown to the user during installation preview. Undeclared access is blocked at the sandbox boundary.

### Hash verification

For compiled lenses (.dylib/.so), declare the expected hash:

```toml
[security]
library_hash = "sha256:a1b2c3d4e5f6..."
```

Desktop verifies this before loading.

---

## 7. Install & Test Locally

### From local path (development)

```bash
# In Graphyn Desktop, open Settings or use the Optics store
# Choose "Install from path" and point to your lens directory
```

Or via the Tauri API:
```typescript
await invoke('lens_install', {
  source: { type: 'local_path', path: '/path/to/my-lens' }
});
```

### Linked mode (fast dev loop)

Linked mode symlinks instead of copying — changes to your source are immediately visible:

```typescript
await invoke('lens_install', {
  source: {
    type: 'local_path',
    path: '/path/to/my-lens',
    install_mode: 'linked'
  }
});
```

### Verify installation

1. Open Graphyn Desktop
2. Go to Optics store pane
3. Your lens should appear in "Installed"
4. If it declares shortcuts, check they're registered (look for log: `[hotkey] Registered lens shortcut: ...`)
5. If it declares `surface = "tray"`, press the shortcut — the tray window should open

---

## 8. Publish to the Optics Store

### Via TBH (tbh.md)

1. Push your lens to a public GitHub repository
2. Create a listing on tbh.md with your lens metadata
3. Users can install directly from the Optics store in Desktop (the TBH install bridge handles the download)

### What happens on install from store

1. Desktop resolves the `store_id` against the TBH catalog
2. Downloads the GitHub archive (tar.gz) from the `repository` field
3. Extracts and finds the `lens.toml` (searches up to 3 levels deep)
4. Validates manifest, checks permissions, verifies hash
5. Copies to `~/.graphyn/lenses/{lens-id}/`
6. Registers in the local lens registry

---

## 9. Project Structure

A typical lens directory:

```
my-lens/
├── lens.toml              # Manifest (required)
├── Cargo.toml             # Rust dependencies
├── src/
│   └── lib.rs             # Lens trait implementation
├── components/            # Frontend components (optional)
│   └── MyResult.tsx       # Custom message type renderer
├── target/
│   └── release/
│       └── libmy_lens.dylib  # Compiled library
└── README.md
```

---

## 10. Reference: Quick Lens

The Quick lens (`store/quick/`) is the reference implementation for a tray-surface lens. Study it for:

- **Manifest**: `lens.toml` — tray surface, global shortcut, capabilities
- **Frontend**: The `@graphyn/quick` package exports the composer surface component
- **Handoff**: Uses `lens_submit_handoff` to create threads from tray input
- **Shortcut**: `CommandOrControl+Alt+Space` opens the tray globally

Key principle: **Quick has zero special-case code in Desktop.** It uses the same generic lens infrastructure (surfaces, shortcuts, handoffs) as any other lens. If Quick can do it, your lens can too.

---

## Checklist

Before shipping:

- [ ] `lens.toml` has `id`, `name`, `version`, `description`
- [ ] `surface` is set correctly for your use case
- [ ] Shortcuts (if any) have unique `id` values and valid `combo` strings
- [ ] Security sandbox level matches your actual access needs
- [ ] `cargo build --release` succeeds
- [ ] Installed locally in Desktop and tested the primary flow
- [ ] Manifest parses correctly: `cargo test` in the lens crate
- [ ] Custom message types (if any) have matching frontend components

---

## Further Reading

- `docs/SURFACE_SPEC_V1.md` — Tab, Pack, App surface architecture
- `docs/LENS_PACKAGE_SPEC_V2.md` — Full v2 manifest specification
- `README.md` — Trait patterns and event system
- `src/manifest.rs` — Rust manifest struct definitions
- `store/quick/lens.toml` — Reference tray lens
- `store/figma/lens.toml` — Reference pane lens with MCP tools
