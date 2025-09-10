# Unified UFO/Designspace Loading

## Implementation Status: ✅ COMPLETED

## Current State Analysis

### Current CLI Implementation
The app currently uses `--load-ufo` flag that accepts both UFO directories and `.designspace` files:
- Located in: `src/core/cli.rs`
- The validation logic already handles both formats
- However, the flag name `--load-ufo` is misleading since it also accepts designspace files

### Loading Process
1. **CLI Parsing** (`src/core/cli.rs`):
   - Validates path exists
   - Checks for `.designspace` extension or UFO directory structure
   
2. **FontIR Loading** (`src/systems/fontir_lifecycle.rs`):
   - Uses `FontIRAppState::from_path()` which auto-detects format
   - `DesignSpaceIrSource` handles both UFO and designspace transparently

3. **File Pane Display** (`src/ui/panes/file_pane.rs`):
   - Always shows master selector circles
   - Displays designspace path and current UFO info
   - Master switching functionality built-in

## Implemented Changes

### 1. CLI Flag Renamed ✅
**Completed**: Renamed `--load-ufo` to `--edit` with better naming

```rust
// src/core/cli.rs
#[clap(
    long = "edit",
    short = 'e',
    default_value = DEFAULT_UFO_PATH,
    help = "Font source to edit (UFO or designspace)",
)]
pub font_source: Option<PathBuf>,  // Better naming: font_source instead of font_path
```

### 2. Source Type Detection ✅
**Completed**: Added explicit source type detection

```rust
// src/core/state/fontir_app_state.rs
pub enum SourceType {
    SingleUfo,
    Designspace { master_count: usize },
}

impl FontIRAppState {
    pub fn is_single_ufo(&self) -> bool {
        matches!(self.source_type, SourceType::SingleUfo)
    }
}
```

The source type is now automatically detected when loading based on file extension.

### 3. File Pane Conditional Display ✅
**Completed**: File pane now shows/hides master selector based on source type

```rust
// src/ui/panes/file_pane.rs

// In update_master_buttons:
let should_show_masters = fontir_state
    .as_ref()
    .map(|state| !state.is_single_ufo())
    .unwrap_or(true);

if !should_show_masters {
    container_node.display = Display::None;  // Hide circles for single UFO
} else {
    container_node.display = Display::Flex;  // Show circles for designspace
}
```

## What Was Implemented

1. **CLI Update**: 
   - Replaced `--load-ufo` with `--edit` flag
   - Used better naming: `font_source` instead of `font_path` or `ufo_path`
   - No backwards compatibility needed (no users yet)

2. **Source Detection**:
   - Added `SourceType` enum to track if source is SingleUfo or Designspace
   - Auto-detection based on file extension (.ufo vs .designspace)
   - Added `is_single_ufo()` helper method

3. **File Pane Updates**:
   - Master selector circles now hidden for single UFO files
   - Master selector circles shown for designspace files
   - Clean, appropriate UI for each source type

## Benefits

1. **Simpler UX**: One flag (`--edit`) for all font formats
2. **Cleaner Interface**: No unnecessary UI elements for single UFOs
3. **Future-proof**: Easy to add more formats later
4. **Intuitive**: Users don't need to know format details

## Example Usage

```bash
# Edit a single UFO
bezy --edit MyFont.ufo

# Edit a designspace (multiple masters)
bezy --edit MyVariable.designspace

# Short form
bezy -e MyFont.ufo
```

## Technical Considerations

### FontIR Integration
- `DesignSpaceIrSource` already handles both formats
- No changes needed to core loading logic
- Just need UI/UX improvements

### Clean Implementation
- No backwards compatibility needed (no existing users)
- Clean, simple API with single `--edit` flag
- Intuitive behavior based on file type

### File Detection
- Extension-based: `.designspace` vs `.ufo`
- Could also check for `metainfo.plist` (UFO) vs XML structure (designspace)
- FontIR source already does this internally

## References

- [Unified Font Object Specification](https://unifiedfontobject.org/)
- [Designspace Format Documentation](https://fonttools.readthedocs.io/en/stable/designspaceLib/index.html)
- FontIR source code: `ufo2fontir::source::DesignSpaceIrSource`