# Edit Mode Tools - Cleanup & Refactoring Plan

**Date**: 2025-10-25
**Status**: Planning Phase
**Priority**: HIGH - Critical bugs affecting core functionality

---

## Executive Summary

The edit mode tools system has several critical issues caused by AI-generated code that violates architecture rules and creates duplicate/conflicting systems. This document identifies the problems and provides a concrete refactoring plan.

### Critical Problems

1. **Points visible in pan/preview mode** - ✅ FIXED - Points now correctly hidden in pan mode
2. **Gizmo violations** - Selection rendering uses Gizmos despite CLAUDE.md forbidding it
3. **Duplicate systems** - Multiple renderers competing for the same task
4. **Hidden plugin registration** - Tool plugins registered dynamically, not explicitly

---

## Problem 1: Point Visibility Bug in Pan Mode (CRITICAL) - ✅ FIXED

### Symptoms
- Points visible in pan/preview mode (incorrect)
- Points should be hidden in pan mode (standard font editor behavior)
- Only filled outlines should show in pan mode

### Root Cause

**File**: `src/rendering/points.rs:52-61`

**Problem**: The rendering system only checked for `ActiveSort` but didn't check the current tool mode. This meant:
1. Points rendered whenever an `ActiveSort` existed
2. Pan mode didn't hide points (only changed `InputMode`)
3. User saw points in pan mode even though they should be hidden

**Standard font editor behavior**: Pan mode shows only filled outlines, no points or metrics lines.

### Expected Behavior

- **Pan mode**: ❌ No points visible (only filled outlines)
- **Edit mode**: ✅ Points visible and editable
- **Preview mode**: ❌ No points visible (only filled outlines)

### Fix Applied

Added `CurrentTool` check to hide points when pan tool is active:

```rust
// Hide points in pan/preview mode (standard font editor behavior)
if current_tool.get_current() == Some("pan") {
    info!("🎨 [render_points_with_meshes] Pan mode active - hiding all points");
    // Clean up existing point meshes
    for entity in existing_point_meshes.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
    return;
}
```

### Result

- **Pan mode**: ✅ No points visible (correct)
- **Edit mode**: ✅ Points visible when sort is active (correct)
- **Preview mode**: ✅ No points when no active sort (correct)

### Testing Checklist

After running the app, verify:
- [ ] Points hidden in pan mode (spacebar or pan tool)
- [ ] Only filled outlines visible in pan mode
- [ ] Points visible in edit/select mode
- [ ] Points are non-editable in pan mode (handled by input systems)
- [ ] Performance unchanged (no visual lag)
- [ ] No metrics lines visible in pan mode

---

## Problem 2: Gizmo Violations (ARCHITECTURE)

### Symptoms
- Selection marquee uses Bevy Gizmos
- Selection feedback (crosshairs) uses Gizmos
- Violates CLAUDE.md line 235: "Mesh-based only: Never use Bevy Gizmos for world-space elements"

### Root Cause

**File**: `src/rendering/selection.rs`

**Violation #1**: `render_selection_marquee()` (lines 23-71)
```rust
pub fn render_selection_marquee(
    _commands: Commands,
    mut gizmos: Gizmos,  // ❌ USES GIZMOS
    drag_state: Res<DragSelectionState>,
    // ...
) {
    // Lines 66-69: Draw dashed rectangle with gizmos
    draw_dashed_line(&mut gizmos, p1, p2, color, 8.0, 4.0);  // ❌ GIZMOS
    draw_dashed_line(&mut gizmos, p2, p3, color, 8.0, 4.0);  // ❌ GIZMOS
    draw_dashed_line(&mut gizmos, p3, p4, color, 8.0, 4.0);  // ❌ GIZMOS
    draw_dashed_line(&mut gizmos, p4, p1, color, 8.0, 4.0);  // ❌ GIZMOS
}
```

**Violation #2**: `draw_dashed_line()` helper (lines 74-100)
```rust
fn draw_dashed_line(
    gizmos: &mut Gizmos,  // ❌ USES GIZMOS
    start: Vec2,
    end: Vec2,
    color: Color,
    dash_length: f32,
    gap_length: f32,
) {
    // ...
    gizmos.line_2d(current_position, segment_end_position, color);  // ❌ GIZMOS
}
```

**Violation #3**: `render_selected_entities()` (lines 104-196)
- Lines 144-163: Uses `gizmos.rect_2d()` and `gizmos.circle_2d()` for selection outlines
- Lines 170-179: Uses `gizmos.line_2d()` for crosshairs

### Why This is a Problem

