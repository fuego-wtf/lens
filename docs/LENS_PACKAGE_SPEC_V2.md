# Lens Package Specification v2

**Version:** 2.0.0
**Date:** 2026-03-06
**Status:** Draft
**PR:** PR-58

---

## Overview

This specification defines the Lens Package format v2, extending the original lens.toml manifest with registry metadata, lifecycle hooks, and enhanced security declarations.

---

## Changes from v1

| Feature | v1 | v2 |
|---------|----|----|
| Manifest version | Implicit | Explicit `manifest_version = 2` |
| Registry metadata | None | `registry` section with install stats |
| Lifecycle hooks | None | `hooks` section with pre/post scripts |
| MCP tools | Implicit from message_types | Explicit `mcp_tools` declaration |
| Author identity | Simple string array | Structured `authors` with roles |
| License | Simple string | SPDX identifier + file reference |
| Entry points | Single entry | Multiple entry points by mode |
| Dependencies | Lens IDs only | Version constraints + optional flag |

---

## Required Fields

All v2 manifests MUST include these fields:

### `[lens]` Section

```toml
[lens]
id = "com.example.my-lens"           # REQUIRED: Unique identifier (reverse domain notation)
name = "My Lens"                      # REQUIRED: Human-readable name
version = "1.0.0"                     # REQUIRED: SemVer version
description = "A sample lens"         # REQUIRED: Brief description
manifest_version = 2                  # REQUIRED: Must be 2 for v2 manifests
min_framework_version = "0.5.0"       # REQUIRED: Minimum Graphyn framework version
```

### `[authors]` Section

```toml
[[authors]]
name = "Jane Developer"
email = "jane@example.com"
role = "maintainer"                   # "maintainer" | "contributor" | "sponsor"
```

### `[license]` Section

```toml
[license]
spdx = "MIT"                          # SPDX license identifier
file = "LICENSE"                      # Path to license file (optional)
```

---

## Optional Fields

### `[registry]` Section

Metadata for lens registry:

```toml
[registry]
category = "productivity"             # "productivity" | "development" | "integration" | "experimental"
tags = ["automation", "mcp"]
homepage = "https://example.com/my-lens"
repository = "https://github.com/example/my-lens"
issues = "https://github.com/example/my-lens/issues"
icon = "assets/icon.png"              # Path to icon file (64x64 recommended)
screenshots = ["assets/screenshot1.png"]
```

### `[hooks]` Section

Lifecycle hooks for installation/activation:

```toml
[hooks]
pre_install = "scripts/pre-install.sh"
post_install = "scripts/post-install.sh"
pre_enable = "scripts/pre-enable.sh"
post_enable = "scripts/post-enable.sh"
pre_disable = "scripts/pre-disable.sh"
post_disable = "scripts/post-disable.sh"
pre_uninstall = "scripts/pre-uninstall.sh"
post_uninstall = "scripts/post-uninstall.sh"
```

### `[mcp_tools]` Section

Explicit MCP tool declarations:

```toml
[[mcp_tools]]
name = "search"
description = "Search the knowledge base"
input_schema = "schemas/search.json"

[[mcp_tools]]
name = "index"
description = "Index a document"
input_schema = "schemas/index.json"
```

### `[entry_points]` Section

Multiple entry points by mode:

```toml
[[entry_points]]
mode = "ask"                          # "ask" | "plan-first" | "code" | "designer"
file = "dist/ask.js"
system_prompt = "prompts/ask.md"

[[entry_points]]
mode = "code"
file = "dist/code.js"
system_prompt = "prompts/code.md"
```

### `[dependencies]` Section

Lens dependencies with version constraints:

```toml
[[dependencies]]
id = "com.graphyn.base"
version = ">=1.0.0,<2.0.0"
optional = false

[[dependencies]]
id = "com.example.optional-helper"
version = "^1.0.0"
optional = true
```

### `[security]` Section (unchanged from v1)

```toml
[security]
library_hash = "sha256:abc123..."
permissions = ["fs:read:~/Documents", "network:api.example.com"]
sandbox = "network"                   # "restricted" | "network" | "full"
```

### `[capabilities]` Section (unchanged from v1)

```toml
capabilities = ["read", "write", "execute"]
```

### `[[message_types]]` Section (unchanged from v1)

```toml
[[message_types]]
key = "search_results"
component = "components/SearchResults.tsx"
description = "Displays search results"
interactive = false
schema = "schemas/search-results.json"
```

---

## Complete v2 Example

