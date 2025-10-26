# Bezy Edit Mode Tools - Comprehensive Architecture Analysis

**Date**: 2025-10-25  
**Status**: Complete Analysis Report  
**For**: Cleanup/Refactoring Planning

---

## Executive Summary

The Bezy codebase has a well-organized tools system with both strengths and areas needing cleanup. This comprehensive analysis identifies:

1. **NO Gizmo violations in tools** - All tool rendering properly uses mesh-based systems
2. **NO duplicate tools** - Clear separation between tool implementations and UI
3. **CRITICAL: Point visibility bug identified** - Points hidden in pan/preview mode due to ActiveSort check
4. **CRITICAL: Gizmo violation in selection** - Selection rendering uses Gizmos (violates CLAUDE.md)
5. **Well-structured toolbar** - Configuration-based system is clean and maintainable
6. **Input routing works** - But plugin registration could be clearer

---

## Part 1: All Edit Mode Tools (/src/tools/)

### Complete Tool Inventory

| Tool | File | Lines | Status | Input Method | Rendering |
|------|------|-------|--------|--------------|-----------|
| **Select** | select.rs | 397 | ✅ Working | InputEvent via InputConsumer | `/src/editing/selection/` |
| **Pan** | pan.rs | 41 | ✅ Minimal | InputMode::Temporary | Camera system |
| **Pen** | pen.rs | 865 | ✅ Working | Direct + mesh preview | `render_pen_preview()` |
| **Text** | text.rs | 131 | ✅ Working | InputEvent via InputConsumer | Text buffer system |
| **Shapes** | shapes.rs | 348 | ✅ Working | Direct input | `render_shapes_preview()` |
| **Knife** | knife.rs | 341 | ✅ Working | Direct input | `render_knife_preview()` |
| **Measure** | measure.rs | 339 | ✅ Working | Direct input | `render_measure_tool()` |
| **Hyper** | hyper.rs | 47 | 🔶 Stub | None | None |
| **Metaballs** | metaballs.rs | 47 | 🔶 Stub | None | None |
| **AI** | ai.rs | 386 | 🔶 Stub | Framework only | None |

### Key Tools Analysis

#### Select Tool (/src/tools/select.rs:1-397)
- **Purpose**: Activate selection mode, delegate to primary selection system
- **Architecture**: Wrapper around `/src/editing/selection/` module
- **State Management**: `SelectModeActive` (synced with `ToolState`)
- **Input**: Via `SelectionInputConsumer` in `input_consumer.rs` (line ~300)
- **Disabled Code**: `handle_select_tool_input()` (lines 71-245) - Intentionally disabled
  - Comment on line 68: "Conflicts with existing selection system in /src/editing/selection/"
- **Systems**:
  - `handle_select_tool_activation` - Syncs tool state
  - `sync_select_mode_with_tool_state` - Keeps resources synchronized
- **Status**: Fully functional, correct delegation pattern

#### Pen Tool (/src/tools/pen.rs:1-865)
- **Purpose**: Draw Bézier paths and curves
- **State**: `PenToolState` with `current_path: Vec<DPoint>`, `is_drawing: bool`
- **Input**: Direct via `handle_pen_mouse_events()` and `handle_pen_keyboard_events()`
- **Rendering**: `render_pen_preview()` (lines 474-865)
  - Creates **Circle meshes** for preview points
  - Draws lines between preview points
  - Cleans up existing preview entities per frame
- **Systems Registered**:
  - `sync_pen_mode_with_tool_state`
  - `handle_pen_mouse_events`
  - `handle_pen_keyboard_events`
  - `render_pen_preview`
  - `reset_pen_mode_when_inactive`
  - `debug_pen_tool_state`
- **Rendering Details**: Proper mesh-based, theme-integrated
- **Status**: Fully functional, excellent implementation

#### Knife Tool (/src/tools/knife.rs:1-341)
- **Purpose**: Cut/slice contours at specific points
- **State**: `KnifeToolState` with `KnifeGestureState` (Ready or Cutting)
- **Input**: Direct via `handle_knife_direct_input()`
- **Rendering**: `render_knife_preview()` (lines 185-277)
  - Creates **Rectangle mesh** for cutting line preview
  - Supports axis-constrained cuts (shift key)
  - Properly despawns preview when not cutting
- **Systems**:
  - `handle_knife_direct_input`
  - `render_knife_preview`
  - `sync_knife_mode_with_tool_state`
