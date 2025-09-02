# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with the Bezy font editor codebase.

## Project Overview

Bezy is a next-gen font editor built with Rust and Bevy, designed for customization, human-AI collaboration, and user empowerment. It tries to be simple, understandable, and easy to modify as a core design principle.

### Core Technologies
- **Rust**: Primary language with focus on readability and education
- **Bevy 0.16**: ECS-based game engine for UI framework
- **Norad**: UFO font format parsing (CRITICAL: all UFO operations must use Norad)
- **Kurbo**: 2D curve mathematics
- **FontC**: For compiling and exporting fonts
- **FontIR**: Primary runtime data structure for font editing, this is a temporary experiment to see how far we can take it, when/if we hit a all with this we will have a better understanding of what is need an create something that works well with FontC/FontIR.  
- **Lyon**: Tessellation for filled glyph rendering

## Quick Start Commands

```bash
# Run the app with default font (Bezy Grotesk, glyph 'a')
cargo run

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
├── core/       # App initialization, CLI, settings, state management
├── data/       # Font data handling (UFO, FontIR), Unicode utilities
├── editing/    # EDITING LOGIC: selection, undo/redo, glyph modifications
├── geometry/   # Mathematical primitives, bezier curves, coordinates
├── rendering/  # PURE RENDERING: visual display only, no editing logic
├── systems/    # Input handling, text layout, command processing
├── tools/      # User interaction tools (select, pen, knife, etc.)
├── ui/         # Interface components, toolbars, panes, themes
└── utils/      # Helper utilities
```

**Key Separation:**
- **Editing** (`src/editing/`): Modifies font data, handles user edits
- **Rendering** (`src/rendering/`): Displays data visually, no modifications
- **Tools** (`src/tools/`): User interaction and tool-specific logic

### Key Design Patterns

#### ECS Architecture
- Uses Bevy's Entity-Component-System pattern
- Major systems: Selection, Edit, Input, Rendering
- Resources: AppState, GlyphNavigation, BezySettings

#### Font Data Model
- **FontIR**: Single source of truth for font data
- **Data Flow**: Load sources → FontIR → Edit FontIR → Save to disk
- **Transform vs FontIR**: 
  - Transform components = temporary visual display only (what you see on screen)
  - FontIR data = permanent font data (what gets saved to disk)
  - When editing: Update BOTH Transform (for immediate visual feedback) AND FontIR (for data persistence)
  - **Common bug**: Updating only Transform means changes look correct but won't save

## Critical Implementation Rules

### 1. Glyph Rendering System
Located in `src/rendering/glyph_renderer.rs`:
- **Pure rendering logic** - displays glyphs visually (no editing logic)
- **Single-system approach**: All glyph elements (points, handles, outlines) render together to prevent visual lag
- **Mesh-based only**: Never use Bevy Gizmos for world-space elements 
- Active sorts: Show editable points, handles, outlines
- Inactive sorts: Filled shapes via Lyon tessellation with EvenOdd fill rule

### 2. Visual Theming
- **ALL visual constants** must be in `src/ui/theme.rs`
- No visual constants outside theme file
- Enables complete theme swapping

### 3. Font Operations
- **ALWAYS use Norad** for UFO loading/saving
- Loading: `norad::Font::load(path)`
- Saving: `font.save(path)` with `layer.insert_glyph()`
- Conversion: FontIR BezPath → GlyphData → norad::Glyph → UFO

### 4. Zoom-Aware Scaling System
Located in `src/rendering/zoom_aware_scaling.rs`:
- Makes mesh-rendered elements (points, lines, handles) stay visually consistent at all zoom levels
- Similar to how other font editors keep UI elements visible when zoomed out
- Tuning values for easy customization
- Applied automatically to all mesh-based rendering throughout the app

## Development Guidelines

### Code Style
- Max 100 characters per line
- Simple, readable code, try not to be too clever and overly complex
- "less is more" approach, don't over-complicate things
- NO COMMENTS unless explicitly requested, try to make the code readable

### Working Principles
- **DO NOT** make changes not explicitly requested, if you have a good idea ask first
- **DO NOT** change defaults without user request
- **DO NOT** add unrequested "improvements" without asking first
- **ALWAYS** ask for clarification if unclear
- **FOCUS** on the specific issue described

### Adding New Tools
1. Create struct implementing `EditTool` trait
2. Add plugin registration in toolbar module
3. Tool automatically appears in UI

## Default and Test Assets

Built-in test font (UFO/Designspace) with Latin and Arabic support
Designspace: `assets/fonts/bezy-grotesk.designspace`
Regular: `assets/fonts/bezy-grotesk-regular.ufo`
Bold: `assets/fonts/bezy-grotesk-Bold.ufo`

## Known Issues

### Glyphs.app Compatibility
- UFOs from Glyphs.app may have incompatible anchor formatting
- Error: "Invalid anchor 'top': 'no value at default location'"
- Workaround: Use UFOs created with norad or FontIR-compatible tools

## Performance Patterns

### Visual Flash Prevention
Use change detection instead of every-frame rebuilding:
```rust
// Track changes with a resource
#[derive(Resource, Default)]
pub struct VisualUpdateTracker {
    pub needs_update: bool,
}

// Only rebuild when needed
if !tracker.needs_update { return; }
```

### Cross-Sort Contamination Prevention
Filter points by sort entity ownership, not glyph name:
```rust
// CORRECT: Filter by entity ownership
if sort_point_entity.sort_entity == current_sort_entity {
    sort_points.push(point_entity);
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
