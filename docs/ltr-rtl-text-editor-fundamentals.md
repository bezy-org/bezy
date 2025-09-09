# LTR and RTL Text Editor Fundamentals

This document explains the fundamental concepts of how Left-to-Right (LTR) and Right-to-Left (RTL) text editors work. This serves as reference context for implementing text editing functionality.

## Core Concepts

### Text Direction Definitions
- **LTR (Left-to-Right)**: Text flows from left to right (English, Latin scripts)
- **RTL (Right-to-Left)**: Text flows from right to left (Arabic, Hebrew scripts)

### Visual vs Logical Order
- **Visual Order**: How text appears on screen
- **Logical Order**: How text is stored in memory (always follows reading direction)

## How LTR Text Editors Work

### Character Insertion
1. **Cursor Position**: Marks insertion point
2. **Character Added**: Appears at cursor position
3. **Cursor Movement**: Moves RIGHT after character insertion
4. **Text Flow**: Characters accumulate left-to-right

### LTR Cursor Positioning Logic
```
Text: "abc"
Positions: |a|b|c|
Cursor at position 0: |abc (before 'a')
Cursor at position 1: a|bc (between 'a' and 'b') 
Cursor at position 2: ab|c (between 'b' and 'c')
Cursor at position 3: abc| (after 'c')
```

### Advance Width Calculation (LTR)
```rust
// Start at root position (0.0)
x_offset = 0.0;

// For each character BEFORE cursor position
for char in text[0..cursor_position] {
    x_offset += char.advance_width;  // Move RIGHT
}

// Cursor positioned AFTER all preceding text
```

### Arrow Key Behavior (LTR)
- **Left Arrow**: Move cursor left (decrease position)
- **Right Arrow**: Move cursor right (increase position)

## How RTL Text Editors Work

### Character Insertion
1. **Cursor Position**: Marks insertion point (visually at LEFT edge of existing text)
2. **Character Added**: Appears to the LEFT of cursor position (pushing existing text further left)
3. **Cursor Movement**: Moves LEFT after character insertion
4. **Text Flow**: Characters accumulate right-to-left

### Cursor Positioning Logic (RTL)
```
Text: "אבג" (Hebrew: alef-bet-gimel)
Visual: "|אבג" (text flows right-to-left, cursor at left edge of new sorts)
Cursor at position 0: אבג| (before first character - rightmost position)
Cursor at position 1: א|בג (after first character)
Cursor at position 2: אב|ג (after second character) 
Cursor at position 3: |אבג (after all text - leftmost position)
```

### Advance Width Calculation (RTL)
```rust
// RTL cursor calculation - CRITICAL UNDERSTANDING:
// WRONG APPROACH: Starting from left edge like LTR
// RIGHT APPROACH: Start from RIGHT EDGE and work toward insertion point

x_offset = 0.0;  // Start at root position (rightmost edge)

// For each character AT OR AFTER cursor position
for char in text[cursor_position..] {
    x_offset -= char.advance_width;  // Move LEFT by width of following text
}

// Result: Cursor positioned at LEFT EDGE of existing text (insertion point)
// This is where the next character will appear in RTL text
```

### Arrow Key Behavior (RTL)
- **Left Arrow**: Move cursor left (increase position - toward end of text buffer)
- **Right Arrow**: Move cursor right (decrease position - toward beginning of text)

**IMPORTANT NOTE ON ARROW KEYS:**
Arrow keys should ALWAYS move the cursor visually in the direction of the arrow, regardless of text direction:
- Left arrow ALWAYS moves cursor visually LEFT on screen
- Right arrow ALWAYS moves cursor visually RIGHT on screen

The difference in RTL is how this visual movement maps to buffer positions:
- In LTR: visual left = decrease position, visual right = increase position  
- In RTL: visual left = increase position, visual right = decrease position

**DO NOT** reverse the visual behavior of arrow keys - users expect the cursor to move in the direction of the arrow they pressed!

## Key Differences Summary

| Aspect | LTR | RTL |
|--------|-----|-----|
| Text Flow |LTR = Left → Right | RTL = Right → Left |
| Character Insertion | Right of cursor | Left of cursor |
| Cursor Movement After Typing | Rightward | Leftward |
| Advance Width Math | Add (+) | Subtract (-) |
| Starting Position | Left edge | Right edge |
| Left Arrow | Decrease position | Increase position |
| Right Arrow | Increase position | Decrease position |

## Implementation Requirements

### RTL Text Editor Must:
1. **Start calculations from right edge** (root position = rightmost point)
2. **Subtract advance widths** to move cursor leftward
5. **Maintain visual consistency** with standard RTL editors

### Common Mistakes:
- ❌ Adding advance widths in RTL (should subtract)
- ❌ Starting from left edge in RTL (should start from right) 
- ❌ Reversing arrow key logic unnecessarily
- ❌ Positioning cursor at wrong edge of text

## Implementation Notes (Bezy Editor)

### Buffer Storage Architecture
Text is stored in **logical order** (reading order) in a `Vec<SortEntry>` buffer, with visual presentation handled by the rendering system. For RTL text, buffer position 0 corresponds to the rightmost visual position, and position N to the leftmost. This follows Unicode standards and industry practices.

### Data Structure Performance
- **Current**: Vec-based buffer with O(n) insertion/deletion complexity
- **Trade-off**: Implementation simplicity over performance, suitable for small text typical in font editing
- **Future**: Consider gap buffer (O(1) operations) or rope data structure for larger documents

### Cursor Positioning Implementation
The RTL cursor positioning is implemented in `src/systems/text_editor_sorts/sort_rendering.rs`:
- Collects all sorts belonging to the specific buffer (avoiding cross-buffer contamination)
- Uses local buffer indexing instead of global indices
- For RTL: Accumulates widths of characters AT OR AFTER cursor position
- This correctly positions the cursor at the LEFT edge where new text will be inserted

### Arrow Key Implementation  
The arrow keys are implemented in `src/systems/text_editor_sorts/unicode_input.rs`:
- **Simplified approach**: Arrow keys work identically for both LTR and RTL
- Left arrow always decreases buffer position
- Right arrow always increases buffer position
- The visual movement is handled entirely by the cursor rendering system
- This ensures arrows always move the cursor visually in the expected direction

### Why This Works
In RTL mode:
- Position 0 = rightmost (where typing starts)
- Position N = leftmost (where cursor ends after typing)
- Decreasing position (left arrow) moves cursor from left to right visually
- Increasing position (right arrow) moves cursor from right to left visually
- The visual result matches user expectations without special-casing the arrow logic