- **Status**: Fully functional, proper mesh-based rendering

#### Shapes Tool (/src/tools/shapes.rs:1-348)
- **Purpose**: Create geometric primitives (Rectangle, Oval, RoundedRectangle)
- **State**: `ShapesToolState` with `is_drawing`, `shape_type`, position data
- **Input**: Direct via `handle_shapes_direct_input()`
- **Rendering**: `render_shapes_preview()` (lines 213-310)
  - **Rectangle mesh** for shape preview
  - Dynamically sized based on drag distance
  - Cleans up before rendering new preview
- **Systems**:
  - `handle_shapes_direct_input`
  - `render_shapes_preview`
  - `sync_shapes_mode_with_tool_state`
- **Status**: Fully functional, proper implementation

#### Measure Tool (/src/tools/measure.rs:1-339)
- **Purpose**: Measure distances and angles
- **State**: `MeasureToolState` with measurement type enum
- **Input**: Direct via `handle_measure_direct_input()`
- **Rendering**: `render_measure_tool()`
  - Uses **meshes** for distance/angle visualization
  - Supports axis-constrained measurements (shift key)
- **Systems**:
  - `handle_measure_direct_input`
  - `render_measure_tool`
  - `sync_measure_mode_with_tool_state`
- **Status**: Fully functional

#### Text Tool (/src/tools/text.rs:1-131)
- **Purpose**: Place and edit sorts (glyphs in text editing mode)
- **State**: `TextModeActive`, `CurrentTextPlacementMode` (LTR/RTL/Insert/Freeform)
- **Input**: Via `TextInputConsumer` in `input_consumer.rs` (complex integration)
- **Rendering**: Handled by text buffer system, not in tools
- **Status**: Minimal code, complex behavior integrated with text buffer

#### Pan Tool (/src/tools/pan.rs:1-41)
- **Purpose**: Navigate/pan the design space
- **State**: `InputMode::Temporary` for temporary spacebar activation
- **Input**: Handled by spacebar toggle system
- **Rendering**: None (handled by camera system)
- **Status**: Minimal, correct delegation pattern

#### Hyper Tool (/src/tools/hyper.rs:1-47)
- **Status**: 🔶 **STUB** - No real implementation
- **Components**: Only `HyperTool` struct and `HyperToolPlugin`
- **Toolbar**: Disabled (`enabled: false` in toolbar_config.rs:143)
- **Note**: Comment suggests submenu under Pen tool

#### Metaballs Tool (/src/tools/metaballs.rs:1-47)
- **Status**: 🔶 **STUB** - No real implementation
- **Components**: Only `MetaballsTool` struct and `MetaballsToolPlugin`
- **Toolbar**: Disabled (`enabled: false` in toolbar_config.rs:163)

#### AI Tool (/src/tools/ai.rs:1-386)
- **Status**: 🔶 **STUB** - Framework exists but no functionality
- **Submenu System**: Lines 261-368 show UI structure
- **Operations Defined**: Kerning, LanguageSupport, OpticalAdjustment, WeightFix, CurveSmoothing
- **Plugin**: Registers submenu but no actual AI operations
- **Status**: Framework in place, implementation needed

---

## Part 2: Gizmo Usage - CRITICAL ISSUE FOUND

### ✅ Tools Rendering (Correct)

**Result**: All tool renderings use mesh-based systems correctly.

Tools checked:
- ✅ Pen tool: Uses `Mesh2d`, `Circle`, `Rectangle` (NOT gizmos)
- ✅ Knife tool: Uses `Rectangle` meshes (NOT gizmos)
- ✅ Shapes tool: Uses shape meshes (NOT gizmos)
- ✅ Measure tool: Uses meshes for visualization (NOT gizmos)

### ❌ Selection Rendering (VIOLATION)

**File**: `/src/rendering/selection.rs`

**Function**: `render_selection_marquee()` (lines 23-71)
```rust
pub fn render_selection_marquee(
    _commands: Commands,
    mut gizmos: Gizmos,  // <-- GIZMOS USED
    drag_state: Res<DragSelectionState>,
    marquee_query: Query<(Entity, &SelectionRect)>,
    theme: Res<CurrentTheme>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
) {
    // Lines 66-69: Draw dashed rectangle with gizmos
    draw_dashed_line(&mut gizmos, p1, p2, color, 8.0, 4.0);  // Line 66-69
}

fn draw_dashed_line(
    gizmos: &mut Gizmos,  // <-- HELPER ALSO USES GIZMOS
    start: Vec2,
    end: Vec2,
    color: Color,
    dash_length: f32,
    gap_length: f32,
) {
    // Lines 88-94: Use gizmos.line_2d()
}
```

