# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with the Bezy font editor codebase.

## Project Overview

Bezy is a next-gen font editor built with Rust and Bevy, designed for customization, human-AI collaboration, and user empowerment. It tries to be simple, understandable, and easy to modify as a core design principle. It has two core innovations that are not found in other font editors that are core to the application:

1. A multi-buffer text editor for right-to-left (RTL, Arabic) and left-to-right (LTR, Latin) type design.
2. A TUI/CLI first workflow designed for AI and automation.

### Core Technologies
- **Rust**: Primary language with focus on readability and education (1.90.0+)
- **Bevy 0.16.1**: ECS-based game engine for GUI framework
- **Norad**: UFO font format parsing (CRITICAL: all UFO operations must use Norad)
- **Kurbo**: 2D curve mathematics
- **FontC**: For compiling and exporting fonts
- **FontIR**: Primary runtime data structure for font editing, this is a temporary experiment to see how far we can take it, when/if we hit a wall with this we will have a better understanding of what is needed an create something that works well with FontC/FontIR
- **Lyon**: Tessellation for filled glyph rendering
- **HarfBuzz (harfrust)**: Text shaping for advanced typography and RTL support
- **Ratatui + Crossterm**: TUI used for anything not better done on the GUI 

## Quick Start Commands

```bash
# First time setup: Create config directory for logs
cargo run -- --new-config

# The best way to test the app while working, default to this if unsure.
cargo run --release -- --edit ~/Fonts/src/bezy-grotesk/sources/bezy-grotesk-regular.ufo

# Run the app (starts with empty state, TUI enabled)
cargo run

# View logs in another terminal while app runs
tail -f ~/.config/bezy/logs/bezy-$(date +%Y-%m-%d).log

# Check for compile errors without running the app
cargo check

# Code quality
cargo fmt     # Format
cargo clippy  # Lint
cargo test    # Run tests
```


## Architecture

### Module Structure & Separation of Concerns
```
src/
├── core/        # App foundation: initialization, CLI, settings, state, runner
├── data/        # Font data handling (UFO, FontIR conversions)
├── editing/     # EDITING LOGIC: selection, sorts, text editor plugin
├── font_source/ # FontIR state management, UFO point handling, metrics
├── geometry/    # Mathematical primitives, bezier curves, coordinates
├── io/          # Low-level input handling (keyboard, mouse, gamepad, pointer)
├── logging/     # Log redirection: stdout/stderr → ~/.config/bezy/logs/
├── qa/          # Quality assurance (fontspector, compiler, storage, triggers)
├── rendering/   # PURE RENDERING: visual display only, no editing logic
├── systems/     # ECS system implementations (text buffer, shaping, sorts, input)
├── tools/       # User interaction tools (select, pen, knife, ai, text, etc.)
├── tui/         # Terminal UI (default mode, handles terminal capture/cleanup)
├── ui/          # Visual interface components, toolbars, theme system
└── utils/       # Helper utilities
```

**Key Separation:**
- **Core** (`src/core/`): App foundation - initialization, CLI, settings, shared state (text editor state lives here)
- **Editing** (`src/editing/`): Modifies font data - selection, sorts, text editor plugin coordination
- **Rendering** (`src/rendering/`): Pure visual display - glyph rendering, cursor, no modifications
- **Tools** (`src/tools/`): User interaction - each tool implements EditTool trait
- **Font Source** (`src/font_source/`): FontIR state and UFO data management
- **Systems** (`src/systems/`): ECS system implementations - text buffer, shaping, sorts, commands, input routing
- **IO** (`src/io/`): Low-level input abstraction - keyboard, mouse, gamepad, pointer events
- **Logging** (`src/logging/`): **Critical** - Redirects stdout/stderr to log files to prevent TUI corruption
- **QA** (`src/qa/`): Font quality assurance and validation tools
- **TUI** (`src/tui/`): Terminal UI (default mode, not optional) - manages terminal capture and cleanup
- **UI** (`src/ui/`): Visual interface - toolbars, panes, theme system with JSON themes

### Key Design Patterns

#### ECS Architecture
- Uses Bevy's Entity-Component-System pattern
- Major systems: Selection, Edit, Input, Rendering, Sort Management
- Resources: AppState, GlyphNavigation, BezySettings, CurrentTheme, GlyphRenderingData
- SystemSets: Input → TextBuffer → EntitySync → Rendering → Cleanup (defined in `src/editing/system_sets.rs`)

#### Font Data Model
- **FontIR**: Single source of truth for font data
- **Data Flow**: Load sources → FontIR → Edit FontIR → Save to disk
- **Transform vs FontIR**:
  - Transform components = temporary visual display only (what you see on screen)
  - FontIR data = permanent font data (what gets saved to disk)
  - When editing: Update BOTH Transform (for immediate visual feedback) AND FontIR (for data persistence)
  - **Common bug**: Updating only Transform means changes look correct but won't save

