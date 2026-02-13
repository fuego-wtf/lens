# Lens

A trait standard for crafting specialized agent perspectives.

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

## What is a Lens?

A Lens is a specialized perspective that agents work through. Each Lens synthesizes domain knowledge into a focused capability — design decomposition, emotion detection, code analysis, etc.

## Features

- **`Lens` trait** — implement `id()`, `name()`, `version()`, `execute()`
- **`StreamingLens`** — stream events in real-time via `LensEvent`
- **`LensManifest`** — declarative `lens.toml` for permissions, metadata, MCP config
- **`runtime` feature** — discovery from `~/.graphyn/lenses/` and dynamic loading

## Usage

```toml
# Lens authors (implement the trait)
[dependencies]
lens = { git = "https://github.com/fuego-wtf/lens.git" }

# Host applications (discover + load lenses)
[dependencies]
lens = { git = "https://github.com/fuego-wtf/lens.git", features = ["runtime"] }
```

## Store

The `store/` directory hosts lens implementations as workspace members:

```
lens/
├── src/              # Lens trait standard
├── store/            # Lens implementations
│   ├── figma/        # Design decomposition
│   ├── vibe/         # Emotion detection
│   └── .../
└── Cargo.toml
```

## License

MIT