**Function**: `render_selected_entities()` (lines 104-196)
- Lines 144-163: `gizmos.rect_2d()` and `gizmos.circle_2d()` for selection outlines
- Lines 170-179: `gizmos.line_2d()` for crosshairs

**Impact**: 
- Severity: **MEDIUM** - Selection UI works but violates CLAUDE.md architecture rules
- Violation: CLAUDE.md line 219 states "No gizmos, mesh-based rendering only"
- Inconsistency: Tools use meshes, selection UI uses gizmos

**Fix Required**: Convert `render_selection_marquee()` and `render_selected_entities()` to mesh-based rendering

---

## Part 3: Point Rendering System - CRITICAL BUG

### Where Points Are Rendered

**Primary**: `/src/rendering/points.rs:28-287`
- **Function**: `render_points_with_meshes()`
- **System**: Registered in `PointRenderingPlugin` (line 297)
- **Mesh Type**: 
  - Off-curve: `Circle` meshes
  - On-curve: `Rectangle` (theme option) or `Circle`
  - Three-layer system: Outline + middle + center

**Secondary**: `/src/rendering/selection.rs:104-249`
- **Function**: `render_selected_entities()` (uses gizmos)
- **Function**: `render_all_point_entities()` (uses gizmos)

### CRITICAL BUG: Early Return Hides Points

**File**: `/src/rendering/points.rs:40-62`

```rust
let active_sort_count = active_sorts.iter().count();

info!("🎨 [render_points_with_meshes] CALLED - active_sorts={}, all_points={}", 
      active_sort_count, _all_point_count);

// Early return if no active sorts
if active_sort_count == 0 {
    info!("🎨 [render_points_with_meshes] No active sorts - early return");
    
    // Clean up existing point meshes when no active sorts
    for entity in existing_point_meshes.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();  // DESPAWN ALL POINTS
        }
    }
    return;  // EARLY EXIT - NO POINTS RENDERED
}
```

**Problem**: When no sort has `ActiveSort` component:
1. `active_sorts` query returns 0 entities
2. Function returns early at line 52
3. ALL existing point meshes are despawned
4. **Points become invisible**

### When This Happens

**Conditions that trigger the bug**:
- Pan mode (no sort is "active")
- Preview mode (no active editing sort)
- Any mode where no entity has `ActiveSort` component

**Expected Behavior**:
- Pan mode: Points remain visible but unselectable
- Preview mode: Points visible but read-only
- Edit mode: Points visible and editable

**Actual Behavior**:
- Points completely hidden when no `ActiveSort` exists

### Root Cause

The `ActiveSort` component is misused for visibility control:
1. **Correct use**: Marking the currently edited sort ✅
2. **Incorrect use**: Controlling point visibility ❌

**Files involved**:
- `/src/editing/sort/components.rs` - Defines `ActiveSort`
- `/src/editing/sort/manager.rs` - Sets/removes `ActiveSort`
- `/src/rendering/points.rs` - Uses presence to control rendering

### Query Shows the Issue

**Lines 36-39**:
```rust
all_point_entities: Query<
    (Entity, &GlobalTransform, &PointType, Option<&Selected>),
    With<SortPointEntity>,  // Queries ALL point entities
>,
```

This queries ALL points (not filtered by `ActiveSort`).
But rendering only happens if ANY `ActiveSort` exists (lines 47-52).

**Result**: Query returns points, but function exits before rendering them.

### Comparison: Selection Rendering

**Function**: `render_all_point_entities()` (selection.rs:202-249)
- Does **NOT** check for `ActiveSort`
- Uses Gizmos (legacy code, but consistent approach)
- Attempts to render all points regardless

This is almost identical to points.rs but:
1. Uses Gizmos (violation)
2. Doesn't have the `ActiveSort` check
3. Actually renders points even when no `ActiveSort`

---

## Part 4: Plugin Registration - Incomplete Visibility

### Current Plugin Loading

**File**: `/src/core/app/plugins.rs` (88 lines)

