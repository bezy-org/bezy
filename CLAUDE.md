# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bezy is a font editor built with Rust and the Bevy game engine. It's designed as a modern, user-empowerable font editing tool that follows modernist design principles. The codebase is intentionally simple and readable for education purposes.

## Key Technologies

- **Bevy**: ECS-based game engine for the UI framework
- **Norad**: UFO font format parsing and manipulation
- **Kurbo**: 2D curve mathematics
- **Rust**: Core programming language

## Development Commands

### Basic Development
```bash
# Run the application (default: Bezy Grotesk font, glyph 'a')
cargo run

# Run with specific UFO font
cargo run -- --load-ufo <path_to_ufo>

# Run with specific Unicode character
cargo run -- --test-unicode <hex_codepoint>

# Debug mode with verbose logging
cargo run -- --debug --log-level debug
```

### Code Quality
```bash
# Format code (max width: 80 characters per rustfmt.toml)
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test
```

### Build Variants
```bash
# Development build with dynamic linking (faster compilation)
cargo run --features dev

# WASM build for web deployment
./build-wasm.sh
# or manually:
cargo run --target wasm32-unknown-unknown

# Release build
cargo build --release
```

## Architecture Overview

### Module Structure
- **`core/`**: Application initialization, CLI, settings, state management
- **`data/`**: UFO font data handling, Unicode utilities, workspace management
- **`editing/`**: Edit operations, selection system, undo/redo, text editing
- **`geometry/`**: Geometric primitives, paths, points, design space coordinates
- **`rendering/`**: Drawing systems, glyph outlines, visual feedback
- **`systems/`**: Bevy systems for input handling, UI interaction, commands
- **`ui/`**: User interface components, toolbars, panes, themes

### Key Architectural Patterns

#### ECS-Based Design
The application uses Bevy's Entity-Component-System pattern. Major systems include:
- **Selection System**: Manages point/component selection state
- **Edit System**: Handles glyph modifications and undo operations
- **Input System**: Processes keyboard/mouse events
- **Rendering System**: Draws glyphs, UI elements, and visual feedback

#### State Management
- **AppState**: Main application state resource
- **GlyphNavigation**: Current glyph and font navigation
- **BezySettings**: Configuration and preferences

#### Plugin Architecture
The application is composed of plugins:
- **Core plugins**: Input, pointer management, settings
- **UI plugins**: HUD, panes, toolbars
- **Editing plugins**: Selection, undo, text editing
- **Rendering plugins**: Cameras, drawing systems

### Design Philosophy

#### Visual Theming
All visual styling constants MUST be declared in `src/ui/theme.rs`. No visual constants should exist outside this file to enable complete theme swapping.

#### Code Style
- Max line width: 80 characters (enforced by rustfmt.toml)
- Simple, readable code suitable for educational purposes
- Modernist "less is more" approach
- Game-like feel with fast, engaging interactions

#### Rendering Architecture - Unified System
The application uses a **single unified rendering system** located in `src/rendering/unified_glyph_editing.rs` that handles ALL glyph visualization:

##### Core Principles
- **NEVER use Bevy Gizmos**: All world-space visual elements MUST use mesh-based rendering for proper z-ordering, camera-responsive scaling, and visual consistency
- **Single rendering system**: The unified system eliminates coordination complexity and ensures reliable cleanup
- **Camera-responsive scaling**: All visual elements work with the zoom-aware scaling system
- **Mesh-based only**: Gizmos cause problems with layering, scaling, and maintainability

##### Unified Rendering Behavior
- **Active sorts**: Show editable points, handles, and outlines for interactive editing
- **Inactive sorts**: Render as filled shapes using Lyon tessellation with proper winding rules (EvenOdd fill rule)
- **Zero visual lag**: All components (points, handles, outlines) render together using live Transform data
- **Proper vector rendering**: Handles font counters/holes correctly (e.g., the hole in letter 'a')

##### Technical Implementation
- **Lyon tessellation**: Converts vector paths to filled meshes for inactive sorts
- **Combined contours**: All contours processed as single path for correct winding rule handling  
- **EvenOdd fill rule**: Standard font rendering approach that properly handles counters
- **Camera-responsive scaling**: Uses `CameraResponsiveScale` for zoom-appropriate visual sizing
- **Entity tracking**: `UnifiedGlyphEntities` resource tracks all rendered elements for reliable cleanup