```toml
# Lens Package v2 Manifest
manifest_version = 2

[lens]
id = "com.graphyn.knowledge-base"
name = "Knowledge Base"
version = "2.0.0"
description = "Local knowledge base with semantic search and MCP tools"
min_framework_version = "0.5.0"

[[authors]]
name = "Graphyn Team"
email = "team@graphyn.xyz"
role = "maintainer"

[license]
spdx = "MIT"
file = "LICENSE"

[registry]
category = "productivity"
tags = ["knowledge", "search", "mcp"]
homepage = "https://graphyn.xyz/lens/knowledge-base"
repository = "https://github.com/graphyn/knowledge-base"
icon = "assets/icon.png"

[security]
permissions = ["fs:read:~/Documents", "fs:write:~/.graphyn/kb"]
sandbox = "restricted"

capabilities = ["read", "write", "search"]

[[mcp_tools]]
name = "kb_search"
description = "Search the knowledge base"
input_schema = "schemas/search.json"

[[mcp_tools]]
name = "kb_index"
description = "Index documents into the knowledge base"
input_schema = "schemas/index.json"

[[entry_points]]
mode = "ask"
file = "dist/ask.js"
system_prompt = "prompts/ask.md"

[[entry_points]]
mode = "code"
file = "dist/code.js"
system_prompt = "prompts/code.md"

[[dependencies]]
id = "com.graphyn.base"
version = ">=1.0.0"
optional = false

[[message_types]]
key = "search_results"
component = "components/SearchResults.tsx"
description = "Displays semantic search results"
interactive = false

[[message_types]]
key = "index_progress"
component = "components/IndexProgress.tsx"
description = "Shows indexing progress"
interactive = false

[hooks]
post_install = "scripts/setup-db.sh"
pre_uninstall = "scripts/cleanup.sh"
```

---

## Field Reference

### `[lens]` Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✅ | Unique identifier (reverse domain notation) |
| `name` | string | ✅ | Human-readable name |
| `version` | string | ✅ | SemVer version string |
| `description` | string | ✅ | Brief description (max 200 chars) |
| `manifest_version` | integer | ✅ | Must be `2` for v2 manifests |
| `min_framework_version` | string | ✅ | Minimum Graphyn framework version |
| `max_framework_version` | string | ❌ | Maximum compatible framework version |

### `[[authors]]` Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | ✅ | Author name |
| `email` | string | ❌ | Author email |
| `role` | string | ❌ | One of: `maintainer`, `contributor`, `sponsor` |

### `[license]` Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `spdx` | string | ✅ | SPDX license identifier |
| `file` | string | ❌ | Path to license file |

### `[registry]` Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `category` | string | ❌ | Category for registry browsing |
| `tags` | string[] | ❌ | Tags for search/filtering |
| `homepage` | string | ❌ | Lens homepage URL |
| `repository` | string | ❌ | Source repository URL |
| `issues` | string | ❌ | Issue tracker URL |
| `icon` | string | ❌ | Path to icon file |
| `screenshots` | string[] | ❌ | Paths to screenshot files |

### `[[mcp_tools]]` Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | ✅ | Tool name |
| `description` | string | ✅ | Tool description |
| `input_schema` | string | ❌ | Path to JSON schema |

### `[[entry_points]]` Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `mode` | string | ✅ | Mode: `ask`, `plan-first`, `code`, `designer` |
| `file` | string | ✅ | Entry point file path |
| `system_prompt` | string | ❌ | Path to system prompt template |

### `[[dependencies]]` Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✅ | Dependency lens ID |
| `version` | string | ❌ | SemVer version constraint |
| `optional` | boolean | ❌ | Whether dependency is optional (default: false) |

---

## Validation Rules

1. **ID Format**: Must match regex `^[a-z0-9-]+(\.[a-z0-9-]+)*$`
2. **Version Format**: Must be valid SemVer (e.g., `1.0.0`, `2.1.0-beta`)
3. **Framework Version**: Must be valid SemVer
4. **Category**: Must be one of: `productivity`, `development`, `integration`, `experimental`
5. **Role**: Must be one of: `maintainer`, `contributor`, `sponsor`
6. **Mode**: Must be one of: `ask`, `plan-first`, `code`, `designer`
7. **Sandbox**: Must be one of: `restricted`, `network`, `full`

---

## Migration from v1

To migrate a v1 manifest to v2:

1. Add `manifest_version = 2` to `[lens]` section
2. Add `min_framework_version` to `[lens]` section
3. Convert `authors` string array to `[[authors]]` table array
4. Convert `license` string to `[license]` table with `spdx` field
5. Add `[registry]` section with category (required for registry publishing)
6. Add `[[mcp_tools]]` declarations if the lens provides MCP tools
7. Add `[[entry_points]]` if the lens has mode-specific entry points

---

## Backwards Compatibility

- v1 manifests continue to work with v2 parsers
- v2-specific features are ignored by v1 parsers
- `manifest_version` defaults to `1` if not specified