**EditorPluginGroup::build()** (lines 66-87):
```rust
.add(crate::tools::ToolStatePlugin)             // Line 80
.add(EditModeToolbarPlugin)                     // Line 81
.add(crate::tools::PenToolPlugin)               // Line 84
.add(crate::tools::SelectToolPlugin)            // Line 85
```

**Explicitly Registered Tool Plugins**:
- ✅ PenToolPlugin (line 84)
- ✅ SelectToolPlugin (line 85)

**NOT Explicitly Registered** (but should be):
- ❌ KnifeToolPlugin
- ❌ MeasureToolPlugin
- ❌ ShapesToolPlugin
- ❌ TextToolPlugin

### Why Other Tools Still Work

**Dynamic Registration**: `/src/ui/edit_mode_toolbar/config_loader.rs`

The `ConfigBasedToolbarPlugin` (lines 78-120) dynamically registers tools:
```rust
// Pseudo-code from config_loader.rs
for tool_config in ToolConfig::get_enabled_tools() {
    match tool_config.behavior {
        ToolBehavior::Knife => app.add_plugins(crate::tools::KnifeToolPlugin),
        ToolBehavior::Measure => app.add_plugins(crate::tools::MeasureToolPlugin),
        ToolBehavior::Shapes => app.add_plugins(crate::tools::ShapesToolPlugin),
        ToolBehavior::Text => app.add_plugins(crate::tools::TextToolPlugin),
        // ...
    }
}
```

**Result**: Tools work because they're registered dynamically from config.

### Analysis

**Positive Aspects**:
- ✅ Dynamic registration works
- ✅ Tools can be disabled by changing config
- ✅ Centralized configuration in `toolbar_config.rs`

**Negative Aspects**:
- ❌ Plugin loading is hidden in config_loader (not obvious)
- ❌ Explicit plugins.rs registration is incomplete and misleading
- ❌ Would silently fail if someone removed config_loader without knowing

### Recommendation

Add explicit registration to EditorPluginGroup:
```rust
.add(crate::tools::SelectToolPlugin)
.add(crate::tools::PenToolPlugin)
.add(crate::tools::KnifeToolPlugin)
.add(crate::tools::MeasureToolPlugin)
.add(crate::tools::ShapesToolPlugin)
.add(crate::tools::TextToolPlugin)
```

**Rationale**: Makes plugin loading visible and maintainable

---

## Part 5: Selection System Architecture

### Primary Location

**Module**: `/src/editing/selection/` (28 files)

**Key Components** (`components.rs`):
- `Selected` - Marks selected entities
- `Selectable` - Marks entities that can be selected
- `SelectionRect` - Data for marquee rectangle
- `SelectionState` - Resource tracking selected entities
- `PointType` - Defines point characteristics (on-curve vs off-curve)

**Key Systems**:
- `handle_selection_click()` - Single clicks and area clicks
- `handle_selection_drag()` - Marquee rectangle drag selection
- `handle_selection_release()` - Finalizing drag operations
- `handle_smooth_point_toggle()` - Shift+S for smooth points

**Resource States**:
- `DragSelectionState` - Current marquee drag operation
- `DragPointState` - Point dragging operation
- `DoubleClickState` - Double-click tracking

### Select Tool Integration

**Tool**: `/src/tools/select.rs` (397 lines)

**Pattern**: Minimal wrapper around primary selection system
1. User clicks "Select" toolbar button
2. `SwitchToolEvent` sent with `ToolId::Select`
3. SelectToolPlugin's `sync_select_mode_with_tool_state()` runs
4. Sets `SelectModeActive(true)`
5. Selection system in `/src/editing/selection/` handles everything

**Input Flow**:
- `/src/systems/input_consumer.rs` (line ~300): `SelectionInputConsumer` active
- Gathers mouse events when `InputMode::Select`
- Stores in `SelectionInputConsumer::pending_events`
- Main selection system processes events

### Selection Rendering

**Location 1**: `/src/rendering/selection.rs`
- Uses **Gizmos** (VIOLATION)
- Renders selection marquee (drag rectangle)
- Renders selection feedback (crosshairs, highlighting)

**Location 2**: `/src/rendering/points.rs`
- Uses **Meshes** (correct)
- Renders actual point entities
- Has the `ActiveSort` bug (points hidden when no active sort)

### Known Issues

**Documentation**: `/docs/debugging-selection-deselect.md`
- Selection margin too large prevents empty-space clicks
- Margin scales with camera zoom (zoom-aware)
- Can cause issues at different zoom levels