### Font Data Model

The application uses FontIR as the primary runtime data structure:
- **FontIR**: The single source of truth for font data that gets modified during editing
- **Data Flow**: Load font sources (UFO, TTF, OTF, etc.) → FontIR runtime structure → Edit FontIR data → Save back to disk
- **Transform Components**: Only for visual positioning in UI, NOT the source of truth
- **Critical**: When editing points, ALWAYS update the underlying FontIR glyph data, not just Transform positions

Legacy UFO support:
- **Font**: Container for all glyphs and metadata
- **Glyph**: Individual character containing contours and components
- **Contour**: Closed or open paths made of points
- **Point**: Curve or line points with coordinates and control handles

### System Architecture

#### System Sets for Guaranteed Execution Order
The application uses Bevy's SystemSet pattern to prevent race conditions and ensure predictable system execution:

```rust
#[derive(SystemSet)]
pub enum FontEditorSets {
    Input,        // Handle keyboard and mouse input
    TextBuffer,   // Update text buffer state
    EntitySync,   // Synchronize ECS entities with buffer state
    Rendering,    // Create visual elements
    Cleanup,      // Clean up orphaned entities
}
```

These sets execute in strict order: Input → TextBuffer → EntitySync → Rendering → Cleanup

#### Component Relationships for Entity Management
Instead of storing entity references in components, use Bevy's idiomatic component relationship pattern:

```rust
#[derive(Component)]
pub struct MetricsFor(pub Entity); // Points to the sort entity

// Query for metrics belonging to a specific sort
fn cleanup_orphaned_metrics(
    metrics_query: Query<(Entity, &MetricsFor), With<MetricsLine>>,
    sort_query: Query<Entity, With<Sort>>,
) {
    for (metrics_entity, metrics_for) in metrics_query.iter() {
        if sort_query.get(metrics_for.0).is_err() {
            // Sort no longer exists, clean up metrics
        }
    }
}
```

**Benefits:**
- Flat entity structure (fastest queries)
- Uses Bevy's change detection (minimal CPU)
- Prevents race conditions through explicit system ordering
- Scales to thousands of entities
- Works with Bevy's grain, not against it

## Testing and Debugging

### Built-in Test Font
The repository includes "Bezy Grotesk" test font at `assets/fonts/bezy-grotesk-regular.ufo` with Latin and Arabic characters for testing.

### Development Features
- Debug overlays for geometry visualization
- Performance monitoring systems
- Coordinate display panes
- Real-time glyph editing feedback

## Build Scripts

- **`build-wasm.sh`**: Creates WASM build for web deployment
- **`build-github-pages.sh`**: Builds and deploys to GitHub Pages
- **`assets/fonts/build-*.sh`**: Font asset processing scripts

## Common Workflows

### Adding New Tools
The toolbar system uses a plugin-based architecture. New editing tools can be added by:
1. Creating a tool struct implementing `EditTool` trait
2. Adding plugin registration in the toolbar module
3. Tools automatically appear in UI with proper ordering

### Font Loading and Saving
**CRITICAL: Always use Norad for UFO operations.** UFO fonts are loaded and saved exclusively through the Norad library:

- **Loading**: Use `norad::Font::load(path)` for all UFO file loading
- **Saving**: Use `font.save(path)` with proper glyph updates via `layer.insert_glyph()`
- **Never bypass Norad**: All UFO reading/writing must go through Norad to ensure format compliance
- **Glyph updates**: Use `font.default_layer_mut().insert_glyph(glyph)` to update glyphs
- **Conversion pattern**: FontIR BezPath → GlyphData → norad::Glyph → UFO file

UFO loading is handled through the `data::ufo` module, while saving is implemented in the file menu system.

#### Known Compatibility Issues
- **Glyphs.app UFO formatting**: UFOs saved by Glyphs.app may have anchor formatting that's incompatible with FontIR
- **FontIR anchor requirements**: FontIR expects specific anchor data structure that may not match other UFO editors
- **Error symptoms**: "Invalid anchor 'top': 'no value at default location'" indicates FontIR/Glyphs compatibility issue
- **Workaround**: Use UFOs created/saved with norad or other FontIR-compatible tools