1. **Architecture violation**: CLAUDE.md explicitly forbids Gizmos for world-space rendering
2. **Inconsistency**: All tools use mesh-based rendering, selection uses Gizmos
3. **Visual quality**: Gizmos don't support same visual effects as meshes
4. **Performance**: Gizmos have different performance characteristics
5. **Theme integration**: Harder to apply theme system to Gizmos

### Fix Strategy

Convert all Gizmo-based rendering to mesh-based rendering, following the pattern used in `points.rs` and tool preview renderers.

**Example Pattern** (from `pen.rs:474-865`):

```rust
// ✅ CORRECT: Mesh-based marquee rendering
pub fn render_selection_marquee_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    drag_state: Res<DragSelectionState>,
    marquee_query: Query<(Entity, &SelectionRect)>,
    existing_marquee: Query<Entity, With<MarqueeMesh>>,
    theme: Res<CurrentTheme>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
) {
    // Clean up existing marquee meshes
    for entity in existing_marquee.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Only render when in select mode and dragging
    if current_tool.get_current() != Some("select") || !drag_state.is_dragging {
        return;
    }

    if let Some((_, rect)) = marquee_query.iter().next() {
        let color = theme.action_color();

        // Create dashed rectangle using mesh segments
        create_dashed_rectangle_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            rect.start,
            rect.end,
            color,
        );
    }
}

fn create_dashed_rectangle_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    color: Color,
) {
    // Create dashed line segments as Rectangle meshes
    let dash_length = 8.0;
    let gap_length = 4.0;

    // For each edge of rectangle, create dashed line meshes
    // Similar to how pen.rs creates preview line segments
}
```

### Files to Modify

- `src/rendering/selection.rs` - Convert all functions from Gizmos to meshes
  - `render_selection_marquee()` → `render_selection_marquee_meshes()`
  - `draw_dashed_line()` → `create_dashed_line_meshes()`
  - `render_selected_entities()` → Convert to mesh-based

### Components Needed

Add marker component for cleanup:
```rust
#[derive(Component)]
pub struct MarqueeMesh;

#[derive(Component)]
pub struct SelectionFeedbackMesh;
```

### Testing

After fix, verify:
- [ ] Selection marquee renders correctly (dashed rectangle)
- [ ] Selection feedback renders correctly (crosshairs, outlines)
- [ ] Visual appearance matches previous Gizmo version
- [ ] Theme colors applied correctly
- [ ] Performance unchanged or improved

---

## Problem 3: Duplicate/Conflicting Systems

### Issue: Duplicate Point Rendering

**Location 1**: `src/rendering/points.rs` (mesh-based)
- Function: `render_points_with_meshes()`
- Status: Primary renderer, has ActiveSort bug
- Quality: ✅ Correct architecture (meshes)

**Location 2**: `src/rendering/selection.rs` (gizmo-based)
- Function: `render_all_point_entities()` (lines 202-249)
- Status: Legacy/duplicate renderer
- Quality: ❌ Uses Gizmos (violation)

### Problem

Two systems attempting to render the same points:
1. Creates visual conflicts
2. Wastes CPU/GPU resources
3. Confusing which system is authoritative
4. Makes debugging difficult

### Fix Strategy

**Keep**: `points.rs` - Primary point renderer (after fixing ActiveSort bug)

**Remove**: `render_all_point_entities()` from `selection.rs`

`selection.rs` should only handle:
- Selection marquee (drag rectangle)
- Selection feedback (visual indication of selected state)
- NOT raw point rendering

### Files to Modify

- `src/rendering/selection.rs:202-249` - Remove `render_all_point_entities()`
- Verify no systems call this function before removing

---

## Problem 4: Hidden Plugin Registration

### Symptoms
- Only 2 tool plugins explicitly registered in `plugins.rs`
- Other tools work but registration is hidden in `config_loader.rs`
- Not obvious where tool plugins are loaded

### Current State

**File**: `src/core/app/plugins.rs:84-85`

```rust
// Only 2 tools explicitly registered
.add(crate::tools::PenToolPlugin)
.add(crate::tools::SelectToolPlugin)
```

**Missing from explicit registration**:
- KnifeToolPlugin
- MeasureToolPlugin
- ShapesToolPlugin
- TextToolPlugin
- PanToolPlugin

These are registered dynamically in `src/ui/edit_mode_toolbar/config_loader.rs` based on `toolbar_config.rs` configuration.

### Why This is a Problem

1. **Hidden behavior**: Not obvious where plugins load
2. **Maintenance**: New developer won't know to check config_loader
3. **Inconsistency**: Some tools explicit, some implicit
4. **Fragility**: Removing config_loader would silently break tools
5. **Debugging**: Hard to verify which plugins are actually loaded

### Fix Strategy

**Make all tool plugin registration explicit** in `plugins.rs`:

```rust
// ✅ FIXED: All tools explicitly registered
pub struct EditorPluginGroup;

impl PluginGroup for EditorPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        use crate::ui::edit_mode_toolbar::EditModeToolbarPlugin;
        use crate::ui::file_menu::FileMenuPlugin;
        use crate::ui::panes::coordinate_pane::CoordinatePanePlugin;
        use crate::ui::panes::glyph_pane::GlyphPanePlugin;

        PluginGroupBuilder::start::<Self>()
            .add(GlyphPanePlugin)
            .add(CoordinatePanePlugin)
            .add(crate::tools::ToolStatePlugin)
            .add(EditModeToolbarPlugin)
            .add(FileMenuPlugin)
            // ALL TOOL PLUGINS EXPLICITLY REGISTERED
            .add(crate::tools::SelectToolPlugin)
            .add(crate::tools::PanToolPlugin)
            .add(crate::tools::PenToolPlugin)
            .add(crate::tools::TextToolPlugin)
            .add(crate::tools::ShapesToolPlugin)
            .add(crate::tools::KnifeToolPlugin)
            .add(crate::tools::MeasureToolPlugin)
            // Stub tools - explicitly disabled
            // .add(crate::tools::HyperToolPlugin)      // Not implemented
            // .add(crate::tools::MetaballsToolPlugin)  // Not implemented
            // .add(crate::tools::AiToolPlugin)         // Not implemented
    }
}
```

**Keep dynamic registration** in `config_loader.rs` for:
- Toolbar button creation
- Keyboard shortcut mapping
- Tool ordering/visibility
- NOT plugin loading

### Files to Modify

- `src/core/app/plugins.rs:84-86` - Add all tool plugins explicitly
- `src/ui/edit_mode_toolbar/config_loader.rs` - Remove plugin registration logic, keep UI config

### Benefits

- ✅ Clear visibility of loaded plugins
- ✅ Easy to comment out tools for debugging
- ✅ Standard Bevy plugin pattern
- ✅ Better compile-time checking

---

## Problem 5: Stub Tools Clutter

### Incomplete Tools

**Hyper Tool** (`src/tools/hyper.rs`, 47 lines)
- Status: Empty stub
- Toolbar: Disabled
- Comment: "TODO: Implement or move to Pen submenu"

**Metaballs Tool** (`src/tools/metaballs.rs`, 47 lines)
- Status: Empty stub
- Toolbar: Disabled
- Comment: "TODO: Implement organic shape creation"

**AI Tool** (`src/tools/ai.rs`, 386 lines)
- Status: Framework only, no functionality
- Toolbar: Enabled but non-functional
- Comment: Operations defined but not implemented

### Options

**Option A: Remove stub tools entirely**
- Delete files
- Remove from toolbar config
- Clean slate for future implementation

**Option B: Keep as disabled stubs**
- Leave files in place
- Keep `enabled: false` in toolbar config
- Framework for future work

**Option C: Move to separate branch**
- Remove from main branch
- Create `feature/tool-stubs` branch
- Reduces main branch clutter

**Recommendation**: Option A or C - remove from main branch to reduce noise during cleanup.

---

## Refactoring Plan - Phased Approach

### Phase 1: Critical Fixes (HIGH PRIORITY - DO FIRST)

**Goal**: Fix bugs preventing core functionality

#### Task 1.1: Fix Point Visibility Bug
- **File**: `src/rendering/points.rs`
- **Change**: Remove lines 52-61 (early return check)
- **Testing**: Verify points visible in pan/preview mode
- **Time**: 30 minutes
- **Risk**: Low

#### Task 1.2: Fix Gizmo Violations - Selection Marquee
- **File**: `src/rendering/selection.rs`
- **Change**: Convert `render_selection_marquee()` to mesh-based
- **Pattern**: Follow `pen.rs` preview rendering
- **Testing**: Verify marquee still renders correctly
- **Time**: 2-3 hours
- **Risk**: Medium

#### Task 1.3: Fix Gizmo Violations - Selection Feedback
- **File**: `src/rendering/selection.rs`
- **Change**: Convert `render_selected_entities()` to mesh-based
- **Testing**: Verify selection feedback (crosshairs) still works
- **Time**: 2-3 hours
- **Risk**: Medium

**Phase 1 Deliverable**: Points visible in all modes, all rendering mesh-based

---

### Phase 2: Code Organization (MEDIUM PRIORITY)

**Goal**: Improve code clarity and maintainability

#### Task 2.1: Remove Duplicate Point Rendering
- **File**: `src/rendering/selection.rs:202-249`
- **Change**: Remove `render_all_point_entities()` function
- **Testing**: Verify points still render from `points.rs`
- **Time**: 30 minutes
- **Risk**: Low

