//! Gap buffer implementation and data types for text editor

use bevy::prelude::*;

/// Text editor state for dynamic sort management
#[derive(Resource, Clone, Default)]
pub struct TextEditorState {
    /// The text buffer containing sort content (like a rope or gap buffer)
    pub buffer: SortBuffer,
    /// Current cursor position in the buffer
    pub cursor_position: usize,
    /// Selection range (start, end) if any
    pub selection: Option<(usize, usize)>,
    /// Viewport offset for scrolling
    pub viewport_offset: Vec2,
    /// Grid layout configuration
    pub grid_config: GridConfig,
}

/// Resource to track the active sort entity in ECS
/// This is the single source of truth for which sort is active
#[derive(Resource, Default)]
pub struct ActiveSortEntity {
    /// The entity ID of the currently active sort, if any
    pub entity: Option<Entity>,
}

/// Layout mode for individual sorts
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SortLayoutMode {
    /// Sort flows left-to-right like Latin text
    #[default]
    LTRText,
    /// Sort flows right-to-left like Arabic/Hebrew text
    RTLText,
    /// Sort is positioned freely in the world space
    Freeform,
}

/// Text mode configuration
#[derive(Resource, Clone, Debug, Default)]
pub struct TextModeConfig {
    /// Whether new sorts should be placed in text or freeform mode
    pub default_placement_mode: SortLayoutMode,
}

/// An entry in the sort buffer representing a glyph or a line break
#[derive(Clone, Debug, PartialEq)]
pub enum SortKind {
    Glyph {
        /// Unicode codepoint (primary identifier)
        codepoint: Option<char>,
        /// Glyph name (fallback identifier when no codepoint)
        glyph_name: String,
        /// With of the drawing space for each glyph
        advance_width: f32,
    },
    LineBreak,
}

/// Unified buffer of all sorts (both text and freeform) using gap buffer for efficient editing
/// Text sorts are grouped by their root sort (marked with is_buffer_root=true), while freeform sorts exist independently
/// This allows switching between text/freeform modes and managing all glyphs in one consistent structure
#[derive(Clone)]
pub struct SortBuffer {
    /// The gap buffer storage
    buffer: Vec<SortEntry>,
    /// Gap start position
    gap_start: usize,
    /// Gap end position (exclusive)
    gap_end: usize,
}

/// Unique identifier for text buffer flows
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferId(pub u32);

impl Default for BufferId {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferId {
    /// Create a new unique buffer ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT_ID: AtomicU32 = AtomicU32::new(1);
        BufferId(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// An entry in the sort buffer representing a glyph
#[derive(Clone, Debug)]
pub struct SortEntry {
    pub kind: SortKind,
    /// Whether this sort is currently active (in edit mode with points showing)
    pub is_active: bool,
    /// Layout mode for this sort
    pub layout_mode: SortLayoutMode,
    /// Root position (used for text buffer roots); for freeform sorts, use freeform_position
    pub root_position: Vec2,
    /// Whether this sort is a buffer root (first sort in a text buffer)
    pub is_buffer_root: bool,
    /// Cursor position within this buffer sequence (only for buffer roots)
    pub buffer_cursor_position: Option<usize>,
    /// Buffer ID for text flow isolation (None for freeform sorts)
    pub buffer_id: Option<BufferId>,
}

/// Grid layout configuration
#[derive(Clone)]
pub struct GridConfig {
    /// Number of sorts per row
    pub sorts_per_row: usize,
    /// Horizontal spacing between sorts
    pub horizontal_spacing: f32,
    /// Vertical spacing between rows
    pub vertical_spacing: f32,
    /// Starting position for the grid
    pub grid_origin: Vec2,
}

/// Iterator for gap buffer
pub struct SortBufferIterator<'a> {
    buffer: &'a SortBuffer,
    index: usize,
}

impl Default for SortBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SortEntry {
    fn default() -> Self {
        Self {
            kind: SortKind::Glyph {
                codepoint: None,
                glyph_name: String::new(),
                advance_width: 0.0,
            },
            is_active: false,
            layout_mode: SortLayoutMode::LTRText,
            root_position: Vec2::ZERO,
            is_buffer_root: false,
            buffer_cursor_position: None,
            buffer_id: None, // Default to no buffer ID (freeform)
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            sorts_per_row: 16,
            horizontal_spacing: 64.0,
            vertical_spacing: 400.0,
            grid_origin: Vec2::ZERO,
        }
    }
}

impl SortBuffer {
    /// Create a new gap buffer with initial capacity
    pub fn new() -> Self {
        let initial_capacity = 1024; // Start with room for plenty of sorts
        let mut buffer = Vec::with_capacity(initial_capacity);
        // Fill with default entries to create the gap
        buffer.resize(initial_capacity, SortEntry::default());

        Self {
            buffer,
            gap_start: 0,
            gap_end: initial_capacity,
        }
    }