### Selection System
Point selection uses coordinate-based hit testing with visual feedback. The selection system supports:
- Single point selection
- Multi-select with Shift+click
- Box selection with click+drag
- Keyboard-based nudging with configurable amounts (2, 8, 32 units)

# critical-behavior-guidelines
NEVER make changes the user didn't explicitly ask for, even if they seem helpful.
NEVER change default behaviors, settings, or configurations unless specifically requested.
NEVER add "improvements" or "enhancements" that weren't requested.
ALWAYS ask for clarification if the request is unclear rather than making assumptions.
FOCUS entirely on the specific issue described by the user.
DO NOT be ambitious or make sweeping changes - be precise and minimal.
RESPECT the existing default tool (select) and user workflows.

# camera-zoom-scaling-system
✅ COMPLETED: Camera-responsive scaling system in src/rendering/camera_responsive.rs

## How It Works
The system automatically adjusts visual element sizes (points, lines, handles, metrics) based on camera zoom level to maintain visibility when zoomed out, similar to how Glyphs app works.

## Current Configuration
- **Zoom In**: Elements keep current size (no change needed)
- **Default Zoom**: Elements use normal 1px width
- **Zoom Out**: Elements scale up to 12x bigger for visibility

## Easy Tuning
Edit these values in `src/rendering/camera_responsive.rs` lines 40-46:

```rust
// EASY TO TUNE: Adjust these three scale factors
zoom_in_max_factor: 1.0,    // Keep current size when zoomed in
default_factor: 1.0,        // Keep current size at default zoom
zoom_out_max_factor: 12.0,  // Make 12x bigger when zoomed out

// Camera scale ranges (adjust if needed)
zoom_in_max_camera_scale: 0.2,   // Maximum zoom in
default_camera_scale: 1.0,       // Default zoom level
zoom_out_max_camera_scale: 16.0, // Maximum zoom out
```

## Technical Details
- Uses PanCam's OrthographicProjection.scale for real zoom detection
- Interpolates smoothly between three scale points
- Applies to unified rendering system: points, outlines, handles, filled shapes, metrics
- Completely mesh-based system with proper tessellation
- Debug output shows current camera scale and responsive factor

## Performance
- Minimal overhead - only updates when camera scale changes
- Unified system handles all visual elements consistently
- Zero visual lag during nudging operations
- Single rendering path eliminates coordination complexity

## IMPORTANT: Unified Mesh-Based Rendering Policy
**The unified rendering system uses mesh-based camera-responsive rendering for ALL glyph elements.**

### Unified System Architecture
- ✅ **UNIFIED RENDERING**: All glyph visualization handled by `unified_glyph_editing.rs`
  - Active sorts: Points, handles, editable outlines with camera-responsive scaling
  - Inactive sorts: Filled tessellated shapes with proper winding rules
  - Zero coordination needed - single source of truth for all glyph rendering
  - Use `spawn_unified_line_mesh()` and Lyon tessellation functions