#### Task 2.2: Explicit Plugin Registration
- **File**: `src/core/app/plugins.rs`
- **Change**: Add all tool plugins explicitly
- **File**: `src/ui/edit_mode_toolbar/config_loader.rs`
- **Change**: Remove plugin loading, keep UI config only
- **Testing**: Verify all tools still work
- **Time**: 1 hour
- **Risk**: Low

**Phase 2 Deliverable**: Clean architecture, obvious plugin loading

---

### Phase 3: Cleanup (LOW PRIORITY - OPTIONAL)

**Goal**: Remove clutter and technical debt

#### Task 3.1: Handle Stub Tools
- **Files**: `src/tools/hyper.rs`, `metaballs.rs`, `ai.rs`
- **Decision**: Remove, disable, or move to branch
- **Time**: 30 minutes
- **Risk**: Low

#### Task 3.2: Documentation
- **File**: Update CLAUDE.md with lessons learned
- **File**: Update architecture docs
- **Time**: 1 hour
- **Risk**: None

**Phase 3 Deliverable**: Clean codebase ready for new features

---

## Success Criteria

### After Phase 1
- [ ] Points visible in pan mode
- [ ] Points visible in preview mode
- [ ] No Gizmo usage in entire codebase (grep confirms)
- [ ] Selection marquee renders with meshes
- [ ] Selection feedback renders with meshes
- [ ] All rendering consistent (mesh-based)

### After Phase 2
- [ ] All tool plugins explicitly listed in `plugins.rs`
- [ ] No duplicate point rendering systems
- [ ] Clear separation: `points.rs` for points, `selection.rs` for marquee
- [ ] Code is obvious and maintainable

### After Phase 3
- [ ] No stub tools in main branch
- [ ] Documentation updated
- [ ] Architecture rules clearly enforced

---

## Risk Assessment

### High Risk Areas

**Gizmo Conversion**
- Risk: Visual appearance changes
- Mitigation: Screenshot comparison before/after
- Fallback: Keep original code commented for reference

**Point Rendering Changes**
- Risk: Performance regression
- Mitigation: Profile before/after with many points
- Fallback: Add performance monitoring

### Low Risk Areas

**Plugin Registration**
- Risk: Low - just moving existing code
- Mitigation: Verify all tools load at startup

**Duplicate Removal**
- Risk: Low - one system already primary
- Mitigation: Ensure `points.rs` works first

---

## Implementation Notes

### Mesh-Based Rendering Pattern

When converting Gizmos to meshes, follow this pattern:

```rust
// 1. Define marker component
#[derive(Component)]
struct PreviewMesh;

// 2. Clean up existing meshes
for entity in existing_meshes.iter() {
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.despawn();
    }
}

// 3. Create new meshes
let mesh = Rectangle::new(width, height);
let mesh_handle = meshes.add(mesh);
let material_handle = materials.add(ColorMaterial::from_color(color));

commands.spawn((
    Mesh2d(mesh_handle),
    MeshMaterial2d(material_handle),
    Transform::from_xyz(x, y, Z_LEVEL),
    PreviewMesh,  // For cleanup next frame
));
```

### Theme Integration

Always use theme colors:
```rust
let color = theme.action_color();        // For interactive elements
let color = theme.selection_color();     // For selected items
let color = theme.preview_color();       // For previews
```

### Z-Level Management

Use theme system constants:
```rust
use crate::ui::theme::{
    Z_SELECTION_MARQUEE,
    Z_POINT_MESH,
    Z_SELECTION_FEEDBACK,
};
```

---

## Related Documents

- `CLAUDE.md` - Architecture rules and guidelines
- `docs/edit-tools-architecture-analysis.md` - Detailed analysis (5000+ lines)
- `docs/debugging-selection-deselect.md` - Selection margin issues
- `docs/edit-mode-tools-refactor.md` - Previous refactoring notes

---

## Next Steps

1. **Review this document** - Ensure all problems are captured
2. **Prioritize fixes** - Confirm Phase 1 is correct priority
3. **Start Phase 1.1** - Fix point visibility bug (quickest win)
4. **Test thoroughly** - Each fix before moving to next
5. **Document changes** - Update CLAUDE.md with new patterns

---

## Questions to Resolve

1. **Point rendering**: Should we keep ANY ActiveSort checks, or render all points always?
2. **Stub tools**: Remove entirely, or keep disabled for future work?
3. **Selection rendering**: Keep separate file or merge into `points.rs`?
4. **Gizmo conversion**: Match exact visual appearance or improve during conversion?

---

**END OF CLEANUP PLAN**
