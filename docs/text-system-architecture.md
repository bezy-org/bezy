# Text System Architecture

This document provides a comprehensive overview of Bezy's text system architecture, focusing on the unified LTR/RTL text tool design and multiple text buffer management.

## Overview

Bezy's text system is built around a unified text tool that handles both left-to-right (LTR) scripts like Latin and right-to-left (RTL) scripts like Arabic and Hebrew. The system maintains multiple independent text buffers while providing seamless cursor/insert mode operation across them.

## Core Architecture

### Key Components

#### 1. Text Tool (`src/tools/text.rs`)
- Single unified tool for both LTR and RTL text placement
- Four placement modes:
  - **LTRText**: Left-to-right text mode (Latin scripts)
  - **RTLText**: Right-to-left text mode (Arabic/Hebrew scripts)
  - **Insert**: Keyboard-based editing within existing text buffers
  - **Freeform**: Individual sorts positioned freely in design space

#### 2. Buffer System (`src/core/state/text_editor/buffer.rs`)
- **SortBuffer**: Gap buffer implementation for efficient text editing
- **BufferId**: Unique identifier for text buffer isolation
- **SortEntry**: Individual glyph or line break within a buffer
- **SortLayoutMode**: Defines text direction (LTRText, RTLText, Freeform)

#### 3. Text Editor State (`src/core/state/text_editor/editor.rs`)
- **TextEditorState**: Central state management for all text operations
- **ActiveSortEntity**: Tracks which sort is currently active for editing
- **GridConfig**: Layout configuration for text buffer positioning

#### 4. Text Shaping System (`src/systems/text_shaping.rs`)
- **TextShapingCache**: Caches shaped text results
- **ShapedText**: Contains harfrust-processed glyph information
- **TextDirection**: Directional information for shaping engines

## Multi-Buffer Architecture

### New Buffer Entity System
The system now uses **ECS entities for each text buffer** instead of storing buffer metadata in sorts:

1. **Buffer Entities**: Each text buffer is an ECS entity with buffer-specific components
2. **Buffer Components**: `TextBuffer`, `BufferCursor`, and `BufferMember` components
3. **Clean Separation**: Buffer properties live in buffer entities, not individual sorts
4. **Proper Isolation**: Complete separation between different text flows via ECS

### Root Sort Behavior
**IMPORTANT**: The root sort is just the **first sort in a text buffer**. It behaves exactly like any other glyph sort:

- ✅ **Same advance width** as the glyph it represents
- ✅ **Same glyph properties** - codepoint, glyph name, metrics
- ✅ **Same editing behavior** - can be selected, moved, etc.
- ✅ **Same positioning behavior** - no special coordinate handling

The only differences are:
- 🎯 **Buffer properties UI**: Can be right-clicked to edit buffer-wide properties

**IMPORTANT**: The root sort **changes dynamically**:
- If you delete the first sort, the second sort becomes the root
- If you move cursor to position 0 and type, the new character becomes the root
- Root sort identification is always the sort at buffer index 0

**Root sorts should NEVER have special advance widths, positioning, or text rendering behavior.**

### New Buffer Entity Architecture
```
ECS World
├── Buffer Entity 1 (LTR)
│   ├── TextBuffer { id: BufferId(1), layout_mode: LTRText, root_position: (100, 200), is_active: true }
│   ├── BufferCursor { position: 3 }
│   └── Name("TextBuffer-1-LTRText")
│
├── Buffer Entity 2 (RTL)  
│   ├── TextBuffer { id: BufferId(2), layout_mode: RTLText, root_position: (500, 300), is_active: false }
│   ├── BufferCursor { position: 1 }
│   └── Name("TextBuffer-2-RTLText")
│
├── Sort Entity 1 ('h')
│   ├── Sort { glyph_name: "h" }
│   ├── BufferMember { buffer_entity: Buffer Entity 1, buffer_index: 0 }
│   ├── ActiveSort
│   └── Transform { translation: (100, 200, 0) }
│
├── Sort Entity 2 ('e')
│   ├── Sort { glyph_name: "e" }
│   ├── BufferMember { buffer_entity: Buffer Entity 1, buffer_index: 1 }
│   └── Transform { translation: (592, 200, 0) }
│
└── Freeform Sort Entity ('z')
    ├── Sort { glyph_name: "z" }
    └── Transform { translation: (700, 100, 0) }
```

