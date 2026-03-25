# Graphyn Surface Spec v1

> Three surfaces. One shell. Everything is a small application.

## Core Concept

Graphyn is a **shell**. Everything inside it is a small application that renders on one of three surfaces. The application itself is surface-agnostic — it declares what it supports, and the user (or the system) decides where it lives.

```
                          GRAPHYN SHELL
 ┌──────────────────────────────────────────────────────┐
 │                                                      │
 │   ┌─────┐  ┌──────────────────┐  ┌─────┐           │
 │   │PACK │  │      TAB         │  │PACK │           │
 │   │     │  │                  │  │     │           │
 │   │     │  │  Full canvas     │  │     │           │
 │   │     │  │  application     │  │     │           │
 │   └─────┘  └──────────────────┘  └─────┘           │
 │   sidebar   editor area          sidebar            │
 └──────────────────────────────────────────────────────┘

                    ┌──────────────────┐
                    │      APP         │
                    │                  │
                    │  Standalone      │
                    │  window          │
                    └──────────────────┘
                    detached / external
```

---

## The Three Surfaces

### 1. Tab

A **Tab** is a full-canvas application that lives in the editor area.

| Property | Value |
|----------|-------|
| **Canvas** | Full editor area (flex, unbounded) |
| **Lifecycle** | Created on demand, persisted in session |
| **Navigation** | Tab bar at top, switchable, closeable |
| **Examples** | Thread view, file editor, settings, agent designer, diff viewer |
| **Multiplicity** | Many tabs open simultaneously |
| **Identity** | `tab:{type}:{id}` (e.g., `tab:thread:abc`, `tab:file:/src/main.rs`) |

A Tab is a **destination** — you navigate to it.

```
┌─ thread-view ──┬─ main.rs ──┬─ settings ──┐
│                                             │
│  Full application canvas                    │
│                                             │
│  The tab owns the entire editor area.       │
│  It can render anything: conversation,      │
│  code editor, form, visualization.          │
│                                             │
│  Graphyn provides:                          │
│  - Tab chrome (title, close, drag)          │
│  - Backend access (API, auth, state)        │
│  - Pane opening (nested navigation)         │
│                                             │
└─────────────────────────────────────────────┘
```

### 2. Pack

A **Pack** is a composable sidebar unit. It lives in the left or right dock.

| Property | Value |
|----------|-------|
| **Canvas** | Sidebar width (200-400px), full height or stacked |
| **Lifecycle** | Always available, toggled via sidebar view switch |
| **Composition** | Stackable — multiple packs in one sidebar (Phase 5+) |
| **Draggable** | Between left dock and right dock (Phase 5+) |
| **Examples** | Explorer, thread list (default packs); user-built packs via lens system |
| **Multiplicity** | One instance per pack type, but multiple types stacked |
| **Identity** | `pack:{type}` (e.g., `pack:explorer`, `pack:threads`) |

A Pack is a **tool you keep at hand** — it assists your work.

```
┌───────────────┐
│ EXPLORER    ⊟ │ ← Pack header (label + action icons)
│───────────────│
│               │ ← Pack content (scrollable)
│ › backyard/ S │
│ › desktop/  S │
│ › design/   S │
│               │
├─ ─ ─ ─ ─ ─ ─ ┤ ← Stack separator (draggable resize)
│ SOURCE CTRL ↻ │ ← Another pack, stacked below
│───────────────│
│ ▼ Changes   4 │
│  config.y   M │
│  prompt.m   M │
└───────────────┘
```

**Pack composition rules:**

1. A sidebar dock holds 1+ packs in a vertical stack
2. Each pack has a collapsible header with action icons
3. Packs can be dragged between left and right docks
4. Packs can be reordered within a dock by dragging headers
5. The drag handle between stacked packs resizes them
6. The activity bar reflects which packs are in each dock

### 3. App

An **App** is a standalone window — a detached application.

| Property | Value |
|----------|-------|
| **Canvas** | Own OS window, fully independent size |
| **Lifecycle** | Launched on demand, independent of main window |
| **Styling** | Minimal shell chrome — the app owns the UI |
| **Backend** | Shared with main Graphyn instance (same auth, same API) |
| **Examples** | External tool with Graphyn backend, detached thread, terminal |
| **Multiplicity** | Multiple app windows simultaneously |
| **Identity** | `app:{type}:{id}` (e.g., `app:terminal:1`, `app:external:my-tool`) |

An App is a **breakout** — when a Tab needs its own space, or an external tool needs Graphyn's backend.

```
┌─ Graphyn ─────────────────┐     ┌─ App Window ────────────┐
│                            │     │                         │
│  Main shell with tabs,     │     │  Standalone surface.    │
│  packs, and activity bars  │────▶│  Graphyn provides:      │
│                            │     │  - Backend (API, auth)  │
│                            │     │  - State sync           │
│                            │     │  The app owns the UI.   │
│                            │     │                         │
└────────────────────────────┘     └─────────────────────────┘
                               Tauri WebviewWindow / child window
```