    /// Get the logical length (excluding gap)
    pub fn len(&self) -> usize {
        self.buffer.len() - (self.gap_end - self.gap_start)
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get sort at logical position
    pub fn get(&self, index: usize) -> Option<&SortEntry> {
        if index >= self.len() {
            return None;
        }

        if index < self.gap_start {
            self.buffer.get(index)
        } else {
            self.buffer.get(index + (self.gap_end - self.gap_start))
        }
    }

    /// Get mutable sort at logical position
    pub fn get_mut(&mut self, index: usize) -> Option<&mut SortEntry> {
        if index >= self.len() {
            return None;
        }

        if index < self.gap_start {
            self.buffer.get_mut(index)
        } else {
            self.buffer.get_mut(index + (self.gap_end - self.gap_start))
        }
    }

    /// Move gap to position for efficient insertion/deletion
    fn move_gap_to(&mut self, position: usize) {
        if position == self.gap_start {
            return;
        }

        if position < self.gap_start {
            // Move gap left
            let move_count = self.gap_start - position;
            let gap_size = self.gap_end - self.gap_start;

            // Move elements from before gap to after gap
            for i in 0..move_count {
                let src_idx = position + i;
                let dst_idx = self.gap_end - move_count + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
            }

            self.gap_start = position;
            self.gap_end = position + gap_size;
        } else {
            // Move gap right
            let move_count = position - self.gap_start;
            let gap_size = self.gap_end - self.gap_start;

            // Move elements from after gap to before gap
            for i in 0..move_count {
                let src_idx = self.gap_end + i;
                let dst_idx = self.gap_start + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
            }

            self.gap_start = position;
            self.gap_end = position + gap_size;
        }
    }

    /// Insert sort at position
    pub fn insert(&mut self, index: usize, sort: SortEntry) {
        if index > self.len() {
            return;
        }

        // Ensure we have space in the gap
        if self.gap_start >= self.gap_end {
            self.grow_gap();
        }

        // Move gap to insertion point
        self.move_gap_to(index);

        // Insert at gap start
        self.buffer[self.gap_start] = sort;
        self.gap_start += 1;
    }

    /// Delete sort at position
    pub fn delete(&mut self, index: usize) -> Option<SortEntry> {
        if index >= self.len() {
            return None;
        }

        // Move the start of the gap to the deletion index.
        self.move_gap_to(index);

        // The element to be deleted is now at the end of the gap.
        // We "delete" it by incrementing the gap's end, effectively swallowing the element.
        let deleted_item = self.buffer[self.gap_end].clone();
        self.buffer[self.gap_end] = SortEntry::default(); // Clear the old slot
        self.gap_end += 1;

        Some(deleted_item)
    }

    /// Grow the gap when it gets too small
    fn grow_gap(&mut self) {
        let old_capacity = self.buffer.len();
        let new_capacity = old_capacity * 2;
        let gap_size = self.gap_end - self.gap_start;
        let new_gap_size = gap_size + (new_capacity - old_capacity);

        // Extend buffer
        self.buffer.resize(new_capacity, SortEntry::default());

        // Move elements after gap to end of new buffer
        let elements_after_gap = old_capacity - self.gap_end;
        if elements_after_gap > 0 {
            for i in (0..elements_after_gap).rev() {
                let src_idx = self.gap_end + i;
                let dst_idx = new_capacity - elements_after_gap + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
                self.buffer[src_idx] = SortEntry::default();
            }
        }

        // Update gap end
        self.gap_end = self.gap_start + new_gap_size;
    }

    /// Get an iterator over all sorts (excluding gap)
    pub fn iter(&self) -> SortBufferIterator {
        SortBufferIterator {
            buffer: self,
            index: 0,
        }
    }

    /// Clear all sorts and reset gap
    pub fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = SortEntry::default();
        }
        self.gap_start = 0;
        self.gap_end = self.buffer.len();
    }
}

impl<'a> Iterator for SortBufferIterator<'a> {
    type Item = &'a SortEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.buffer.len() {
            None
        } else {
            let item = self.buffer.get(self.index);
            self.index += 1;
            item
        }
    }
}

impl SortKind {
    pub fn is_glyph(&self) -> bool {
        matches!(self, SortKind::Glyph { .. })
    }

    pub fn is_line_break(&self) -> bool {
        matches!(self, SortKind::LineBreak)
    }

    pub fn codepoint(&self) -> Option<char> {
        match self {
            SortKind::Glyph { codepoint, .. } => *codepoint,
            SortKind::LineBreak => None,
        }
    }

    pub fn glyph_name(&self) -> &str {
        match self {
            SortKind::Glyph { glyph_name, .. } => glyph_name,
            SortKind::LineBreak => "",
        }
    }

    /// Get display string prioritizing codepoint over glyph name
    pub fn display_string(&self) -> String {
        match self {
            SortKind::Glyph {
                codepoint,
                glyph_name,
                ..
            } => {
                if let Some(cp) = codepoint {
                    format!("U+{:04X}", *cp as u32)
                } else {
                    glyph_name.clone()
                }
            }
            SortKind::LineBreak => "↵".to_string(),
        }
    }
}