### New ECS Component Details

#### TextBuffer Component
```rust
#[derive(Component, Debug, Clone)]
pub struct TextBuffer {
    pub id: BufferId,                      // Unique buffer identifier
    pub layout_mode: SortLayoutMode,       // LTRText or RTLText
    pub root_position: Vec2,               // World position of buffer start
    pub is_active: bool,                   // Currently active for editing
}
```

#### BufferCursor Component
```rust
#[derive(Component, Debug, Clone)]
pub struct BufferCursor {
    pub position: usize,                   // Cursor position within the buffer
}
```

#### BufferMember Component  
```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct BufferMember {
    pub buffer_entity: Entity,             // The buffer entity this sort belongs to
    pub buffer_index: usize,               // Index of this sort within the buffer (0 = root)
}
```

#### ActiveTextBuffer Resource
```rust
#[derive(Resource, Default, Debug)]
pub struct ActiveTextBuffer {
    pub buffer_entity: Option<Entity>,     // The currently active buffer entity
}
```

#### BufferId System
- Each text buffer gets a unique `BufferId` 
- Freeform sorts are stored separately in `freeform_sorts` vector
- Complete isolation between different text flows
- IDs are generated atomically using `AtomicU32`

## Text Tool Modes

### 1. LTR Text Mode
- Creates new LTR text buffer with Latin placeholder ('a') 
- Text flows left-to-right
- **Auto-switches to Insert mode** after placement for immediate typing
- Cursor starts at position 1 (after the root character) for natural text flow
- Character positioning: `x_offset += previous_character.advance_width`

### 2. RTL Text Mode  
- Creates new RTL text buffer with Arabic placeholder ('alef-ar')
- Text flows right-to-left  
- **Auto-switches to Insert mode** after placement for immediate typing
- Cursor starts at position 0 (before the root character) for natural RTL flow
- Complex positioning: each character's right edge touches previous character's left edge
- Uses Arabic Unicode range detection for complex script shaping

### 3. Insert Mode
- Operates on the **last active buffer** (whichever buffer last had a selected/active sort)
- Does not create new buffers, only edits existing ones
- Respects the original buffer's text direction (LTR/RTL)
- **Automatic target**: LTR/RTL modes switch to this after placing sorts

### 4. Freeform Mode
- Creates individual sorts with no buffer affiliation
- Each sort positioned independently in world space
- **Stays in Freeform mode** for multi-placement workflow
- No text flow or cursor-based editing

## Cursor and Insert System

### Cursor Operation  
- **Per-Buffer Cursors**: Each buffer entity has its own `BufferCursor` component
- **Active Buffer Tracking**: `ActiveTextBuffer` resource tracks which buffer is active
- **Cursor Position**: Stored in `BufferCursor` component, completely separate from sorts
- **Insert Mode Target**: Always operates on the active buffer entity
- **Dynamic Root Support**: Cursor position persists even when root sort changes

### Text Direction Handling

#### LTR (Left-to-Right)
**LTR means text flows from LEFT to RIGHT. New characters are added to the RIGHT side.**
```
Root position: (100, 200)
[Root: 'a'] → [Char: 'b'] → [Char: 'c'] → |cursor
Positions:     (100,200)   (200,200)    (350,200)  (cursor ready for next char)

Standard text editor behavior:
- Cursor starts AFTER (to the right of) the initial character
- Each new character appears to the RIGHT of the cursor  
- Cursor advances RIGHT after each character
- User can move cursor LEFT with arrow keys to insert/edit
```

#### RTL (Right-to-Left)
```
Root position: (100, 200)
|cursor ← [Char: 'ج'] ← [Char: 'ب'] ← [Root: 'أ']  
Positions:   (50,200)     (100,200)   (150,200)   (100,200)
```

## Text Shaping Integration

### HarfRust Integration
- **Complex Script Support**: Uses HarfRust (Rust HarfBuzz bindings) for Arabic shaping
- **Contextual Forms**: Handles initial, medial, final, isolated forms
- **Bidirectional Text**: Proper RTL/LTR text mixing
- **Ligature Support**: Arabic ligature handling