---

## Surface Capabilities Matrix

| Capability | Tab | Pack | App |
|-----------|-----|------|-----|
| Full canvas | Yes | No (sidebar-width) | Yes |
| Always visible | No (one active tab) | Yes (docked) | Yes (own window) |
| Composable/stackable | No | Yes | No |
| Draggable between docks | No | Yes (left/right) | N/A |
| Backend access | Yes | Yes | Yes |
| Can open other tabs | Yes | Yes | No (isolated) |
| Keyboard shortcut toggle | No | Yes (activity bar) | No |
| Resizable | N/A (flex) | Drag handle | OS window chrome |
| User-created | Yes (via lens) | Yes (via lens) | Yes (via lens) |

---

## The Activity Bar

The activity bar is **not** a surface. It is the sidebar's chrome — a vertical icon strip that controls which packs are visible.

```
┌──┐
│📋│  pack:threads        ← toggles thread list pack
│📁│  pack:explorer       ← toggles file explorer pack
│🔍│  pack:search         ← toggles search pack
│⎇ │  pack:git            ← toggles source control pack
│◎ │  pack:lenses         ← toggles lenses pack
│──│
│⚙ │  tab:settings        ← opens settings as a tab (special case)
└──┘
```

Each dock (left, right) has its own activity bar. When a pack is dragged to the other dock, its icon moves with it.

---

## Layout Model

```typescript
interface GraphynLayout {
  left: Dock;
  right: Dock;
  editor: EditorArea;
}

interface Dock {
  width: number;           // persisted, resizable
  collapsed: boolean;      // toggle via activity bar
  packs: PackInstance[];   // ordered stack
  activePack: string;      // which pack has focus (for keyboard nav)
}

interface PackInstance {
  type: string;            // "explorer" | "threads" | "git" | "search" | "lenses"
  collapsed: boolean;      // header-level collapse within stack
  height: number | "flex"; // explicit height or fill remaining
}

interface EditorArea {
  tabs: TabInstance[];     // ordered
  activeTab: string;       // which tab is focused
  splits: Split[];         // optional: side-by-side editor splits
}

interface TabInstance {
  type: string;            // "thread" | "file" | "settings" | "agent"
  id: string;              // unique identifier
  label: string;           // tab title
  dirty: boolean;          // unsaved changes indicator
}
```

**Persisted layout** — the full layout serializes to localStorage (prototype) or user preferences (production). User's composition survives restart.

---

## Pack Header Spec

Every pack has a standard header:

```
┌─────────────────────────────────────────┐
│ ▼ LABEL                   [⊕] [↻] [⊟]  │
│   ↑                        ↑   ↑   ↑   │
│   collapse toggle          │   │   │    │
│                            │   │   └─ collapse all (context-specific)
│                            │   └───── refresh
│                            └───────── primary action (new file, new thread, etc.)
└─────────────────────────────────────────┘
```

| Element | Behavior |
|---------|----------|
| Label | Pack name, uppercase, 12px, bold, muted |
| Collapse toggle | Click label or chevron to collapse pack body |
| Action icons | Right-aligned, 0.4 opacity, brighten on hover |
| Primary action | Leftmost action icon, context-specific |
| Drag handle | Entire header is draggable for recomposition |

---

## How a Lens Declares Surface Support

A lens (plugin) declares which surfaces it can render on:

```toml
# lens.toml
[lens]
id = "my-custom-tool"
name = "My Custom Tool"
version = "0.1.0"

[surfaces]
tab = true       # Can render as a full tab
pack = true      # Can render as a sidebar pack
app = true       # Can render as a standalone window

[pack]
icon = "wrench"
label = "My Tool"
default_dock = "left"     # "left" | "right"
default_position = 3      # order in stack

[tab]
label = "My Tool"
closeable = true

[app]
title = "My Tool — Graphyn"
width = 800
height = 600
resizable = true
```

A lens that supports multiple surfaces can be opened in any of them. The user decides. A file explorer might default to pack but could be opened as a tab for a full-screen tree view.

---

## Prior Art

| Editor | Equivalent of Pack | Equivalent of Tab | Equivalent of App |
|--------|-------------------|-------------------|-------------------|
| VS Code | View (in ViewContainer) | Editor Tab | Auxiliary Window |
| Zed | Panel (in Dock) | Item (in Pane) | — |
| Cursor | Same as VS Code | Same as VS Code | — |
| OpenCode | Collapsible sidebar section | Session tab | — |

### Key references:
- VS Code `ViewContainer` + `View` registration via `contributes.viewContainers`
- Zed `Panel` trait with `position() -> DockPosition` and `icon() -> IconName`
- VS Code `FileDecoration` for git status (badge + color + propagation)
- Zed `GitSummary` sum-tree aggregation for folder status

---

## Version History

| Version | Date | Change |
|---------|------|--------|
| v1 | 2026-03-20 | Initial spec — Pack, Tab, App surfaces defined |
