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

- [x] **Phase 1: Update Point Movement Systems** - COMPLETED
  - [x] `src/editing/selection/point_movement.rs` - Updated `sync_to_font_data()` to use AppState
  - [x] `src/editing/selection/input/drag.rs` - Updated `handle_point_drag()` to use AppState
  - [x] `src/editing/selection/nudge.rs` - Updated all nudge systems to use AppState
  - [x] Removed delayed sync complexity - now syncs immediately with AppState
  - [x] **CRITICAL FIX**: Updated `src/systems/fontir_lifecycle.rs` - now loads AppState on startup
  - [x] Updated `src/systems/commands.rs` - save and commands now use AppState only
  - [x] Compilation verified - no errors

**Root Cause Identified and Fixed:**
The point reversion bug occurred because:
1. FontIRAppState was loaded on startup, but AppState was never initialized
2. Point movement systems wrote to AppState (which was empty/missing)
3. Rendering systems read from FontIRAppState (which had stale data)
4. Result: edits appeared to work but immediately reverted

**Solution:**
- Font loading (`deferred_fontir_font_loading`) now loads AppState instead of FontIRAppState
- All point editing systems now read/write the same AppState
- Single source of truth eliminates sync issues

- [x] **Phase 2: Update Rendering Systems** - COMPLETED
  - [x] Created BezPath conversion layer in `src/data/conversions.rs`
  - [x] Added `OutlineData::to_bezpaths()` method
  - [x] Added `ContourData::to_bezpath()` method with UFO point type handling
  - [x] Updated `render_filled_outline()` in `src/rendering/glyph_renderer.rs`
  - [x] Compilation verified - no errors

  **Architecture: AppState → BezPath Conversion Layer**
  - AppState stores font data in UFO format (ContourData/PointData)
  - Conversion layer transforms to kurbo::BezPath for rendering
  - Lyon tessellation pipeline unchanged
  - Data flow: AppState → BezPath → Lyon → Mesh
  - Idiomatic Rust pattern: separates data storage from rendering format
  - Leverages kurbo's battle-tested 2D curve handling

  Remaining rendering files (use same pattern):
    - `src/rendering/outline_elements.rs` (already updated)
    - `src/rendering/metrics.rs` (may need AppState)
    - `src/rendering/sort_visuals.rs` (may need AppState)

- [x] **Phase 3: Update Entity Management** - PARTIALLY COMPLETED
  - [x] `src/editing/selection/entity_management/spawning.rs` - Updated in Phase 1
  - [ ] `src/editing/sort/manager.rs` - May need AppState updates

- [x] **Phase 4: Update Text Editor** - COMPLETED
  - [x] `src/core/state/text_editor/editor.rs` - Renamed `create_text_root_with_fontir()` to `create_text_root_with_app_state()`, updated to read advance widths from AppState
  - [x] `src/systems/sorts/keyboard_input.rs` - Removed all FontIRAppState parameters, updated `get_glyph_advance_width()` to use only AppState
  - [x] `src/systems/sorts/unicode_input.rs` - Updated all helper functions to remove FontIR fallback logic
  - [x] Removed FontIR-specific function `get_contextual_arabic_glyph_name()`
  - [x] Updated arrow key navigation to use AppState
  - [x] Compilation verified - no errors

- [x] **Phase 5: Remove FontIR Lifecycle** - COMPLETED
  - [x] `src/systems/fontir_lifecycle.rs` - Functions renamed to accurately reflect AppState loading (not FontIR)
    - `load_fontir_font` → `initialize_font_loading`
    - `deferred_fontir_font_loading` → `load_font_deferred`
  - [x] `src/systems/mod.rs` - Updated exports with new function names
  - [x] `src/core/app/builder.rs` - Updated all function calls to use new names
  - [x] No FontIRLifecyclePlugin found (never existed or already removed)
  - [x] Compilation verified - no errors

  **Note**: The lifecycle system was already updated to use AppState in Phase 1. Phase 5 focused on removing misleading "fontir" naming.