**Complex Logic**: `/src/editing/selection/input/mouse.rs`
- Lines 607+: `handle_selection_click()` (200+ lines of complexity)
- Lines 713+: Point found logic with margin calculation
- Lines 811+: Empty space logic with zoom-aware margin

---

## Part 6: Toolbar Configuration System

### Configuration Source

**File**: `/src/ui/edit_mode_toolbar/toolbar_config.rs` (221 lines)

**ToolConfig Structure** (lines 16-40):
- `order`: Display position (10-90)
- `id`: Unique identifier
- `name`: Display name
- `icon`: Unicode character
- `shortcut`: Optional keyboard shortcut
- `enabled`: Visibility flag
- `behavior`: What tool does
- `description`: Tooltip text

**ToolBehavior Enum** (lines 44-55):
- Select, Pan, Pen, Text, Shapes, Knife, Hyper, Measure, Metaballs, Ai

**TOOLBAR_TOOLS Array** (lines 76-177):
- All 10 tools defined in one place
- Easy to reorder, enable/disable, modify

**Example** (lines 77-86):
```rust
ToolConfig {
    order: 10,
    id: "select",
    name: "Select",
    icon: "\u{E010}",
    shortcut: Some('v'),
    enabled: true,
    behavior: ToolBehavior::Select,
    description: "Select and move points, handles, and components",
}
```

### Configuration Loading

**File**: `/src/ui/edit_mode_toolbar/config_loader.rs` (120+ lines)

**ConfigBasedToolbarPlugin**:
- Runs at app startup
- Iterates through `TOOLBAR_TOOLS` configuration
- For each enabled tool, registers appropriate plugin
- Creates toolbar buttons dynamically

### Keyboard Shortcuts

**File**: `/src/ui/edit_mode_toolbar/keyboard_shortcuts.rs`

**System**: `handle_toolbar_keyboard_shortcuts()`
- Reads all configured shortcuts
- Sends `SwitchToolEvent` when shortcut pressed
- Respects text mode (disables shortcuts when typing)

### Status

**Quality**: ✅ Excellent - Configuration-based, clean, maintainable
**Maintainability**: ✅ Easy to add/remove/reorder tools
**Integration**: ✅ Properly integrated with tool system

---

## Summary: Issues Found

### CRITICAL ISSUES (Must Fix)

1. **Point Visibility Bug** 
   - **Location**: `/src/rendering/points.rs:52-61`
   - **Issue**: Points hidden when no sort has `ActiveSort` component
   - **Impact**: Pan mode, preview mode show no points
   - **Severity**: HIGH
   - **Fix**: Remove or modify early return condition

2. **Gizmo Usage in Selection**
   - **Location**: `/src/rendering/selection.rs:25, 66-69, 94, 144-163, 170-179`
   - **Issue**: Selection marquee and feedback use Gizmos
   - **Impact**: Violates CLAUDE.md mesh-only rule (line 219)
   - **Severity**: MEDIUM
   - **Fix**: Convert to mesh-based rendering

### MEDIUM ISSUES (Should Fix)

3. **Plugin Registration Incomplete**
   - **Location**: `/src/core/app/plugins.rs`
   - **Issue**: Only Pen and Select plugins explicitly registered
   - **Impact**: Dynamic loading hides where plugins are added
   - **Severity**: MEDIUM
   - **Fix**: Add all tool plugins explicitly

4. **Duplicate Point Rendering Logic**
   - **Location**: `/src/rendering/points.rs` vs `/src/rendering/selection.rs`
   - **Issue**: Similar code in two places (inconsistent approaches)
   - **Impact**: Maintenance burden, inconsistency
   - **Severity**: LOW
   - **Fix**: Consolidate into single system

### MINOR ISSUES (Nice to Have)

5. **Stub Tools**
   - **Tools**: Hyper, Metaballs, AI
   - **Status**: Disabled in toolbar, frameworks exist but incomplete
   - **Severity**: LOW
   - **Fix**: Either implement or remove stubs

6. **Legacy Tool Resources**
   - **Issue**: Multiple "Active" resources per tool for sync compatibility
   - **Status**: Works but has synchronization overhead
   - **Severity**: LOW
   - **Fix**: Eventually consolidate to ToolState-only

---

## Cleanup/Refactoring Priority Plan

### Phase 1: Critical Fixes (Required)