### Shaping Pipeline
1. **Detection**: `needs_complex_shaping()` identifies Arabic/complex scripts
2. **Shaping**: `shape_text_with_harfbuzz()` processes text runs
3. **Caching**: Results cached in `TextShapingCache`
4. **Positioning**: Shaped glyphs positioned according to text direction

### Script Detection
```rust
// Arabic Unicode ranges covered:
// U+0600-U+06FF: Arabic
// U+0750-U+077F: Arabic Supplement  
// U+08A0-U+08FF: Arabic Extended-A
// U+FB50-U+FDFF: Arabic Presentation Forms-A
// U+FE70-U+FEFF: Arabic Presentation Forms-B
```

## Implementation Status & Migration Plan

### Current Status (New ECS Architecture) ✅
The system now uses a clean ECS-based buffer architecture with major improvements completed:

- **Buffer Entities**: Each text buffer is an ECS entity with dedicated components
- **Clean Separation**: Sorts contain no buffer metadata, just `BufferMember` references  
- **ECS Cursor Storage**: Cursor positions stored in `BufferCursor` components
- **Proper Isolation**: Complete separation via ECS entity relationships
- **Legacy Code Cleanup**: Removed conflicting dual systems for educational simplicity

### Architecture Benefits Achieved ✅
- **Clear separation**: Buffer properties in buffer entities, not sorts
- **Simplified sorts**: Sorts only contain glyph data and buffer membership
- **Clean cursor management**: `BufferCursor` components handle cursor storage
- **Better isolation**: ECS entities provide natural separation between text flows
- **Educational Ready**: No more confusing legacy systems running in parallel

### Recent Major Improvements (Sept 2025) ✅

#### Text System Cleanup & Bug Fixes
- **✅ Fixed missing cursor in Insert mode**: Cursor now appears when app starts normally (without `--no-default-buffer`)
- **✅ Fixed cross-contamination between text buffers**: Replaced legacy `get_text_sort_flow_position` with ECS-based positioning
- **✅ Fixed cursor visibility logic**: Cursor only shows in Text tool + Insert mode (eliminates visual confusion)
- **✅ Removed legacy code**: Deleted unused functions (`calculate_legacy_cursor_position`, `handle_unicode_text_input_legacy`)
- **✅ Buffer visual markers**: Temporarily disabled distracting page icons (code preserved for future use)

#### ECS Buffer System Enhancements
- **✅ Auto buffer entity creation**: System automatically creates TextBuffer entities for legacy sorts with buffer_id
- **✅ Active buffer management**: Enhanced system to auto-set active buffer when none is set
- **✅ BufferMember linking**: Sort entities now properly linked to buffer entities during spawning
- **✅ ECS positioning system**: New `calculate_buffer_local_position()` function replaces legacy cross-contamination logic

#### Vertical Alignment Fix
- **✅ Fixed line height calculation**: Corrected formula to match legacy system (`line_height = upm - descender`)
- **✅ Multi-line positioning**: Proper line break handling for text that spans multiple rows

### Migration Plan

#### Phase 1: Improve Current System ✅ 
- [x] Document current architecture
- [x] Create target architecture design
- [x] Identify key improvement areas

#### Phase 2: ECS Buffer Implementation ✅
- [x] Create new TextBuffer, BufferCursor, BufferMember components
- [x] Implement buffer entity creation in sort placement
- [x] Update cursor rendering to use buffer entities  
- [x] Add TextBufferManagerPlugin for buffer management
- [x] Update architecture documentation

#### Phase 3: Legacy Cleanup ✅ COMPLETED
- [x] Remove unused legacy functions and clean up dead code
- [x] Replace legacy positioning with ECS buffer-aware system
- [x] Fix buffer cross-contamination issues
- [x] Ensure educational simplicity - no dual systems

### Current Working Status

#### ✅ Working Features (LTR Text)
- **Insert Mode**: Cursor appears correctly, typing works
- **Arrow Key Navigation**: Left/right/up/down cursor movement
- **Character Insertion**: Proper positioning and buffer management
- **Backspace/Delete**: Cursor-based deletion works correctly
- **Line Breaks**: Enter key creates new lines with proper positioning
- **Buffer Isolation**: Multiple text buffers work independently
- **Cursor Visibility**: Only shows when functional (Text tool + Insert mode)