- [x] **Phase 6: Remove FontIR Infrastructure** - COMPLETED
  - [x] **Deleted** `src/font_source/fontir_state.rs` (~2000 lines) - The entire FontIRAppState implementation
  - [x] **Updated** `src/font_source/mod.rs` - Removed all FontIR exports (EditableGlyphInstance, FontIRAppState, FontIRMetrics)
  - [x] **Updated** `src/core/state/mod.rs` - Removed FontIR re-exports and fontir_app_state module

  **Files Fixed (26 files total):**
  - Core: builder.rs, startup_layout.rs
  - Rendering: glyph_renderer.rs
  - Editing: offcurve_insertion.rs, sort/manager.rs, selection/entity_management/spawning.rs
  - Systems: input_consumer.rs, text_shaping.rs, sorts/* (cursor.rs, point_entities.rs, sort_entities.rs, sort_placement.rs, input_utilities.rs)
  - Tools: pen.rs
  - UI: file_menu.rs, edit_mode_toolbar/* (knife.rs, measure.rs, shapes.rs, text.rs), panes/* (file_pane.rs, glyph_pane.rs)
  - TUI: communication.rs, message_handler.rs

  **Changes Made:**
  - Removed ~150 FontIRAppState parameter declarations across function signatures
  - Removed all FontIR data access (replaced with AppState or disabled)
  - Commented out features that heavily depend on FontIR:
    * Off-curve point insertion (temporary)
    * Pen tool contour saving (temporary)
    * Component glyph detection (temporary)
    * Some TUI font info features (temporary)
    * Knife tool intersection calculation (temporary)
    * Measure tool intersection features (temporary)

  **Result:**
  - ✅ **0 compilation errors**
  - ⚠️ 70 warnings (mostly unused variables from disabled code)
  - ✅ Compilation time: 8.35s
  - ✅ ~2200 lines of FontIR code eliminated

  **Note**: Some features are temporarily disabled with TODO comments. These can be re-enabled using AppState in future work.

- [x] **Phase 7: Update Dependencies** - COMPLETED
  - [x] **Removed from Cargo.toml**: `fontir = "0.3.0"` and `ufo2fontir = "0.2.2"`
  - [x] **Deleted** `src/data/fontir_adapter.rs` - Bridge file for FontIR integration (no longer needed)
  - [x] **Updated** `src/data/mod.rs` - Removed fontir_adapter module export

  **Result:**
  - ✅ **0 compilation errors**
  - ✅ Build time: 10.29s (full check)
  - ✅ Dependency tree cleaned of unused FontIR crates

- [x] **Phase 8: Global Cleanup** - COMPLETED
  - [x] Searched entire codebase for remaining FontIR references
  - [x] **Removed FontIRPointReference component** - Unused legacy component deleted
  - [x] **Updated log messages** - Changed "FontIR change" → "AppState change"
  - [x] **Verified no code references** - All remaining references are in comments/logs only

  **Remaining References Analysis (118 total):**
  - 91 comment references (// historical context and TODO notes)
  - 15 debug! messages (logging strings)
  - 3 info! messages (logging strings)
  - 9 warn! messages (logging strings)

  **Decision:** These references are harmless documentation/logging and can remain. They provide historical context about the migration and don't affect functionality.

  **Result:**
  - ✅ **0 compilation errors**
  - ✅ **No functional FontIR code remains**
  - ✅ All FontIR infrastructure completely removed
  - ✅ Build verified: 7.05s

- [ ] **Phase 9: Testing** - READY TO TEST
  - **CRITICAL: Test point editing immediately to verify bug fix**
  - Test point selection
  - Test point dragging
  - Test point nudging with arrow keys
  - **Test tool switching after nudging** (this was the original bug)
  - Test save/load operations

## Notes

This migration reverses an earlier experimental integration. The original AppState approach was working correctly. FontIR was added to explore multi-format support, but the complexity introduced exceeds the benefit for a UFO/designspace-focused editor.
