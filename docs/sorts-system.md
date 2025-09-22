# Sorts System in Bezy

## What is a Sort?

A "sort" is a visual container in the Bezy font editor that displays the outline and metrics of a glyph. Think of it as a viewport or frame that can show any glyph from the font being edited.

## Key Concepts

### Sorts are NOT tied to specific glyphs
- A sort is a flexible container that can display ANY glyph from the font
- When you type a character, you're changing what glyph that sort displays
- The sort itself remains the same entity, just showing different content

### Active vs Inactive Sorts
- **Active Sort**: The sort currently being edited
  - Shows green metrics lines
  - Has editable control points (can move bezier handles)
  - Only ONE sort can be active at a time
  - This is where editing operations happen

- **Inactive Sorts**: All other sorts in the text buffer
  - Show filled outlines (no editable points)
  - Display the glyph but cannot be edited
  - Multiple inactive sorts can exist

### The Text Buffer
- Contains multiple sorts arranged in a text-like layout
- Each sort in the buffer displays a glyph
- Sorts can be navigated like text (cursor movement)
- The buffer maintains sort order and positioning

## How Sort Switching Works

### Typing Characters
- When you type (e.g., press 'a'), the sort at the cursor position changes to display that glyph
- The sort remains in the same position in the buffer
- Typing does NOT make a sort active - it just changes what glyph is displayed

### Activating a Sort
- Click on a sort or navigate to it with cursor keys
- The previously active sort becomes inactive (filled outline)
- The newly activated sort shows green metrics and editable points
- Only the active sort can be edited

### TUI Integration Goal
When selecting a codepoint in the TUI (e.g., pressing Enter on "U+0041 A"):
1. Find the currently active sort (the one with green metrics)
2. Change that active sort to display the selected glyph
3. The sort remains active and in the same position
4. This is similar to typing but through TUI selection instead of keyboard

## Common Misconceptions

❌ **Wrong**: "Creating a sort for Unicode U+0041"
- Sorts aren't created for specific glyphs
- Sorts are containers that can show any glyph

❌ **Wrong**: "Each glyph needs its own sort"
- One sort can display any glyph from the font
- You can have multiple sorts showing the same glyph

❌ **Wrong**: "Typing makes a sort active"
- Typing changes what glyph a sort displays
- Activation is a separate action (clicking/navigating to the sort)

## TextEditorState Methods

- `activate_sort(index)`: Makes the sort at the given index active
- `insert_sort_at_cursor()`: Creates a new sort at the cursor position
- `buffer.get(index)`: Gets the sort at a specific index
- `sort.kind.glyph_name()`: Gets the name of the glyph currently displayed in the sort
- `sort.kind.codepoint()`: Gets the Unicode codepoint of the displayed glyph

## Current Implementation Issue

The TUI glyph selection currently tries to:
1. Find a sort displaying the selected glyph
2. Activate that sort

What it SHOULD do:
1. Keep the currently active sort active
2. Change what glyph that active sort displays
3. Just like typing would do, but triggered from TUI

This is the fundamental misunderstanding that needs to be corrected in the implementation.