#### ❌ Known Issues

##### RTL Text System Completely Broken
- **RTL text tools are completely non-functional** - this is a major issue
- Root cause: RTL positioning logic needs complete rewrite for new ECS architecture
- Arabic text placement, cursor positioning, and text flow all broken
- RTL-specific features not migrated to new buffer entity system

##### Minor Issues
- Legacy functions still exist in `editor.rs` (test functions and unused helpers)
- Some deprecated fields like `buffer_cursor_position` could be cleaned up
- Buffer visual markers are disabled (by design, but may want them back eventually)

### Technical Implementation Details (Sept 2025)

#### Key Files Modified
- **`/src/systems/text_editor_sorts/sort_entities.rs`**: 
  - Added `calculate_buffer_local_position()` function for ECS-based positioning
  - Updated `spawn_missing_sort_entities()` to use new positioning
  - Updated `update_buffer_sort_positions()` to use buffer entities
  - Added BufferMember component linking during entity creation

- **`/src/systems/text_buffer_manager.rs`**:
  - Added `create_missing_buffer_entities()` system for legacy compatibility
  - Enhanced `update_active_buffer()` with fallback logic
  - Temporarily disabled `render_buffer_markers()` (visual page icons)

- **`/src/systems/text_editor_sorts/sort_rendering.rs`**:
  - Fixed cursor visibility logic to require Text tool + Insert mode
  - Removed 150+ lines of unused `calculate_legacy_cursor_position()` function

- **`/src/ui/edit_mode_toolbar/text.rs`**:
  - Changed default TextPlacementMode from LTRText to Insert for immediate usability

#### ECS Architecture Changes
```rust
// NEW: Buffer entity auto-creation from legacy buffer_id
fn create_missing_buffer_entities() {
    // Creates TextBuffer entities for sorts with buffer_id but no entity
}

// NEW: ECS-based positioning (replaces legacy cross-contamination)  
fn calculate_buffer_local_position() {
    // Uses BufferMember relationships for proper isolation
    // Formula: line_height = upm - descender (matches legacy exactly)
}

// ENHANCED: BufferMember linking during sort creation
entity_commands.insert(BufferMember::new(buffer_entity, buffer_local_index));
```

### Next Priority Areas
1. **🔥 URGENT: Fix RTL text system** - completely non-functional
2. **Text Shaping Integration**: Connect HarfRust to rendering pipeline for Arabic
3. **Final legacy cleanup**: Remove remaining deprecated fields from SortEntry
4. **RTL cursor positioning**: Rewrite RTL logic for new ECS architecture

## Key Design Decisions

### 1. Unified Tool Approach
Instead of separate LTR and RTL tools, one tool with mode switching provides:
- Consistent user experience
- Shared logic for common operations
- Easy mode switching with Tab key

### 2. Buffer ID Isolation
Each text buffer gets a unique ID to:
- Prevent cross-contamination between text flows
- Enable proper cursor targeting in insert mode
- Support multiple independent text areas

### 3. Gap Buffer Storage
Single gap buffer for all sorts provides:
- Efficient insertion/deletion operations
- Memory locality for better performance
- Simplified memory management

### 4. Root-Based Cursor Storage
Cursor position stored in buffer root rather than global state:
- Supports multiple independent cursors
- Maintains cursor position when switching between buffers
- Simplifies cursor state management

## Usage Patterns

### Creating New Text Buffers
1. Select Text tool (T key)
2. Choose LTR or RTL mode (Tab to cycle)
3. Click in design space
4. Type characters - they flow in buffer sequence

### Editing Existing Buffers  
1. Select Text tool, switch to Insert mode
2. Click on any character in desired buffer
3. That buffer becomes active for typing
4. Cursor operations work within that buffer

### Mixed Text Scenarios
- Multiple independent LTR buffers
- Multiple independent RTL buffers  
- LTR and RTL buffers coexisting
- Freeform sorts mixed with text buffers

This architecture provides the foundation for sophisticated multilingual font editing with proper text direction support while maintaining simplicity and performance.