- ❌ **AVOID GIZMOS**: For world-space elements (they don't integrate with camera-responsive scaling)
  - Gizmos acceptable only for UI elements or temporary debug visualization
  - Never use gizmos for preview systems, metrics, or interactive elements

### Implementation Pattern
```rust
// ✅ CORRECT: Mesh-based with camera-responsive scaling
let entity = spawn_metrics_line(
    &mut commands,
    &mut meshes, 
    &mut materials,
    start_pos,
    end_pos,
    color,
    parent_entity,
    line_type,
    &camera_scale, // Always pass camera scale
);

// ❌ INCORRECT: Gizmo-based (doesn't scale with camera)
gizmos.line_2d(start_pos, end_pos, color);
```

This ensures consistent visual scaling and maintains the professional font editor experience at all zoom levels.

## CRITICAL: Visual Flash Prevention Pattern

### Problem: Visual Artifacts During State Transitions
When developing UI systems that manage visual elements, a common issue is **visual flashing** during state transitions (e.g., when placing new sorts). This manifests as brief visual artifacts where elements appear distorted or cross-contaminated.

### Root Cause: Every-Frame Rebuilding
The issue typically stems from systems that rebuild ALL visual elements every frame:
```rust
// ❌ PROBLEMATIC: Every-frame nuclear approach
fn render_system() {
    // Despawn ALL elements every frame
    for entity in existing_elements.iter() {
        commands.entity(entity).despawn();
    }
    // Rebuild everything from scratch
    spawn_all_elements();
}
```

This creates a 1-frame gap where elements are despawned but not yet respawned, causing visual artifacts.

### Solution: Change-Detection Based Updates
Use Bevy's change detection to only rebuild when actually needed:

```rust
// ✅ CORRECT: Change-detection based approach
#[derive(Resource, Default)]
pub struct VisualUpdateTracker {
    pub needs_update: bool,
}

fn detect_changes(
    mut tracker: ResMut<VisualUpdateTracker>,
    changed_query: Query<Entity, (With<SomeComponent>, Changed<SomeComponent>)>,
) {
    if !changed_query.is_empty() {
        tracker.needs_update = true;
    }
}

fn render_system(mut tracker: ResMut<VisualUpdateTracker>) {
    // Only rebuild when actually needed
    if !tracker.needs_update {
        return; // Early exit prevents unnecessary rebuilding
    }
    tracker.needs_update = false;
    
    // Rebuild elements atomically
    rebuild_visual_elements();
}
```

### Key Benefits:
- **Eliminates Visual Flash**: Elements only update when necessary
- **Better Performance**: Avoids expensive every-frame rebuilding
- **Atomic Updates**: Changes happen in a single frame when needed
- **Maintains Responsiveness**: Still updates immediately when state changes

### Implementation Notes:
- Use `.chain()` to ensure change detection runs before rendering
- Track state with dedicated resources rather than every-frame queries
- Leverage `Changed<T>`, `Added<T>`, and `RemovedComponents<T>` for precise change detection
- Consider this pattern for any system managing dynamic visual elements

This pattern is essential for maintaining professional-quality UI responsiveness without visual artifacts.

## CRITICAL: Cross-Sort Contamination Prevention

### Problem: Points From Multiple Sorts Getting Mixed
When rendering sorts in a unified system, a critical issue can occur where points from different sort entities get collected together, causing visual contamination:
- **Handle lines connecting points from different sorts**
- **Points appearing in wrong locations during state transitions**
- **Visual "flash" when placing new sorts with the same glyph**

### Root Cause: Filtering by Glyph Name Instead of Sort Entity
```rust
// ❌ PROBLEMATIC: Causes cross contamination between sorts
for (point_entity, transform, point_ref, ...) in point_query.iter() {
    if point_ref.glyph_name == sort.glyph_name {
        // WRONG: Collects points from ALL sorts with same glyph!
        sort_points.push(point_entity);
    }
}
```

This causes multiple sorts with glyph 'a' to collect each other's points, leading to errant handle connections.

### Solution: Filter Points by Sort Entity Ownership
```rust
// ✅ CORRECT: Each sort only collects its own points
// Include SortPointEntity in the query
point_query: Query<(Entity, &Transform, &GlyphPointReference, &SortPointEntity)>,

// Filter by the sort entity that owns each point
for (point_entity, transform, point_ref, sort_point_entity) in point_query.iter() {
    if sort_point_entity.sort_entity == current_sort_entity {
        // Only collect points that belong to THIS specific sort
        sort_points.push(point_entity);
    }
}
```

### Key Implementation Details:
1. **Always track sort ownership** in point entities using a component like `SortPointEntity`
2. **Filter by entity ownership**, not by shared properties like glyph name
3. **Validate sort consistency** when collecting points for rendering
4. **Use selective clearing** - only despawn/respawn elements for sorts that actually changed

### Benefits:
- **Complete sort isolation**: Multiple sorts with same glyph remain independent
- **No visual contamination**: Handles only connect points within same sort
- **Stable rendering**: Unchanged sorts remain visually stable during updates
- **Better performance**: Selective updates instead of nuclear rebuilding

This pattern is critical when implementing any multi-instance editing system where multiple copies of the same data can coexist.