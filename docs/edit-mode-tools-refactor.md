# Edit Mode Tools Architecture Refactor

## Executive Summary

The edit mode tools are the heart of Bezy's user interaction, but the current architecture has become brittle and error-prone. This document consolidates all refactor planning and provides a single source of truth for fixing the tool system.

## Current Architecture Problems

### 1. Fragmented State Management
- Tool state scattered across multiple resources (`SelectModeActive`, `PenModeActive`, `InputMode`, etc.)
- No single source of truth for which tool is active
- State synchronization issues between different systems

### 2. Performance Issues
- **Input lag**: Multiple layers of indirection before input reaches tools
- **Visual delays**: Tool switches don't immediately update visuals
- **Cleanup failures**: Previous tool's visuals remain after switching

### 3. Input Routing Complexity
- 888-line `input_consumer.rs` file handles all tool routing
- Multiple layers: Input → InputEvent → InputConsumer → SpecificConsumer → process_events → actual logic
- Each layer adds a frame of latency

### 4. Broken Tools
- **Select Tool**: Not activating properly, selection state not clearing
- **Pen Tool**: Coordinate system confusion, path not closing, preview not showing
- **Knife Tool**: Cut preview not rendering, state not resetting
- **Measure Tool**: Measurements not displaying

## Solution: Performance-First ECS-Native Architecture

### Core Design Principles

1. **Work WITH Bevy ECS**: Use Resources, Components, Systems, Events, and System Sets
2. **Single Source of Truth**: One unified tool state system
3. **Direct Input Path**: Tool gets input in the same frame it occurs
4. **Immediate Visual Feedback**: No frame delays between action and visual
5. **Zero-Cost Tool Switching**: Instant cleanup and activation

### Implementation Status

#### ✅ Completed: Unified Tool State

Created `src/tools/tool_state.rs` with:

```rust
#[derive(Resource, Debug, Default)]
pub struct ToolState {
    pub active: ToolId,
    active_changed: bool,
    previous: Option<ToolId>,
    temporary_stack: Vec<ToolId>,  // For spacebar-style modes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolId {
    #[default]
    Select, Pen, Knife, Pan, Text, Shapes, Measure, Hyper, Metaballs, Ai,
}
```

Features:
- Single source of truth for active tool
- Temporary tool stack for spacebar pan
- Change detection for cleanup systems
- Event-based tool switching

#### 🚧 In Progress: Tool Integration

Modified select tool to use unified state:
- Syncs `SelectModeActive` with `ToolState`
- Proper activation/deactivation
- Needs testing

Added TODO in `config_loader.rs` for human contribution:
- Send `SwitchToolEvent` instead of setting resources directly

#### ⏳ Pending: Simplify Input Routing

Replace complex input consumer chain with direct dispatch:

```rust
fn route_input(
    tool_state: Res<ToolState>,
    mut input: ResMut<InputState>,
    mut select_state: ResMut<SelectState>,
    mut pen_state: ResMut<PenState>,
) {
    // Direct dispatch - no indirection
    match tool_state.active {
        ToolId::Select => handle_select_input(&input, &mut select_state),
        ToolId::Pen => handle_pen_input(&input, &mut pen_state),
        // ...
    }
}
```

### System Architecture

#### System Sets for Organization

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ToolSystemSet {
    ToolManagement,  // Handle tool switching
    ToolInput,       // Process input for active tool
    ToolUpdate,      // Run active tool's logic
    ToolRender,      // Render tool-specific visuals
    ToolCleanup,     // Clean up after tool changes
}
```

Execution order:
```
ToolManagement → ToolInput → ToolUpdate → ToolRender → ToolCleanup
```

#### Events for Communication

```rust
#[derive(Event)]
pub struct SwitchToolEvent {
    pub tool: ToolId,
    pub temporary: bool,
}

#[derive(Event)]
pub struct ToolActivated {
    pub tool_id: ToolId,
    pub previous: Option<ToolId>,
}

#[derive(Event)]
pub struct ToolDeactivated {
    pub tool_id: ToolId,
}
```

#### Tool-Specific Components

```rust
#[derive(Component)]
pub struct ToolOwned {
    pub tool_id: ToolId,  // Mark entities created by specific tool
}

#[derive(Component)]
pub struct ToolPreview;  // Mark temporary tool visuals
```

### TUI Integration

Tools communicate with TUI via events:

```rust
// Tool → TUI
pub enum TuiRequest {
    ShowUnicodePanel,      // For text tool
    UpdateStatus(String),
    ShowPanel(PanelType),
}

// TUI → Tool
pub enum TuiResponse {
    UnicodeSelected(char),
    InputProvided(String),
    PanelAction(PanelAction),
}
```

### Performance Optimizations

#### Entity Pooling
Pre-allocate visual entities, toggle visibility instead of spawn/despawn:

```rust
#[derive(Resource)]
pub struct ToolVisualPools {
    select: SelectVisualPool,
    pen: PenVisualPool,
}