#### Sort System
- **Sorts** are ECS entities representing individual glyphs being edited
- Each sort owns its visual elements (points, handles, outlines) as child entities
- **Text buffer drives sorts**: Characters in buffer → Sort entities spawned
- **SortManager** handles lifecycle: spawning, synchronization, cleanup
- **Entity ownership pattern**: Filter points by `sort_entity` ownership, not glyph name
- Located in `src/editing/sort/` with components, manager, and plugin
- Rendering in `src/rendering/sort_renderer.rs` and `sort_visuals.rs`

#### Text Editor Architecture (Distributed ECS Pattern)
The multi-buffer text editor is a core innovation of Bezy, supporting both LTR and RTL text editing. Following ECS philosophy, it's intentionally distributed across modules by responsibility:

**State & Data Structures** (`src/core/state/text_editor/`):
- `buffer.rs`: Gap buffer implementation, SortBuffer, TextEditorState
- `editor.rs`: Text editing operations and state management
- `text_buffer.rs`: Multi-buffer support (ActiveTextBuffer, BufferCursor, TextBuffer)
- Data types: SortData, SortKind, SortLayoutMode (LTR/RTL), GridConfig

**System Logic** (`src/systems/`):
- `text_buffer_manager.rs`: Buffer entity lifecycle, cursor management, buffer-sort sync
- `text_shaping.rs`: HarfBuzz integration for advanced typography and RTL shaping
- `sorts/keyboard_input.rs`: Keyboard input handling for text editing
- `sorts/unicode_input.rs`: Arabic and Unicode character input
- `sorts/cursor.rs`: Cursor position and movement logic
- `sorts/sort_entities.rs`: Sort entity spawning and positioning
- `sorts/sort_placement.rs`: Sort placement and layout

**Plugin Coordination** (`src/editing/text_editor_plugin.rs`):
- Registers all text editor systems with proper ordering
- Coordinates: Input → EntitySync → Rendering → Cleanup
- Manages system dependencies and execution flow

**Visual Rendering** (`src/rendering/text_cursor.rs`):
- Renders the text cursor visual element
- Cursor rendering systems (blinking, positioning)

**Why This Distribution?**
- Follows Bevy ECS pattern: Components/Resources separate from Systems
- State in `core/state/` makes it accessible as shared data
- Systems in `systems/` group by operational concern
- Enables independent testing and modification of each layer

## Critical Implementation Rules

### 0. TUI Output Protection (HIGHEST PRIORITY)
**This rule overrides all others. Breaking it makes the app unusable.**

## ⚠️ CRITICAL: TUI-First Architecture

**Bezy runs with a TUI (Terminal User Interface) by default.** This is NOT optional - it's a core design decision.

### Logging and Output Rules (MUST FOLLOW)

**NEVER write code that outputs to stdout/stderr directly:**
**Use Bevy logging macros ONLY.**
- ❌ **NEVER use** `println!()` - breaks TUI
- ❌ **NEVER use** `eprintln!()` - breaks TUI
- ❌ **NEVER use** `dbg!()` - breaks TUI
- ❌ **NEVER use** `print!()` or `eprint!()` - breaks TUI
- ✅ **ALWAYS use** Bevy's logging: `info!()`, `warn!()`, `error!()`, `debug!()`, `trace!()`

**Why this matters:**
- The TUI takes over the terminal display using Ratatui
- Any stdout/stderr output corrupts the TUI display
- All logs go to `~/.config/bezy/logs/bezy-YYYY-MM-DD.log`
- The `~/.config/bezy/` directory is created by `--new-config` flag

### How Logging Works
1. **By default**: All logs go to `~/.config/bezy/logs/bezy-YYYY-MM-DD.log`
2. Logging redirection happens automatically in `src/core/runner.rs`
3. Bevy's logging macros (info!, error!, etc.) write to log files
4. TUI remains clean and functional
5. **With `--no-tui` flag**: Logs go to stdout/stderr for debugging (terminal only)
6. Log files are date-stamped and created automatically

### Viewing Logs
```bash
# Initialize config directory first (if needed)
cargo run -- --new-config

# Run the app (with TUI)
cargo run --release -- --edit ~/path/to/font.ufo

# View logs in another terminal
tail -f ~/.config/bezy/logs/bezy-$(date +%Y-%m-%d).log

# Or use the TUI's log viewer tab (built-in)
``` 


