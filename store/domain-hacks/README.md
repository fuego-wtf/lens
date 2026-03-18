# Domain Hacks Lens

Domain growth hacking system for AI-powered development.

## Overview

This Lens implements a modular domain growth hacking system with:
- **Domain Availability Checking** - GoDaddy API + DNS/HTTP cross-verification
- **Strategy Generation** - Tier-1 (platform), Tier-2 (action), Tier-3 (category) strategies
- **Landing Page Templates** - SEO-optimized templates for each domain
- **MCP Server Integration** - Tool-based interface for agent consumption

## Verified Domains

| Domain | Strategy | Traffic Potential |
|---------|----------|------------------|
| `devboosthub.info` | Tier-1: Dev productivity hub | 12M/month |
| `devboosthub.net` | Tier-1: Dev productivity hub | 12M/month |
| `videoai-tools.com` | Tier-3: Video AI tools | 11M/month |
| `videoaitools.net` | Tier-3: Video AI tools | 11M/month |

**Total Cost:** ~$50/year (4 domains × $12-15)

## Capabilities

### 1. Domain Availability Check
- Query GoDaddy API for availability
- Cross-verify via DNS resolution (A, MX, NS records)
- HTTP accessibility check
- Price estimation and registration links

### 2. Strategy Generation

Three tiers of growth strategies:

#### Tier 1: Platform-Specific Ecosystems
- `yt.tools` - YouTube developer tools
- `x.tools` - X/Twitter automation
- `devboosthub.net` - Centralized dev tool hub

#### Tier 2: Action-Oriented Domain Hacks
- `buildit.dev` - "Build it" CTA
- `shipit.today` - Daily shipping motivation
- `codeit.io` - "Code it" action verb

#### Tier 3: Category Authority Domains
- `figma-tools.dev` - Figma plugin directory
- `terminal-tools.dev` - CLI tool hub
- `graphic-tools.dev` - AI design assets

### 3. Landing Page Templates

- SEO-optimized templates
- Open Graph metadata
- Structured data for search engines
- CTA-optimized layouts

### 4. MCP Server Integration

Exposes tools for agent consumption:
- `check_domain_availability` - Single domain check
- `check_domains_batch` - Batch domain checks
- `generate_strategy` - Generate growth strategies
- `suggest_domains` - Get domain suggestions

## Usage

### Basic Lens Execution

```rust
use lens::Lens;
use domain_hacks::DomainHacksPlugin;

let plugin = DomainHacksPlugin::new();

// Non-streaming: basic availability check
let result = plugin.execute(ctx).await?;

// Streaming: full pipeline with strategies and templates
let (result, event_stream) = plugin.execute_streaming(ctx).await?;

// Process events
while let Some(event) = event_stream.next().await {
    match event {
        LensEvent::Started => { /* Pipeline started */ }
        LensEvent::Progress { message } => { /* Update UI */ }
        LensEvent::Data { key, value } => { /* Process data */ }
        LensEvent::Completed => { /* Pipeline done */ }
    }
}
```

### MCP Server Usage

```rust
use domain_hacks::DomainMcpServer;
use lens::McpServerLens;

let server = DomainMcpServer::new();

// List available tools
let tools = server.tool_list();
// ["check_domain_availability", "check_domains_batch", "generate_strategy", "suggest_domains"]

// Call tool
let request = ToolCallRequest {
    tool_name: "check_domain_availability".to_string(),
    params: serde_json::json!({ "domain": "example.com" }),
};
let result = server.call_tool(request)?;
```

## Configuration

Set environment variables for GoDaddy API:

```bash
export GODADDY_API_KEY="your_api_key"
export GODADDY_API_SECRET="your_api_secret"
```

Or provide via plugin input:

```json
{
  "domains": ["example.com", "test.dev"],
  "godaddy_api_key": "key",
  "godaddy_api_secret": "secret",
  "generate_strategies": true,
  "generate_landing_pages": true
}
```

## Architecture

```
domain-hacks/
├── src/
│   ├── domains.rs      # GoDaddy API + verification
│   ├── strategies.rs   # Tier-1/2/3 strategies
│   ├── mcp_server.rs  # MCP tool interface
│   ├── plugin.rs      # Main Lens implementation
│   ├── types.rs       # Domain + strategy types
│   ├── utils.rs       # DNS/HTTP verification
│   ├── error.rs       # Error types
│   └── lib.rs         # Module exports
├── Cargo.toml
├── plugin.toml
└── README.md
```

## Build

```bash
cd /Users/resatugurulu/Developer/graphyn-workspace/lens/store/domain-hacks
cargo build
cargo test
```

## License

MIT - [Graphyn](https://graphyn.xyz)

---

`★ Insight ─────────────────────────────────────`
**This Lens gives you:**
1. **Reusable growth architecture** - One codebase, deploy to 4+ verified domains
2. **Cross-verified availability** - API + DNS + HTTP prevents false positives
3. **Three-tier strategy framework** - Platform, action, and category patterns
4. **MCP-ready** - Agents can invoke tools directly via standard protocol
5. **SEO-optimized templates** - Ready-to-deploy landing pages for search visibility
`─────────────────────────────────────────────────`
