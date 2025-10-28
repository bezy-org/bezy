# FontIR Removal Migration Plan

## Background

FontIR was an experimental integration with Google Fonts toolchain (`fontir` and `ufo2fontir` crates). After extensive testing, it has proven to cause more problems than it solves:

**Issues Discovered:**
- Caching synchronization bugs causing point reversion after edits
- Complex dual-layer architecture: `glyph_cache` (immutable) + `working_copies` (mutable)
- `working_copies` never successfully created during normal editing operations
- Performance overhead from BezPath rebuilding and cache management
- Read-only architecture requiring parallel save implementation
- No clear benefit - multi-format support not needed (only UFO and designspace required)

**Root Cause of Point Reversion Bug:**
- Transform updates (visual) happen immediately
- FontIR sync happens on delayed events (key release, tool switch)
- Tool switches don't trigger sync events as expected
- Result: visual changes not persisted, data reverts on next render cycle

## Goal

Revert to simple AppState-only architecture with direct UFO/designspace manipulation via `norad` library. This provides:
- Single source of truth for font data
- Direct UFO read/write without caching layers
- Simpler, more maintainable code
- Better performance (no cache rebuilding)
- Fixes point reversion bug naturally

## Current State Analysis

**FontIRAppState Usage (to be removed):**
- Used in ~39 files across the codebase
- Primary locations:
  - `src/editing/selection/point_movement.rs` - sync point edits to data
  - `src/editing/selection/input/drag.rs` - drag point updates
  - `src/editing/selection/nudge.rs` - nudge point updates
  - `src/systems/fontir_lifecycle.rs` - FontIR initialization and management
  - `src/font_source/fontir_state.rs` - FontIR wrapper (2000+ lines)

**AppState (to be restored/enhanced):**
- Simple HashMap-based structure: `HashMap<String, GlyphData>`
- Direct norad integration for UFO read/write
- Already has working save functionality
- Located in `src/core/state/mod.rs`

## Migration Steps

### Phase 1: Update Point Movement Systems (High Priority)
These systems directly cause the point reversion bug:

1. **`src/editing/selection/point_movement.rs`**
   - Change `sync_to_font_data` signature: remove `FontIRAppState`, add back `AppState`
   - Update logic to write directly to AppState HashMap
   - Remove FontIR-specific code paths

2. **`src/editing/selection/input/drag.rs`**
   - Update `handle_point_drag` to use AppState instead of FontIRAppState
   - Immediate sync to AppState (no delayed batching needed)

3. **`src/editing/selection/nudge.rs`**
   - Update nudge systems to use AppState
   - Remove delayed sync complexity - can sync immediately with AppState
   - Remove `sync_before_tool_switch` system (no longer needed)

### Phase 2: Update Rendering Systems
4. **`src/rendering/glyph_renderer.rs`**
   - Read from AppState instead of FontIRAppState
   - Simpler data access pattern (direct HashMap lookup)

5. **`src/rendering/outline_renderer.rs`**
   - Update glyph outline rendering to use AppState

### Phase 3: Update Entity Management
6. **`src/editing/selection/entity_management/sync.rs`**
   - Re-enable and update `sync_point_positions_to_sort` to use AppState
   - Ensure it reads from correct data source

7. **`src/editing/selection/systems.rs`**
   - Update all selection systems to use AppState

### Phase 4: Remove FontIR Lifecycle
8. **`src/systems/fontir_lifecycle.rs`**
   - Remove entire file (background loading, FontIR initialization)

9. **`src/core/app/plugins.rs`**
   - Remove FontIRLifecyclePlugin registration

### Phase 5: Update Save/Load Operations
10. **`src/systems/commands.rs`**
    - Remove FontIRAppState save path
    - Keep only AppState save functionality

11. **`src/core/app/builder.rs`**
    - Remove FontIR initialization
    - Restore AppState initialization from UFO/designspace files

### Phase 6: Remove FontIR Infrastructure
12. **`src/font_source/fontir_state.rs`**
    - Remove entire file (2000+ lines of FontIR wrapper code)

13. **`src/font_source/mod.rs`**
    - Remove FontIRAppState exports

14. **`src/core/state/mod.rs`**
    - Remove FontIRAppState resource
    - Ensure AppState is properly exported

### Phase 7: Update Dependencies
15. **`Cargo.toml`**
    - Remove `fontir` dependency
    - Remove `ufo2fontir` dependency
    - Keep `norad` (direct UFO manipulation)

### Phase 8: Search and Replace
16. **Global cleanup:**
    - Search for all `FontIRAppState` usages: `grep -r "FontIRAppState" src/`
    - Replace with `AppState` where needed
    - Remove where not needed

### Phase 9: Testing
17. **Functionality testing:**
    - Test point selection
    - Test point dragging
    - Test point nudging with arrow keys
    - Test tool switching after edits
    - Test save/load operations
    - Verify no point reversion occurs

## Expected Benefits

1. **Bug Fix:** Point reversion bug will be fixed naturally
2. **Performance:** No cache rebuilding overhead
3. **Simplicity:** Single data source, easier to understand and maintain
4. **Maintainability:** ~2000 fewer lines of wrapper code
5. **Reliability:** Direct data access, no sync timing issues

## Risks and Mitigations

**Risk:** Breaking existing functionality during migration
**Mitigation:** Phase-by-phase approach, testing after each phase

**Risk:** AppState may need enhancements for features that relied on FontIR
**Mitigation:** AppState already has most needed functionality, add incrementally if needed

**Risk:** Designspace support complexity
**Mitigation:** Keep designspace handling simple - edit individual master UFOs directly

## Status Tracking

- [ ] Phase 1: Update Point Movement Systems
- [ ] Phase 2: Update Rendering Systems
- [ ] Phase 3: Update Entity Management
- [ ] Phase 4: Remove FontIR Lifecycle
- [ ] Phase 5: Update Save/Load Operations
- [ ] Phase 6: Remove FontIR Infrastructure
- [ ] Phase 7: Update Dependencies
- [ ] Phase 8: Global Cleanup
- [ ] Phase 9: Testing

## Notes

This migration reverses an earlier experimental integration. The original AppState approach was working correctly. FontIR was added to explore multi-format support, but the complexity introduced exceeds the benefit for a UFO/designspace-focused editor.