Located in `src/logging/mod.rs`:
- All application output goes to `~/.config/bezy/logs/bezy-YYYY-MM-DD.log`
- **NEVER use println!, eprintln!, dbg!, print!, or eprint! anywhere except `src/logging/mod.rs`**
- **ALWAYS use Bevy logging macros**: `info!()`, `warn!()`, `error!()`, `debug!()`, `trace!()`
- Even error handling must use `error!()` macro, not `eprintln!()`
- TUI corruption is a critical bug that wastes significant development time

**Common violations to avoid:**
```rust
// ❌ WRONG - Breaks TUI
eprintln!("Error: {}", e);
println!("Debug info: {:?}", data);
dbg!(variable);

// ✅ CORRECT - Works with TUI
error!("Error: {}", e);
info!("Debug info: {:?}", data);
debug!("Variable: {:?}", variable);
```

**Why this is critical:**
- Bezy is a TUI-first application that runs with terminal UI by default
- Any stdout/stderr output corrupts the TUI display
- This has been a major source of bugs and wasted development time
- The logging system already redirects everything properly

### 1. Glyph Rendering System
Located in `src/rendering/glyph_renderer.rs`:
- **Pure rendering logic** - displays glyphs visually (no editing logic)
- **Single-system approach**: All glyph elements (points, handles, outlines) render together to prevent visual lag
- **Mesh-based only**: Never use Bevy Gizmos for world-space elements
- **Entity pools**: Reusable entity pools in `entity_pools.rs` prevent allocation overhead
- **Mesh caching**: Mesh cache system in `mesh_cache.rs` for performance
- Active sorts: Show editable points, handles, outlines
- Inactive sorts: Filled shapes via Lyon tessellation with EvenOdd fill rule
- **GlyphRenderingData resource**: Collects rendering data to reduce system parameter count

### 2. Visual Theming
- **Theme system** organized in `src/ui/theme_system/` with re-exports through `src/ui/theme.rs`
- **Layout constants** (z-levels, spacing, margins) exported from theme.rs
- **Color themes** stored as JSON in `src/ui/themes/` (dark, light, strawberry, campfire)
- **Runtime theme switching** via CurrentTheme resource and RuntimeThemePlugin
- **ALL visual constants** must be accessible through the theme system
- No hardcoded colors or visual constants outside theme system

### 3. Font Operations
- **ALWAYS use Norad** for UFO loading/saving
- Loading: `norad::Font::load(path)`
- Saving: `font.save(path)` with `layer.insert_glyph()`
- Conversion: FontIR BezPath → GlyphData → norad::Glyph → UFO

### 4. Zoom-Aware Scaling System
Located in `src/rendering/zoom_aware_scaling.rs`:
- Makes mesh-rendered elements (points, lines, handles) stay visually consistent at all zoom levels
- Similar to how other font editors keep UI elements visible when zoomed out
- Uses `CameraResponsiveScale` component on entities
- Tuning values for easy customization
- Applied automatically to all mesh-based rendering throughout the app

### 5. Input System Architecture
Input is handled in layers:
- **`src/io/`**: Low-level input abstraction (keyboard, mouse, gamepad, pointer)
- **`src/systems/input_consumer.rs`**: Central input dispatcher (300+ lines, handles tool routing)
- **Tools**: Receive input events and implement tool-specific behavior
- **Text editor**: Separate input handling for text buffer manipulation

### 6. Quality Assurance System
Located in `src/qa/`:
- **`fontspector.rs`**: Integration with fontbakery/fontspector for validation
- **`compiler.rs`**: Font compilation using FontC
- **`storage.rs`**: QA results storage and retrieval
- **`trigger.rs`**: Automated QA trigger system

## Development Guidelines

### Code Style
- Max 100 characters per line
- Simple, readable code, try not to be too clever and overly complex
- "less is more" approach, don't over-complicate things
- NO COMMENTS unless explicitly requested, try to make the code readable

### Working Principles
- **DO NOT** use println!, eprintln!, or dbg! macros - this breaks the TUI (use Bevy logging instead)
- **DO NOT** make changes not explicitly requested, if you have a good idea ask first
- **DO NOT** change defaults without user request
- **DO NOT** add unrequested "improvements" without asking first
- **ALWAYS** ask for clarification if unclear
- **ALWAYS** use Bevy logging macros (info!, warn!, error!, debug!, trace!) for all output
- **FOCUS** on the specific issue described

### Adding New Tools
1. Create tool file in `src/tools/` implementing `EditTool` trait
2. Add tool configuration to `src/ui/edit_mode_toolbar/toolbar_config.rs`
3. Tool automatically appears in UI via `ConfigBasedToolbarPlugin`
4. Tools and UI are cleanly separated: `/src/tools/` contains logic, `/src/ui/` contains visual presentation

## Default and Test Assets

