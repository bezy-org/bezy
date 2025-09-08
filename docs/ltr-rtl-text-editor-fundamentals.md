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
1. **Cursor Position**: Marks insertion point (visually at RIGHT edge of existing text)
2. **Character Added**: Appears to the LEFT of cursor position
3. **Cursor Movement**: Cursor stays at same visual position (new text grows leftward)
4. **Text Flow**: Characters accumulate right-to-left

### Cursor Positioning Logic (RTL)
```
Text: "אבג" (Hebrew: alef-bet-gimel)
Visual: גבא| (text flows right-to-left, cursor at right edge)

Cursor at position 0: |אבג (before first character - rightmost position)
Cursor at position 1: א|בג (after first character)
Cursor at position 2: אב|ג (after second character) 
Cursor at position 3: אבג| (after all text - leftmost position)
```

### Advance Width Calculation (RTL)
```rust
// RTL cursor calculation - CRITICAL UNDERSTANDING:
// In RTL, we start from the RIGHT and work LEFT
// The cursor position represents how far LEFT we've moved from the right edge

x_offset = 0.0;  // Start at root position (rightmost edge)

// For each character BEFORE cursor position  
for char in text[0..cursor_position] {
    x_offset -= char.advance_width;  // Move LEFT (subtract!)
}

// Cursor positioned at LEFT edge of all preceding text
```

### Arrow Key Behavior (RTL)
- **Left Arrow**: Move cursor left (increase position - toward beginning of text)
- **Right Arrow**: Move cursor right (decrease position - toward end of text)

## Key Differences Summary

| Aspect | LTR | RTL |
|--------|-----|-----|
| Text Flow | Left → Right | Right → Left |
| Character Insertion | Right of cursor | Left of cursor |
| Cursor Movement After Typing | Rightward | Stays at insertion point |
| Advance Width Math | Add (+) | Subtract (-) |
| Starting Position | Left edge | Right edge |
| Left Arrow | Decrease position | Increase position |
| Right Arrow | Increase position | Decrease position |

## Implementation Requirements

### RTL Text Editor Must:
1. **Start calculations from right edge** (root position = rightmost point)
2. **Subtract advance widths** to move cursor leftward
3. **Position cursor at left edge** of preceding text
4. **Handle arrow keys logically** (left arrow moves toward text beginning)
5. **Maintain visual consistency** with standard RTL editors

### Common Mistakes:
- ❌ Adding advance widths in RTL (should subtract)
- ❌ Starting from left edge in RTL (should start from right) 
- ❌ Reversing arrow key logic unnecessarily
- ❌ Positioning cursor at wrong edge of text

## Testing RTL Implementation

### Expected Behavior:
1. **Empty buffer**: Cursor at (0.0, 0.0) - root position
2. **Type one character**: Cursor moves to (-advance_width, 0.0) 
3. **Type second character**: Cursor moves to (-total_width, 0.0)
4. **Left arrow**: Cursor moves toward text beginning (position increases)
5. **Right arrow**: Cursor moves toward text end (position decreases)

### Debug Questions:
- Is cursor positioned at left edge of existing text?
- Do advance widths subtract (not add) in RTL?
- Do arrow keys move cursor in expected logical direction?
- Does typing move cursor to correct insertion point?

This document serves as the definitive reference for LTR/RTL text editor behavior in the Bezy font editor.