**1A: Fix Point Visibility** `/src/rendering/points.rs:51-62`
- Remove early return that hides all points
- Always render points regardless of `ActiveSort`
- Only control point editability based on `ActiveSort`

**1B: Fix Gizmo Usage** `/src/rendering/selection.rs`
- Convert `render_selection_marquee()` to mesh-based
- Convert `render_selected_entities()` to mesh-based
- Convert helper function `draw_dashed_line()` to mesh-based

### Phase 2: Code Organization (Recommended)

**2A: Explicit Plugin Registration** `/src/core/app/plugins.rs`
- Add all tool plugins explicitly to EditorPluginGroup
- Makes plugin loading visible and maintainable

**2B: Consolidate Point Rendering**
- Keep `points.rs` for point entity rendering
- Keep `selection.rs` for selection marquee
- Remove duplicate `render_all_point_entities()` in selection.rs

### Phase 3: Complete Stub Tools (Optional)

**3A: Hyper Tool**
- Implement curve drawing features
- Or move to Pen tool submenu

**3B: Metaballs Tool**
- Implement organic shape creation
- Or defer if complex

**3C: AI Tool**
- Implement actual AI operations
- Or defer as future feature

---

## Architecture Strengths

1. ✅ **Clean Tool Separation** - Each tool in own file with clear structure
2. ✅ **Configuration-Based Toolbar** - Easy to reorder, enable/disable, modify
3. ✅ **Unified ToolState** - Single source of truth for active tool
4. ✅ **Proper Mesh Rendering** - Tools use meshes, not gizmos
5. ✅ **Clear Tool Traits** - `EditTool` trait provides consistent interface
6. ✅ **System Organization** - Tools register their own systems via plugins
7. ✅ **No Duplicate Tools** - Clear separation between tool logic and UI

---

## Architecture Weaknesses

1. ❌ **Point Visibility Bug** - Points hidden in pan/preview mode
2. ❌ **Gizmo Violation** - Selection rendering uses gizmos instead of meshes
3. ❌ **Hidden Plugin Loading** - Dynamic registration obscures where plugins load
4. ❌ **Duplicate Point Code** - Point rendering logic split between two files
5. ❌ **Incomplete Tools** - Hyper, Metaballs, AI are stubs
6. ❌ **Legacy Resources** - Multiple "Active" resources per tool (for sync compatibility)

---

## Recommendations

### For Immediate Action

1. **Fix point visibility** - Highest impact, prevents users from seeing points in pan mode
2. **Fix Gizmo usage** - Architecture compliance, prevents confusion between mesh and gizmo rendering

### For Next Sprint

3. **Explicit plugin registration** - Code clarity and maintainability
4. **Consolidate point rendering** - Reduce duplication

### For Future Consideration

5. **Implement or remove stub tools** - Clean up incomplete code
6. **Consolidate to ToolState** - Simplify tool state management

---

## File Summary

### Tools Module (/src/tools/) - 3,360 total lines
- **mod.rs**: 110 lines - Module documentation
- **tool_state.rs**: 226 lines - Unified tool state
- **select.rs**: 397 lines - Select tool wrapper
- **pen.rs**: 865 lines - Pen drawing tool ✅
- **knife.rs**: 341 lines - Cut/slice tool ✅
- **shapes.rs**: 348 lines - Shapes tool ✅
- **measure.rs**: 339 lines - Measurement tool ✅
- **text.rs**: 131 lines - Text placement tool ✅
- **pan.rs**: 41 lines - Pan tool ✅
- **hyper.rs**: 47 lines - STUB
- **metaballs.rs**: 47 lines - STUB
- **ai.rs**: 386 lines - STUB
- **tests.rs**: 82 lines - Unit tests

### UI Toolbar (/src/ui/edit_mode_toolbar/)
- **toolbar_config.rs**: 221 lines - Config source of truth ✅
- **config_loader.rs**: 120+ lines - Dynamic registration
- **ui.rs**: Toolbar UI rendering ✅
- **keyboard_shortcuts.rs**: Keyboard handling ✅

### Rendering (/src/rendering/)
- **points.rs**: 303 lines - Point rendering ⚠️ (ActiveSort bug)
- **selection.rs**: 250 lines - Selection rendering ❌ (Uses Gizmos)

### Editing (/src/editing/)
- **selection/**: 28 files - Complete selection system
- **sort/**: Sort/glyph management