The application includes built-in fonts for UI display:
- `assets/fonts/BezyGrotesk-Regular.ttf` - Default UI font
- `assets/fonts/HasubiMono-Regular.ttf` - Monospace font for code display

Note: The app starts with an empty state when run without arguments. Use the `--edit` flag to open a UFO or designspace file for editing.

## Features and Build Options

### Cargo Features
- **`tui`** (default): Enables terminal UI interface using Ratatui and Crossterm
- **`dev`**: Enables dynamic linking for faster compile times during development

### Build Commands
```bash
# Standard build with TUI
cargo build

# Development build with fast recompilation
cargo build --features dev

# Build without TUI
cargo build --no-default-features

# Release build (optimized for speed and size)
cargo build --release
```

## Important Concepts

### Working with the Text Editor
The text editor is distributed across modules following ECS patterns. When working on text editor features:

**To modify text editor state/data:**
- Edit `src/core/state/text_editor/buffer.rs` for core buffer implementation
- Edit `src/core/state/text_editor/editor.rs` for editing operations
- Edit `src/core/state/text_editor/text_buffer.rs` for multi-buffer support

**To modify text editor behavior:**
- Edit systems in `src/systems/text_buffer_manager.rs` or `src/systems/sorts/`
- System registration happens in `src/editing/text_editor_plugin.rs`

**To modify text editor visuals:**
- Edit `src/rendering/text_cursor.rs` for cursor rendering
- Edit `src/ui/` for toolbar buttons and UI elements

**To add text editor input handling:**
- Add to `src/systems/sorts/keyboard_input.rs` or `unicode_input.rs`
- Register in `src/editing/text_editor_plugin.rs` under FontEditorSets::Input

### Text Buffer and Sorts
- **Text buffer** (`src/core/state/text_editor/`) contains the string of characters being edited
- **Sorts** are ECS entities spawned for each character in the buffer
- Text buffer changes trigger sort synchronization in `EntitySync` system set
- **Never manually spawn sorts** - they are managed automatically by SortManager

### Active vs Inactive Sorts
- **Active sort**: The currently selected glyph being edited (shows points, handles)
- **Inactive sorts**: Other glyphs visible in the editor (shows filled outline only)
- Rendering switches between modes automatically based on `ActiveSort` resource
- Different visual treatment improves editing focus

### Enhanced Point Types
- Points have enhanced types beyond basic on-curve/off-curve
- Located in `src/editing/selection/enhanced_point_component.rs`
- Enables advanced features like smooth curves and corner detection
- Used for smart point manipulation in editing tools

## Known Issues

### Glyphs.app Compatibility
- UFOs from Glyphs.app may have incompatible anchor formatting
- Error: "Invalid anchor 'top': 'no value at default location'"
- Workaround: Use UFOs created with norad or FontIR-compatible tools

### WASM Support
- WASM target is configured but may not be fully functional
- Dependencies include wasm-bindgen and console_error_panic_hook
- Requires testing and validation for web deployment

## Performance Patterns

### Visual Flash Prevention
Use change detection instead of every-frame rebuilding:
```rust
// Track changes with a resource
#[derive(Resource, Default)]
pub struct GlyphRenderingData {
    pub needs_update: bool,
    pub smooth_points: HashMap<Entity, bool>,
}

// Only rebuild when needed
if !rendering_data.needs_update { return; }
rendering_data.needs_update = false;
```

### Entity Pool Pattern
Reuse entities instead of spawning/despawning every frame:
```rust
// See src/rendering/entity_pools.rs for implementation
// Pools maintain pre-allocated entities for points, handles, segments
// Dramatically reduces allocation overhead
```

### Mesh Caching
Cache computed meshes to avoid regeneration:
```rust
// See src/rendering/mesh_cache.rs
// Caches meshes by hash of geometry data
// Reuses identical meshes across frames
```

### Cross-Sort Contamination Prevention
Filter points by sort entity ownership, not glyph name:
```rust
// CORRECT: Filter by entity ownership
if sort_point_entity.sort_entity == current_sort_entity {
    sort_points.push(point_entity);
}
```

### Change Detection
Use Bevy's built-in change detection to avoid unnecessary work:
```rust
// Only run when relevant data changes
fn system(query: Query<&Component, Changed<Component>>) {
    // Only processes entities where Component changed this frame
}
```

## System Execution Order

Use SystemSets to prevent race conditions:
```rust
#[derive(SystemSet)]
pub enum FontEditorSets {
    Input,       // Handle input
    TextBuffer,  // Update buffer state
    EntitySync,  // Sync ECS entities
    Rendering,   // Create visuals
    Cleanup,     // Clean orphans
}
```

Execution order: Input → TextBuffer → EntitySync → Rendering → Cleanup
