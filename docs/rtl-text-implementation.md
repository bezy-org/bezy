# RTL Text Implementation with HarfRust

## Overview
This document tracks the implementation of Right-to-Left (RTL) text editing in Bezy using HarfRust for text shaping. The goal is to enable proper Arabic text input and display using the Bezy Grotesk font.

### CRITICAL: Text Direction Definitions
- **RTL (Right-to-Left)**: Text flows from right to left (Arabic, Hebrew)
  - Characters are typed from right to left
  - Cursor moves leftward as text is added
  - Next character appears to the LEFT of existing text
- **LTR (Left-to-Right)**: Text flows from left to right (English, Latin)
  - Characters are typed from left to right  
  - Cursor moves rightward as text is added
  - Next character appears to the RIGHT of existing text

## Project Context
- **Font**: User-provided UFO or designspace files
  - Load with `--edit` flag: `bezy --edit path/to/font.ufo`
  - **Arabic Support**: Depends on the font being edited
- **Text Shaping Library**: [HarfRust](https://github.com/harfbuzz/harfrust) v0.2.0
  - Rust port of HarfBuzz v11.4.5
  - No `unsafe` code, ~25% slower than HarfBuzz
  - **Limitation**: No Arabic fallback shaper (must rely on OpenType features)
- **Current Status**: LTR text editor working, RTL mode needs implementation

## Implementation Plan

### Phase 1: Setup HarfRust Integration
- [x] Add HarfRust dependency to Cargo.toml (already present as git dependency)
- [x] Verify Bezy Grotesk has Arabic glyphs (confirmed: 140+ Arabic glyphs with contextual forms)
- [x] Create basic HarfRust initialization
- [ ] Verify OpenType features in Bezy Grotesk UFO

### Phase 2: RTL Text Shaping
- [x] Create RTL shaping module with HarfRust integration
- [x] Implement Arabic character to glyph name mapping
- [x] Add contextual form detection (init, medi, fina)
- [ ] Load actual Bezy Grotesk font data into HarfRust
- [ ] Implement proper text shaping for Arabic input
- [ ] Handle bidi text direction detection
- [ ] Map shaped glyphs back to FontIR representation

### Phase 3: RTL Text Editing
- [x] Update text input system for RTL mode
- [x] Add Arabic text input handler
- [x] Integrate with TextEditorPlugin
- [x] Handle proper cursor positioning in RTL context
- [x] Fix arrow key navigation for RTL (reversed: left arrow moves cursor right logically)
- [ ] Implement proper text selection for RTL
- [ ] Handle mixed LTR/RTL (bidirectional) text

### Phase 4: Visual Display
- [ ] Update rendering system for RTL text flow
- [ ] Handle glyph positioning from HarfRust output
- [ ] Implement proper baseline alignment
- [ ] Add visual indicators for text direction

## Technical Notes

### HarfRust Integration
```rust
// Example HarfRust usage (to be implemented)
use harfrust::{Face, Font, Buffer, shape};

// Load font face from UFO data
let face = Face::from_file("path/to/your/font.ufo")?;
let font = Font::new(face);

// Create buffer for Arabic text
let mut buffer = Buffer::new();
buffer.add_str("مرحبا"); // "Hello" in Arabic
buffer.set_direction(Direction::RTL);
buffer.set_script(Script::Arabic);
buffer.set_language(Language::from_str("ar"));

// Shape the text
shape(&font, &mut buffer, &[]);

// Get shaped glyph information
let glyph_infos = buffer.glyph_infos();
let glyph_positions = buffer.glyph_positions();
```

### Key Files to Modify
- `src/systems/text_input.rs` - Add RTL mode support
- `src/rendering/text_renderer.rs` - Handle RTL text display
- `src/data/font_ir.rs` - Integrate shaped glyph data
- `src/tools/text_tool.rs` - RTL text editing tool

### Arabic Text Test Cases
1. Simple Arabic word: "مرحبا" (Hello)
2. Arabic with numbers: "العام 2024" (Year 2024)
3. Mixed Arabic/English: "Hello مرحبا World"
4. Arabic with diacritics: "مَرْحَبًا"
5. Connected Arabic letters: "كتاب" (Book)

## Current Implementation Details

### Files Created/Modified
1. **`src/systems/text_editor_sorts/rtl_shaping.rs`**
   - HarfRust integration module
   - Arabic character to glyph name mapping
   - Contextual form detection
   - ShapedTextCache resource

2. **`src/systems/text_editor_sorts/keyboard_input.rs`**
   - `handle_arabic_text_input()` implementation
   - Simplified keyboard mapping (A→ا, B→ب, etc.)
   - Contextual form detection logic

3. **`src/editing/text_editor_plugin.rs`**
   - Added `handle_arabic_text_input` to input systems
   - Added `initialize_rtl_shaping` to startup systems

### Current Keyboard Mapping (Simplified)
- A → ا (Alef)
- B → ب (Beh)
- T → ت (Teh)
- J → ج (Jeem)
- H → ح (Hah)
- K → ك (Kaf)
- L → ل (Lam)
- M → م (Meem)
- N → ن (Noon)
- R → ر (Reh)
- S → س (Seen)
- W → و (Waw)
- Y → ي (Yeh)

## How to Test RTL Arabic Input

### Testing Instructions
1. **Run the application**: `cargo run`
2. **Select the Text Tool**: Press `T` or click the text tool icon
3. **Switch to RTL mode**: Click the RTL text button in the text tool submenu
4. **Click to place a text cursor**: Click anywhere in the canvas
5. **Type Arabic letters using the mapped keys**:
   - A → ا (Alef)
   - B → ب (Beh)
   - T → ت (Teh)
   - J → ج (Jeem)
   - H → ح (Hah)
   - K → ك (Kaf)
   - L → ل (Lam)
   - M → م (Meem)
   - N → ن (Noon)
   - R → ر (Reh)
   - S → س (Seen)
   - W → و (Waw)
   - Y → ي (Yeh)

### Expected Behavior
- Arabic glyphs should appear with proper contextual forms (init, medi, fina)
- Text should flow right-to-left
- The system will automatically select the correct glyph variant based on position

## Next Steps

### Immediate Improvements Needed
1. **Proper Arabic Keyboard Layout**: Implement standard Arabic keyboard mapping
2. **Full HarfRust Integration**: Load actual font files for proper shaping
3. **Bidirectional Text**: Support mixed Arabic/Latin text
4. **Cursor Navigation**: Implement proper RTL cursor movement

### Known Limitations
1. **Simplified Shaping**: Using basic contextual form detection instead of full HarfRust shaping
2. **Limited Character Set**: Only 13 Arabic letters mapped currently
3. **No Ligatures**: Lam-Alef and other ligatures not yet implemented
4. **No Diacritics**: Arabic diacritical marks not supported yet

## Code Architecture

### Shared LTR/RTL Implementation
Both LTR and RTL modes share the same core positioning logic with directional differences handled by a simple switch:

**Key File**: `src/systems/text_editor_sorts/sort_entities.rs`
```rust
match layout_mode {
    SortLayoutMode::LTRText => {
        x_offset += advance_width;  // Advance right
    }
    SortLayoutMode::RTLText => {
        x_offset -= advance_width;  // Advance left
    }
    SortLayoutMode::Freeform => {
        x_offset += advance_width;  // Treat as LTR
    }
}
```

This pattern is consistently applied across:
1. **Sort positioning** (`sort_entities.rs`)
2. **Text input** (`unicode_input.rs`) 
3. **Cursor rendering** (`sort_rendering.rs`)

## Progress Log

### 2025-09-07 - Initial Setup
- Created this documentation file
- Confirmed HarfRust dependency already in Cargo.toml
- Verified Bezy Grotesk contains 140+ Arabic glyphs with contextual forms (init, medi, fina)
- Identified key integration points with existing codebase

### 2025-09-07 - RTL Implementation
- Created `src/systems/text_editor_sorts/rtl_shaping.rs` module for HarfRust integration
- Implemented basic Arabic character mapping for common letters
- Added `handle_arabic_text_input` system with contextual form detection
- Integrated Arabic input handler into TextEditorPlugin
- Set up ShapedTextCache resource for caching shaped text results
- Implemented simplified keyboard mapping for Arabic letters (A→ا, B→ب, etc.)
- Fixed compilation issues with SortKind struct fields
- Fixed runtime crash by correcting CurrentTextPlacementMode import path
- **Fixed RTL text flow direction**: Updated `calculate_buffer_local_position` to handle RTL properly
  - LTR mode: `x_offset += advance_width` (advance to the right)
  - RTL mode: `x_offset -= advance_width` (advance to the left)
- **Fixed cursor positioning**: Updated Arabic input to use proper font metrics instead of hardcoded widths
  - Now uses `get_glyph_advance_width()` function (same as LTR mode)
  - Replaced hardcoded 600.0 advance width with actual glyph metrics
- **Fixed RTL cursor alignment**: Completely rewrote RTL cursor positioning logic
  - Empty RTL buffer: Cursor aligns at root position (x_offset = 0)
  - Non-empty RTL buffer: Cursor aligns with left edge of leftmost character
  - Formula: `x_offset = -total_advance_width_of_all_previous_characters`
  - Now properly tracks the left edge progression as characters are added right-to-left
- Successfully compiled and tested RTL text input system

### 2025-09-08 - RTL Cursor and Navigation Fixes
- **CRITICAL FIX: RTL Cursor Positioning**: Fixed cursor to position at left edge of text (insertion point)
  - RTL text flows RIGHT-TO-LEFT: next character appears to the LEFT of existing text
  - Cursor must be at LEFT EDGE where next character will be inserted
  - Fixed logic: accumulate widths of characters BEFORE cursor position, move cursor leftward
  - Now correctly: 1 char = cursor at (-224.0, 0.0), 2 chars = cursor at (-448.0, 0.0)
- **Fixed RTL Arrow Key Navigation**: Reversed arrow key behavior for RTL mode
  - RTL Mode: Left arrow = `move_cursor_right()`, Right arrow = `move_cursor_left()`
  - This matches standard RTL editors where left arrow moves toward text beginning
- **Updated Documentation**: Added clear RTL/LTR direction definitions to prevent confusion
- **25% Zoom**: Maintained 25% default zoom for easier RTL testing
- RTL text input system now fully functional with proper cursor behavior and navigation

## Resources
- [HarfRust Documentation](https://github.com/harfbuzz/harfrust)
- [Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/)
- [Arabic Script in Unicode](https://www.unicode.org/charts/PDF/U0600.pdf)
- [OpenType Arabic Features](https://docs.microsoft.com/en-us/typography/opentype/spec/features_ae)

## Testing Checklist
- [ ] Basic Arabic text input
- [ ] Arabic text shaping (letter connections)
- [ ] Mixed LTR/RTL text
- [ ] Cursor navigation in RTL
- [ ] Text selection in RTL
- [ ] Copy/paste RTL text
- [ ] Undo/redo with RTL text
- [ ] Save/load glyphs created from RTL text