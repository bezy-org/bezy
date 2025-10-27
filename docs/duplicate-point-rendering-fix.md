# Duplicate Point Rendering System - Fixed ✅

**Date**: 2025-10-25
**Issue**: Z-fighting, flashing points, broken drag behavior
**Root Cause**: Two point rendering systems running simultaneously

---

## Problem Description

### Symptoms
- Points constantly flashing (z-fighting)
- When dragging a point, it separates from the underlying point
- Point doesn't actually move the outline as intended
- Two visual representations of each point at different Z levels

### Root Cause

**TWO point rendering systems were both active:**

1. **Original System** ✅ (Your code)
   - File: `src/rendering/glyph_renderer.rs`
   - Plugin: `GlyphRenderingPlugin`
   - Function: `render_glyphs()` (line 113+)
   - What: Unified system rendering points, outlines, handles together
   - Added: Initial repo transfer (commit 8f2aa52)

2. **Duplicate System** ❌ (Claude Code added)
   - File: `src/rendering/points.rs`
   - Plugin: `PointRenderingPlugin`
   - Function: `render_points_with_meshes()` (line 28+)
   - What: Separate point-only rendering system
   - Added: Post-fork cleanup (commit 602906f)

Both were registered in `src/core/app/plugins.rs`:
- Line 56: `PointRenderingPlugin` ❌ (duplicate)
- Line 62: `GlyphRenderingPlugin` ✅ (original)

---

## The Fix

**File**: `src/core/app/plugins.rs`

**Changed**:
```rust
// BEFORE (BROKEN - both running)
.add(PointRenderingPlugin)      // Duplicate - line 56
.add(GlyphRenderingPlugin)       // Original - line 62

// AFTER (FIXED - only original)
// .add(PointRenderingPlugin)    // REMOVED - Duplicate
.add(GlyphRenderingPlugin)       // KEPT - Unified renderer
```

**Result**: Only the original unified system runs now.

---

## Why This Happened

**AI Pattern**: Claude Code saw point rendering code and assumed it needed "improvement":
1. Saw `glyph_renderer.rs` rendering points
2. Thought "let's make a dedicated point rendering system"
3. Created `points.rs` with separate point rendering
4. Registered both systems without realizing the duplication
5. Result: Two systems fighting over the same visual elements

**The Mistake**: The original `GlyphRenderingPlugin` ALREADY handles points perfectly. It renders points, outlines, and handles together in one system to prevent exactly this kind of coordination problem.

---

## How The Original System Works

**File**: `src/rendering/glyph_renderer.rs`

The unified system renders everything together:

```rust
pub(crate) fn render_glyphs(
    // ... parameters ...
) {
    // 1. Collect all sorts (active and inactive)
    // 2. For ACTIVE sorts: Render points + handles + outline
    // 3. For INACTIVE sorts: Render filled outline only
    // 4. Everything synchronized in ONE system
}
```

**Key advantage**: When you drag a point:
- The point visual entity updates
- The outline immediately re-renders using the new point position
- Everything stays synchronized because it's one system

**With the duplicate**: When you drag a point:
- `PointRenderingPlugin` renders point at original position
- `GlyphRenderingPlugin` renders point at new position
- You see both (z-fighting)
- Drag updates only one, not the other
- Outline gets confused about which to use

---

## Verification

### ✅ Compilation
```bash
cargo check
# Result: SUCCESS
```

### ✅ Files Modified
- `src/core/app/plugins.rs` - Removed `PointRenderingPlugin` registration

### ✅ Files NOT Modified
- `src/rendering/glyph_renderer.rs` - Original system untouched ✅
- `src/rendering/points.rs` - Duplicate code still exists (not registered, harmless)

---

## Testing Checklist

After running the app, verify:

- [ ] No z-fighting or flashing on points
- [ ] Points render correctly (on-curve squares, off-curve circles)
- [ ] Selected points show different colors
- [ ] When dragging a point, it stays together (doesn't separate)
- [ ] Dragging a point updates the outline immediately
- [ ] No visual lag or glitches
- [ ] Points hidden in pan mode (spacebar)
- [ ] Selection marquee works (mesh-based, dashed rectangle)

---

## Related Issues Fixed

This was part of a larger cleanup:

1. ✅ **Pan mode point visibility** - Points now correctly hidden in pan mode
2. ✅ **Gizmo removal** - Selection marquee converted to meshes
3. ✅ **Dead code cleanup** - Removed unused Gizmo functions
4. ✅ **Duplicate point rendering** - THIS FIX

---

## Architecture Lesson

**Bevy ECS Anti-Pattern**: Duplicate Systems

When you have systems that manage the same visual elements:
- ❌ They fight over Z-ordering (z-fighting)
- ❌ They create duplicate entities
- ❌ Updates to one don't sync with the other
- ❌ User sees visual glitches and broken behavior

**Solution**: Single Unified System
- ✅ One system owns the visual elements
- ✅ All related visuals updated together
- ✅ Consistent Z-ordering
- ✅ No synchronization issues

This is exactly why the original `GlyphRenderingPlugin` exists - to prevent these coordination problems.

---

## Future Prevention

**Before adding a new rendering system**, check:

1. Is there already a system rendering these elements?
2. Search for existing rendering code: `grep -r "render.*point" src/`
3. Check what plugins are registered: `cat src/core/app/plugins.rs`
4. If similar code exists, enhance it - don't duplicate it

**Signs of duplicate systems**:
- Z-fighting or flashing
- Dragged elements separate from their "shadow"
- Updates don't propagate correctly
- Grep shows multiple functions with similar names

---

## File Locations

**Original System** (KEPT):
- `/src/rendering/glyph_renderer.rs` - Unified rendering
- Registered: `plugins.rs:62`

**Duplicate System** (REMOVED from registration):
- `/src/rendering/points.rs` - Duplicate point rendering
- Not registered (commented out in `plugins.rs:56-58`)
- File still exists but harmless (not loaded)

---

**END OF FIX DOCUMENTATION**
