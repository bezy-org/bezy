# Gizmo Code Investigation - What's Actually Running

**Date**: 2025-10-25
**Investigation**: Selection rendering systems audit

---

## TL;DR

**GOOD NEWS**: Most of the Gizmo code is already dead/unused!

- ✅ Selected points rendering: Already mesh-based in `points.rs`
- ❌ Selection marquee: Using Gizmos (ACTIVE - needs fix)
- ✅ `render_selected_entities`: Dead code (NOT registered)
- ✅ `render_all_point_entities`: Dead code (NOT registered)

---

## What's Actually Registered

**File**: `src/editing/selection/mod.rs:134`

```rust
.add_systems(
    PostUpdate,
    (
        crate::rendering::selection::render_selection_marquee,  // ❌ USES GIZMOS
        utils::debug_print_selection_rects,
    )
        .in_set(FontEditorSets::Rendering),
)
```

**Only ONE Gizmo system is actually running**: `render_selection_marquee()`

---

## Dead Code (Not Registered Anywhere)

These functions exist in `src/rendering/selection.rs` but are **never called**:

1. **`render_selected_entities()`** (lines 104-196)
   - Uses Gizmos to draw crosshairs on selected points
   - NOT registered as a system
   - Can be deleted entirely

2. **`render_all_point_entities()`** (lines 202-249)
   - Uses Gizmos to render all point entities
   - NOT registered as a system
   - Can be deleted entirely

---

## How Selection Actually Works (Current State)

### 1. Selected Points Rendering ✅ CORRECT (Mesh-based)

**File**: `src/rendering/points.rs:101-116`

```rust
let (outline_color, middle_color) = if selected.is_some() {
    (
        theme.theme().selected_secondary_color(), // darker color
        theme.theme().selected_primary_color(),   // lighter color
    )
} else if point_type.is_on_curve {
    // ... normal point colors
}
```

**Status**: ✅ Already using meshes, already working correctly!

### 2. Selection Marquee (Drag Rectangle) ❌ NEEDS FIX (Gizmos)

**File**: `src/rendering/selection.rs:23-71`

**Current Implementation**: Gizmo-based dashed rectangle

```rust
pub fn render_selection_marquee(
    _commands: Commands,
    mut gizmos: Gizmos,  // ❌ USES GIZMOS
    drag_state: Res<DragSelectionState>,
    marquee_query: Query<(Entity, &SelectionRect)>,
    theme: Res<CurrentTheme>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
) {
    // ...
    draw_dashed_line(&mut gizmos, p1, p2, color, 8.0, 4.0);  // ❌ GIZMOS
}
```

**Status**: ❌ Active system using Gizmos - this is the ONLY one that needs fixing

---

## The Fix is Much Simpler Than Expected

We only need to:

1. **Delete dead code** from `src/rendering/selection.rs`:
   - Remove `render_selected_entities()` (lines 104-196)
   - Remove `render_all_point_entities()` (lines 198-249)

2. **Convert marquee to meshes**:
   - Replace `render_selection_marquee()` with mesh-based version
   - Keep it registered in the same place (mod.rs:134)

3. **Selected points**: Already working correctly, no changes needed!

---

## Why This Happened

Looking at git history:

1. `src/rendering/selection.rs` was added in initial repo transfer (commit 8f2aa52)
2. It had all three Gizmo functions from the start
3. But only `render_selection_marquee` was ever actually registered
4. The other two were dead code from day one

**Likely scenario**: AI generated a comprehensive "selection rendering" module but the user only hooked up the marquee, and the mesh-based point rendering was already working in `points.rs`.

---

## Cleanup Plan

### Phase 1: Delete Dead Code (5 minutes)

Remove from `src/rendering/selection.rs`:
- Lines 104-196: `render_selected_entities()`
- Lines 198-249: `render_all_point_entities()`
- Lines 74-100: `draw_dashed_line()` helper (used by marquee, will be replaced)

### Phase 2: Convert Marquee to Meshes (30-60 minutes)

Replace `render_selection_marquee()` with mesh-based implementation:

**Pattern to follow** (from metrics.rs or pen.rs):
- Use `Rectangle` meshes for line segments
- Calculate dashed pattern positions
- Spawn mesh entities with `MarqueeMesh` marker component
- Despawn old meshes each frame before rendering new ones

**Key references**:
- `src/rendering/metrics.rs:1220+` - Has dashed line mesh rendering
- `src/tools/pen.rs:474-865` - Preview rendering pattern
- `src/rendering/points.rs` - Mesh cleanup pattern

### Phase 3: Update Registration (Already done!)

The system is already registered at `src/editing/selection/mod.rs:134` - just keep the same registration, same function name, different implementation.

---

## File Size Reduction

**Before**: `src/rendering/selection.rs` = 250 lines
**After**: `src/rendering/selection.rs` = ~120 lines (52% smaller!)

Remove:
- `render_selected_entities()`: 93 lines
- `render_all_point_entities()`: 48 lines
- Dead imports and unused code: ~10 lines

**Total removal**: ~150 lines of dead Gizmo code

---

## Testing Checklist

After cleanup:

- [ ] Selection marquee still renders during drag selection
- [ ] Marquee is dashed rectangle (not solid)
- [ ] Marquee uses theme.action_color()
- [ ] Selected points still render with different colors
- [ ] No Gizmos anywhere (run: `grep -r "Gizmos" src/rendering`)
- [ ] Code compiles without warnings
- [ ] Performance unchanged

---

## Recommendation

**Start with Phase 1**: Just delete the dead code first. This is risk-free since those functions aren't called.

Then we can tackle the marquee conversion with a clean slate and half the code to worry about.

---

**END OF FINDINGS**