pub struct SelectVisualPool {
    marquee: Entity,           // Pre-allocated, just toggle visibility
    hover_highlight: Entity,
    selection_outlines: Vec<Entity>,
}
```

#### Direct Updates
- No queuing or delays
- Input processed same frame
- Visuals updated immediately

## Implementation Plan

### Phase 1: Foundation (✅ COMPLETED)
- [x] Create unified ToolState resource
- [x] Add tool lifecycle events
- [x] Add ToolStatePlugin to app
- [x] Integrate toolbar with SwitchToolEvent
- [x] Update spacebar toggle for temporary modes
- [x] Write and pass unit tests for ToolState

### Phase 2: Fix Core Tools (✅ SELECT & PEN DONE)
- [x] Fix select tool completely
  - [x] Direct input handling (bypasses input_consumer.rs)
  - [x] Click selection with shift multi-select
  - [x] Marquee selection (drag to select)
  - [x] Visual feedback (marquee rectangle)
- [x] Fix pen tool
  - [x] Direct input handling
  - [x] Instant path preview with gizmos
  - [x] Line to cursor while drawing
  - [x] Tool state cleanup on deactivation
- [ ] Fix knife tool
  - [ ] Cut preview
  - [ ] State reset

### Phase 3: Simplify Input
- [ ] Replace input_consumer.rs with direct routing
- [ ] Remove InputConsumer trait layers
- [ ] Test input responsiveness

### Phase 4: Performance
- [ ] Implement entity pooling
- [ ] Profile and optimize
- [ ] Ensure < 1 frame input latency

### Phase 5: Polish
- [ ] Add tool validation
- [ ] Error recovery
- [ ] Developer documentation

## Tool-Specific Fixes

### Select Tool
```rust
// Ensure single source of truth
fn sync_select_mode(tool_state: Res<ToolState>, mut select_mode: ResMut<SelectModeActive>) {
    select_mode.0 = tool_state.is_active(ToolId::Select);
}
```

### Pen Tool
```rust
// Fix coordinate system
fn pen_tool_input(world_pos: Vec2, active_sort_pos: Vec2) -> Vec2 {
    world_pos - active_sort_pos  // Convert to relative coordinates
}

// Show preview immediately
fn pen_preview(pen_state: Res<PenState>, mut gizmos: Gizmos) {
    for window in pen_state.path_points.windows(2) {
        gizmos.line_2d(window[0], window[1], Color::BLUE);
    }
}
```

### Knife Tool
```rust
// Ensure state cleanup
fn knife_cleanup(tool_state: Res<ToolState>, mut knife_state: ResMut<KnifeState>) {
    if !tool_state.is_active(ToolId::Knife) && tool_state.just_changed() {
        knife_state.gesture = KnifeGestureState::Ready;
        knife_state.intersections.clear();
    }
}
```

## Success Metrics

- **Input latency**: < 1 frame from input to visual
- **Tool switch time**: < 1ms
- **Memory allocations**: 0 during normal use
- **Visual updates**: 60fps maintained
- **Developer experience**: New tool in < 100 lines

## Recent Fixes

- **Fixed startup panic**: InputMode resource was not initialized, causing "Resource does not exist" error
- **Solution**: Added `.insert_resource(InputMode::Normal)` to ToolStatePlugin
- **Build cleaned**: Fixed all compilation errors, reduced warnings from 91 to 23

## Current Status

### What's Working
- ✅ Unified ToolState resource provides single source of truth
- ✅ Tool switching via SwitchToolEvent
- ✅ Temporary tool modes (spacebar pan)
- ✅ Integration with existing toolbar UI
- ✅ Unit tests passing for core functionality
- ✅ **Select tool fully working with direct input** (bypasses input_consumer.rs)
  - Click selection
  - Shift-click multi-select
  - Marquee drag selection
  - Visual feedback
- ✅ **Pen tool with instant preview** (direct input)
  - Immediate path rendering with gizmos
  - Line to cursor while drawing
  - Clean state management

### What Still Needs Work
- ❌ Knife, Measure, and other tools still use input_consumer.rs
- ❌ Legacy resources (SelectModeActive, PenModeActive) still exist but synced
- ❌ 888-line input_consumer.rs still needed for legacy tools

## Next Steps

1. **Port Knife Tool** - Direct input for cut preview
2. **Port Measure Tool** - Direct input for measurement display
3. **Port remaining tools** - Shapes, Text, etc.
4. **Remove input_consumer.rs** - Once all tools use direct input
5. **Clean up legacy resources** - SelectModeActive, PenModeActive, etc.
6. **Performance validation** - Confirm 0-frame input latency

## Notes

- Always use Bevy logging (`info!`, `debug!`, etc) - NEVER `println!` (breaks TUI)
- Test with `cargo run --release` for accurate performance
- Keep performance as top priority - users notice lag immediately