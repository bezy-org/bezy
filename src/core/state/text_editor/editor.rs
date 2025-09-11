//! Text editing operations and cursor management

use super::buffer::*;
use crate::core::state::FontMetrics;
use bevy::prelude::*;

impl TextEditorState {
    /// Get only text sorts (sorts that flow like text)
    pub fn get_text_sorts(&self) -> Vec<(usize, &SortData)> {
        let mut text_sorts = Vec::new();

        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.layout_mode == SortLayoutMode::LTRText
                    || sort.layout_mode == SortLayoutMode::RTLText
                {
                    text_sorts.push((i, sort));
                }
            }
        }

        text_sorts
    }

    /// Get sorts belonging to a specific buffer ID
    pub fn get_sorts_for_buffer(&self, buffer_id: BufferId) -> Vec<(usize, &SortData)> {
        let mut buffer_sorts = Vec::new();

        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.buffer_id == Some(buffer_id) {
                    buffer_sorts.push((i, sort));
                }
            }
        }

        buffer_sorts
    }

    /// Find the first sort for a specific buffer ID
    pub fn find_buffer_root(&self, buffer_id: BufferId) -> Option<(usize, &SortData)> {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.buffer_id == Some(buffer_id) {
                    return Some((i, sort));
                }
            }
        }
        None
    }

    /// Get all unique buffer IDs
    pub fn get_all_buffer_ids(&self) -> Vec<BufferId> {
        let mut buffer_ids = Vec::new();

        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if let Some(buffer_id) = sort.buffer_id {
                    if !buffer_ids.contains(&buffer_id) {
                        buffer_ids.push(buffer_id);
                    }
                }
            }
        }

        buffer_ids
    }

    /// Add a new freeform sort at the specified position
    pub fn add_freeform_sort(
        &mut self,
        glyph_name: String,
        position: Vec2,
        advance_width: f32,
        codepoint: Option<char>,
    ) {
        // Clear all states first
        self.clear_all_states();

        let new_sort = SortData {
            kind: SortKind::Glyph {
                codepoint,
                glyph_name: glyph_name.clone(),
                advance_width,
            },
            is_active: true, // Automatically activate the new sort
            layout_mode: SortLayoutMode::Freeform,
            root_position: position,
            buffer_cursor_position: None,
            buffer_id: None, // Freeform sorts have no buffer ID
        };

        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, new_sort);

        // Debug: Verify the sort was added correctly
        if let Some(added_sort) = self.buffer.get(insert_index) {
            info!(
                "Added freeform sort '{}' at buffer index {} with is_active = {}",
                glyph_name, insert_index, added_sort.is_active
            );
        }
        info!(
            "Added and activated freeform sort '{}' at position ({:.1}, {:.1})",
            glyph_name, position.x, position.y
        );
    }

    /// Calculate flow position for text sorts using same logic as cursor positioning
    pub fn get_text_sort_flow_position(
        &self,
        buffer_position: usize,
        font_metrics: &FontMetrics,
        _leading: f32,
    ) -> Option<Vec2> {
        if let Some(sort) = self.buffer.get(buffer_position) {
            if sort.layout_mode == SortLayoutMode::LTRText
                || sort.layout_mode == SortLayoutMode::RTLText
            {
                // Find the FIRST sort with matching buffer ID (the actual buffer root)
                let mut active_root_index = None;
                let mut root_position = Vec2::ZERO;

                // Look for the FIRST sort in the entire buffer that matches this sort's buffer ID
                if let Some(sort_buffer_id) = sort.buffer_id {
                    // Search forward from the beginning to find the first sort with this buffer ID
                    for i in 0..self.buffer.len() {
                        if let Some(candidate) = self.buffer.get(i) {
                            if candidate.buffer_id == Some(sort_buffer_id) {
                                // This is the first sort with matching buffer ID - it's the root
                                active_root_index = Some(i);
                                root_position = candidate.root_position;
                                warn!("🔍 ROOT MATCH: Found buffer root at buffer[{}] for buffer_id={:?}", i, sort_buffer_id);
                                println!("🔍 ROOT DEBUG: Using buffer[{}] as root at position ({:.1}, {:.1})", 
                                         i, root_position.x, root_position.y);
                                break;
                            }
                        }
                    }
                } else {
                    // Fallback for sorts without buffer ID (shouldn't happen for text sorts)
                    warn!("🔍 ROOT SEARCH: Sort at buffer[{}] has no buffer_id - this shouldn't happen for text sorts", buffer_position);
                }

                let root_index = active_root_index?;

                // If this is the root itself, return root position
                if buffer_position == root_index {
                    return Some(root_position);
                }

                // Calculate position using same logic as cursor positioning
                let mut x_offset = 0.0;
                let mut y_offset = 0.0;

                // Get font metrics for line height calculation
                let upm = font_metrics.units_per_em as f32;
                let descender = font_metrics.descender.unwrap_or(-256.0) as f32;
                let line_height = upm - descender;

                // Determine text direction from target sort's layout mode
                let target_sort = self.buffer.get(buffer_position);
                let is_rtl = target_sort
                    .map(|s| is_rtl_layout_mode(&s.layout_mode))
                    .unwrap_or(false);

                // EDGE-TO-EDGE POSITIONING using advance widths correctly
                // For RTL: Each character's RIGHT edge touches the previous character's LEFT edge
                // For LTR: Each character's LEFT edge touches the previous character's RIGHT edge

                // Position calculation for text flow
                warn!(
                    "🔍 CALCULATING POSITION for buffer[{}], buffer_len={}",
                    buffer_position,
                    self.buffer.len()
                );
                warn!("🔍 BUFFER DUMP:");
                for (i, entry) in self.buffer.iter().enumerate() {
                    if let SortKind::Glyph {
                        glyph_name,
                        advance_width,
                        ..
                    } = &entry.kind
                    {
                        warn!(
                            "🔍   buffer[{}]: '{}' advance={:.1}",
                            i, glyph_name, advance_width
                        );
                    }
                }

                if is_rtl {
                    // RTL: Position characters to flow LEFT from the root
                    // CRITICAL: In RTL, each character's RIGHT edge should be at the positioning point
                    // So we need to offset by the character's own width first
                    println!("🔍 RTL CALC: Calculating offset for buffer[{buffer_position}] from root at buffer[{root_index}]");

                    // RTL edge-to-edge positioning: Each character's RIGHT edge touches previous character's LEFT edge
                    // Step 1: Position current character so its RIGHT edge is at the correct position
                    if let Some(current_sort) = self.buffer.get(buffer_position) {
                        if let SortKind::Glyph {
                            advance_width: current_advance,
                            glyph_name: current_glyph,
                            ..
                        } = &current_sort.kind
                        {
                            if buffer_position == root_index {
                                // First character in buffer: RIGHT edge at root position
                                x_offset = -current_advance;
                                println!("🔍 RTL CALC: First character '{current_glyph}' advance={current_advance:.1}, RIGHT edge at root, offset={x_offset:.1}");
                            } else {
                                // Subsequent characters: RIGHT edge at previous character's LEFT edge
                                // Calculate where the previous character's left edge is
                                let mut previous_left_edge = 0.0;
                                // Only count sorts that belong to the SAME buffer
                                for i in root_index..buffer_position {
                                    if let Some(sort_entry) = self.buffer.get(i) {
                                        // CRITICAL: Only accumulate if this sort belongs to the same buffer
                                        if sort_entry.buffer_id == sort.buffer_id {
                                            if let SortKind::Glyph { advance_width, .. } =
                                                &sort_entry.kind
                                            {
                                                previous_left_edge -= advance_width;
                                            }
                                        }
                                    }
                                }
                                // Position current character so its RIGHT edge is at previous LEFT edge
                                x_offset = previous_left_edge - current_advance;
                                println!("🔍 RTL CALC: Character '{current_glyph}' advance={current_advance:.1}, RIGHT edge at previous LEFT edge {previous_left_edge:.1}, offset={x_offset:.1}");
                            }
                        }
                    }
                } else {
                    // LTR: Standard positioning - accumulate advances of characters BEFORE target
                    // Collect all sorts belonging to this buffer IN ORDER
                    let mut buffer_sorts = Vec::new();
                    let mut target_index_in_buffer = None;
                    
                    for i in 0..self.buffer.len() {
                        if let Some(sort_entry) = self.buffer.get(i) {
                            if sort_entry.buffer_id == sort.buffer_id {
                                if i == buffer_position {
                                    target_index_in_buffer = Some(buffer_sorts.len());
                                }
                                buffer_sorts.push((i, sort_entry));
                            }
                        }
                    }
                    
                    // Now calculate position based on buffer-local index
                    if let Some(target_idx) = target_index_in_buffer {
                        for idx in 0..target_idx {
                            if let Some((_, sort_entry)) = buffer_sorts.get(idx) {
                                match &sort_entry.kind {
                                    SortKind::Glyph { advance_width, .. } => {
                                        x_offset += advance_width;
                                    }
                                    SortKind::LineBreak => {
                                        // Line break: reset x to start of line, move y down by line height
                                        x_offset = 0.0;
                                        y_offset -= line_height;
                                        println!("🔍 LTR LINEBREAK: Reset x_offset=0, y_offset={y_offset:.1} (line_height={line_height:.1})");
                                    }
                                }
                            }
                        }
                    }
                }

                // Return final position
                let final_pos = Vec2::new(root_position.x + x_offset, root_position.y + y_offset);
                println!(
                    "🎯 FINAL POSITION: buffer[{}] '{}' at ({:.1}, {:.1}) (x_offset={:.1})",
                    buffer_position,
                    sort.kind.glyph_name(),
                    final_pos.x,
                    final_pos.y,
                    x_offset
                );
                Some(final_pos)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Create a new text root at the specified world position
    pub fn create_text_root(&mut self, world_position: Vec2, layout_mode: SortLayoutMode) {
        self.create_text_root_with_fontir(world_position, layout_mode, None);
    }

    /// Create a new text root with FontIR access for proper advance width calculation
    pub fn create_text_root_with_fontir(
        &mut self,
        world_position: Vec2,
        layout_mode: SortLayoutMode,
        fontir_app_state: Option<&crate::core::state::FontIRAppState>,
    ) {
        // Only clear states if buffer is empty (first text root)
        if self.buffer.is_empty() {
            self.clear_all_states();
            debug!("Creating first text root: Cleared states for empty buffer");
        } else {
            debug!(
                "Text root with existing buffer: Not clearing {} existing entries",
                self.buffer.len()
            );
        }

        // Choose appropriate default glyph based on layout mode
        let (placeholder_glyph, placeholder_codepoint) = match &layout_mode {
            SortLayoutMode::RTLText => ("alef-ar".to_string(), '\u{0627}'), // Arabic Alef
            _ => ("a".to_string(), 'a'), // Latin lowercase a for LTR and Freeform
        };

        let advance_width = if let Some(fontir_state) = fontir_app_state {
            fontir_state.get_glyph_advance_width(&placeholder_glyph)
        } else {
            // Fallback to reasonable default if FontIR not available
            500.0
        };

        let buffer_id = BufferId::new();
        let text_root = SortData {
            kind: SortKind::Glyph {
                codepoint: Some(placeholder_codepoint), // Use appropriate codepoint for layout mode
                // Use a visible placeholder glyph instead of empty string
                // This ensures the root has a visible outline and points for editing
                glyph_name: placeholder_glyph,
                advance_width, // Get from FontIR runtime data
            },
            is_active: true, // Automatically activate the new text root
            layout_mode: layout_mode.clone(),
            root_position: world_position,
            // For LTR text, cursor goes after the glyph (position 1)
            // For RTL text, cursor goes before the glyph (position 0)
            buffer_cursor_position: Some(match &layout_mode {
                SortLayoutMode::RTLText => 0,
                _ => 1,
            }),
            buffer_id: Some(buffer_id), // Assign unique buffer ID for isolation
        };

        // Insert at the end of the buffer
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, text_root);

        let cursor_pos = match &layout_mode {
            SortLayoutMode::RTLText => 0,
            _ => 1,
        };
        info!("Created and activated new {} text root at world position ({:.1}, {:.1}), cursor at position {}", 
              match layout_mode {
                  SortLayoutMode::LTRText => "LTR",
                  SortLayoutMode::RTLText => "RTL",
                  SortLayoutMode::Freeform => "Freeform",
              },
              world_position.x, world_position.y, cursor_pos);

        // Verify it was inserted correctly
        if let Some(inserted_root) = self.buffer.get(insert_index) {
            info!(
                "Verified inserted root at index {}: is_active={}",
                insert_index, inserted_root.is_active
            );
        }
    }

    /// Create a text sort at a specific world position (for text tool)
    pub fn create_text_sort_at_position(
        &mut self,
        glyph_name: String,
        world_position: Vec2,
        advance_width: f32,
        layout_mode: SortLayoutMode,
        codepoint: Option<char>,
    ) {
        // Only create a new root if there are no buffer roots yet
        if self.find_active_buffer_root_index().is_none() {
            // FIXED: Use the actual click position for the text root
            self.create_text_root(world_position, layout_mode);
        }
        // After root is created, insert the glyph at the cursor
        self.insert_sort_at_cursor(glyph_name, advance_width, codepoint);
    }

    /// Get the visual position for a sort based on its layout mode
    pub fn get_sort_visual_position(&self, buffer_position: usize) -> Option<Vec2> {
        if let Some(sort) = self.buffer.get(buffer_position) {
            match sort.layout_mode {
                SortLayoutMode::LTRText | SortLayoutMode::RTLText => {
                    // Text sorts now use their stored root_position
                    // But we need to calculate relative positions for text flow
                    // All text sorts use their flow position calculation
                    {
                        // Non-root text sorts flow from their text root
                        self.get_text_sort_flow_position(
                            buffer_position,
                            &FontMetrics::default(),
                            0.0,
                        )
                    }
                }
                SortLayoutMode::Freeform => Some(sort.root_position),
            }
        } else {
            None
        }
    }

    /// Find a sort handle at a given world position (for freeform sorts)
    pub fn find_sort_handle_at_position(
        &self,
        world_position: Vec2,
        tolerance: f32,
        font_metrics: Option<&FontMetrics>,
    ) -> Option<usize> {
        // Check handles for all sorts (both buffer and freeform have handles)
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if let Some(sort_pos) = self.get_sort_visual_position(i) {
                    let descender = if let Some(metrics) = font_metrics {
                        metrics.descender.unwrap_or(-200.0) as f32
                    } else {
                        -200.0 // Default descender value
                    };
                    // Match the rendering logic exactly: handle_position = world_pos + Vec2::new(0.0, descender)
                    let handle_pos = sort_pos + Vec2::new(0.0, descender);
                    let distance = world_position.distance(handle_pos);

                    debug!(
                        "Checking sort '{}' at index {}: sort_pos=({:.1}, {:.1}), handle_pos=({:.1}, {:.1}), click_pos=({:.1}, {:.1}), distance={:.1}, tolerance={:.1}",
                        sort.kind.glyph_name(), i, sort_pos.x, sort_pos.y, handle_pos.x, handle_pos.y, world_position.x, world_position.y, distance, tolerance
                    );

                    if distance < tolerance {
                        debug!("Found matching handle for sort '{}' at index {} (distance={:.1} < tolerance={:.1})", sort.kind.glyph_name(), i, distance, tolerance);
                        return Some(i);
                    }
                }
            }
        }
        debug!(
            "No handle found at position ({:.1}, {:.1}) with tolerance {:.1}",
            world_position.x, world_position.y, tolerance
        );
        None
    }

    /// Find a sort body at a given world position
    pub fn find_sort_body_at_position(
        &self,
        world_position: Vec2,
        tolerance: f32,
    ) -> Option<usize> {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if let Some(sort_pos) = self.get_sort_visual_position(i) {
                    if world_position.distance(sort_pos) < tolerance {
                        debug!(
                            "Found matching body for sort {} at index {}",
                            sort.kind.glyph_name(),
                            i
                        );
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    /// Get the sort at a specific buffer position
    pub fn get_sort_at_position(&self, position: usize) -> Option<&SortData> {
        self.buffer.get(position)
    }

    /// Get the currently active sort
    pub fn get_active_sort(&self) -> Option<(usize, &SortData)> {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_active {
                    return Some((i, sort));
                }
            }
        }
        None
    }

    /// Activate a sort at the given buffer position (only one can be active)
    pub fn activate_sort(&mut self, position: usize) -> bool {
        // First deactivate ALL sorts (including buffer roots) to ensure only one is active at a time
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                // Deactivate all sorts - buffer roots can be inactive for filled rendering
                sort.is_active = false;
            }
        }

        // Then activate the specified sort
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_active = true;
            info!(
                "🔥 [activate_sort] Activated sort '{}' at buffer position {}",
                sort.kind.glyph_name(),
                position
            );
            true
        } else {
            false
        }
    }

    /// Clear active state from all sorts
    pub fn clear_active_state(&mut self) {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_active = false;
            }
        }
        debug!("Cleared active state from all sorts");
    }

    /// Clear both active state and selections from all sorts
    pub fn clear_all_states(&mut self) {
        debug!(
            "Clear all states: Called with buffer length {}",
            self.buffer.len()
        );
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_active = false;
            }
        }
        debug!("Cleared all active states and selections from all sorts");
    }

    /// Get the visual position (world coordinates) for a buffer position
    pub fn get_world_position_for_buffer_position(&self, buffer_position: usize) -> Vec2 {
        let row = buffer_position / self.grid_config.sorts_per_row;
        let col = buffer_position % self.grid_config.sorts_per_row;

        let x = col as f32 * (1000.0 + self.grid_config.horizontal_spacing);
        let y = -(row as f32) * (1200.0 + self.grid_config.vertical_spacing);

        self.grid_config.grid_origin + Vec2::new(x, y)
    }

    /// Get the buffer position for a world coordinate (for click detection)
    pub fn get_buffer_position_for_world_position(&self, world_pos: Vec2) -> Option<usize> {
        let relative_pos = world_pos - self.grid_config.grid_origin;

        // Calculate grid row and column
        let col =
            (relative_pos.x / (1000.0 + self.grid_config.horizontal_spacing)).floor() as usize;

        // Handle negative Y coordinates correctly for downward-growing grid
        let row = if relative_pos.y <= 0.0 {
            ((-relative_pos.y) / (1200.0 + self.grid_config.vertical_spacing)).floor() as usize
        } else {
            0
        };

        // Convert grid position to buffer position
        let buffer_position = row * self.grid_config.sorts_per_row + col;

        // Validate the position is within bounds
        if buffer_position < self.buffer.len() {
            Some(buffer_position)
        } else {
            None
        }
    }

    /// Insert a new sort at the cursor position (for typing)
    pub fn insert_sort_at_cursor(
        &mut self,
        glyph_name: String,
        advance_width: f32,
        codepoint: Option<char>,
    ) {
        self.insert_sort_at_cursor_with_respawn(glyph_name, advance_width, codepoint, None);
    }

    /// Insert a new sort at the cursor position with optional respawn queue for entity management
    pub fn insert_sort_at_cursor_with_respawn(
        &mut self,
        glyph_name: String,
        advance_width: f32,
        codepoint: Option<char>,
        _respawn_queue: Option<
            &mut crate::systems::sorts::sort_entities::BufferSortRespawnQueue,
        >,
    ) {
        debug!("Insert at cursor: Starting insertion of '{}'", glyph_name);
        info!(
            "insert_sort_at_cursor called with glyph '{}', advance_width {}",
            glyph_name, advance_width
        );

        if let Some(root_index) = self.find_active_buffer_root_index() {
            debug!(
                "Insert at cursor: Found active root at index {}",
                root_index
            );
            let cursor_pos_in_buffer = self
                .buffer
                .get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);

            warn!(
                "🔍 INSERT DEBUG: glyph='{}', cursor_pos={}, root_index={}, buffer_len={}",
                glyph_name,
                cursor_pos_in_buffer,
                root_index,
                self.buffer.len()
            );

            // Check if this is the first character being typed into an empty text root
            let _is_empty_root = if let Some(root_sort) = self.buffer.get(root_index) {
                match &root_sort.kind {
                    SortKind::Glyph {
                        glyph_name: root_glyph,
                        ..
                    } => {
                        // Check if root is still a placeholder glyph
                        let is_placeholder = root_glyph == "a" || root_glyph == "alef-ar";
                        warn!("🔍 PLACEHOLDER CHECK: root_glyph='{}', is_placeholder={}, layout_mode={:?}", 
                              root_glyph, is_placeholder, root_sort.layout_mode);
                        is_placeholder
                    }
                    _ => false,
                }
            } else {
                false
            };

            // For RTL text: always insert new characters, never replace the root
            // The root (like alef-ar) stays as the foundation of the RTL text buffer

            // Get the layout mode and buffer ID from the buffer root
            let (root_layout_mode, root_buffer_id) = self
                .buffer
                .get(root_index)
                .map(|sort| (sort.layout_mode.clone(), sort.buffer_id))
                .unwrap_or((SortLayoutMode::LTRText, None));

            let new_sort = SortData {
                kind: SortKind::Glyph {
                    codepoint,
                    glyph_name: glyph_name.clone(),
                    advance_width,
                },
                is_active: false, // Don't make new sorts active by default
                layout_mode: root_layout_mode.clone(),
                root_position: Vec2::ZERO, // Will be calculated by flow
                buffer_cursor_position: None,
                buffer_id: root_buffer_id, // CRITICAL: Inherit buffer ID from root for isolation
            };

            // NEVER replace the root entity - always insert as a separate entity
            // This preserves the root entity for consistent rendering

            // Insert at the cursor position relative to root
            // cursor_pos_in_buffer=0 means "after root", cursor_pos_in_buffer=1 means "after first character", etc.
            let insert_buffer_index = root_index + cursor_pos_in_buffer + 1;

            debug!(
                "Inserting: Before insert - buffer has {} entries",
                self.buffer.len()
            );
            debug!(
                "Inserting: Inserting at index {} (mode: {:?}, root at {})",
                insert_buffer_index, root_layout_mode, root_index
            );

            self.buffer.insert(insert_buffer_index, new_sort);
            debug!(
                "Inserting: Insert successful - buffer now has {} entries",
                self.buffer.len()
            );
            info!(
                "🔤 Inserted character '{}' at buffer index {} (root stays at {})",
                glyph_name, insert_buffer_index, root_index
            );
            info!(
                "🔤 Buffer now has {} entries after insertion",
                self.buffer.len()
            );
            info!(
                "🔤 Layout mode: {:?}, insertion was RTL: {}",
                root_layout_mode,
                root_layout_mode == SortLayoutMode::RTLText
            );

            // Update the cursor position in the root to point after the inserted character
            // Count how many text sorts are in this sequence after insertion
            let mut text_sort_count = 0;
            for i in (root_index + 1)..self.buffer.len() {
                if let Some(sort) = self.buffer.get(i) {
                    // Continue counting all text sorts in this buffer
                    // Count text sorts in this sequence
                    if sort.layout_mode == SortLayoutMode::LTRText
                        || sort.layout_mode == SortLayoutMode::RTLText
                    {
                        text_sort_count += 1;
                    }
                }
            }

            // After insertion, cursor moves one position forward - same for RTL and LTR
            let new_cursor_pos = cursor_pos_in_buffer + 1;

            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(new_cursor_pos);
                debug!(
                    "Inserting: Updated cursor position to {} (RTL: {}, text_sorts: {})",
                    new_cursor_pos,
                    is_rtl_layout_mode(&root_layout_mode),
                    text_sort_count
                );
                info!("📍 Updated root cursor position to {} (mode: {:?}, text sorts: {}, insert_pos: {})", 
                      new_cursor_pos, root_layout_mode, text_sort_count, insert_buffer_index);

                // CRITICAL: Keep the root active so it maintains its outline
                info!(
                    "🔥 Root sort '{}' remains active with is_active={}",
                    root_sort.kind.glyph_name(),
                    root_sort.is_active
                );
            }

            // DEBUG: Show current buffer state after insertion (moved outside borrow)
            info!("🔍 BUFFER STATE after insertion:");
            for (i, entry) in self.buffer.iter().enumerate() {
                if let SortKind::Glyph {
                    glyph_name,
                    advance_width,
                    ..
                } = &entry.kind
                {
                    info!(
                        "  Buffer[{}]: glyph='{}', advance_width={:.1}, layout={:?}",
                        i, glyph_name, advance_width, entry.layout_mode
                    );
                }
            }
        } else {
            // No active text buffer, so create a new one with this character.
            debug!("Insert at cursor: NO ACTIVE ROOT FOUND - will create new text root");
            debug!(
                "Insert at cursor: Buffer has {} entries but no active root found",
                self.buffer.len()
            );
            info!(
                "No active buffer root found, creating new text root with glyph '{}'",
                glyph_name
            );
            // FIXED: Use a reasonable default position instead of Vec2::ZERO
            let default_position = Vec2::new(500.0, 0.0);
            self.create_text_root_with_glyph(
                glyph_name,
                advance_width,
                default_position,
                codepoint,
            );
        }
    }

    /// Delete the sort at the cursor position
    pub fn delete_sort_at_cursor(&mut self) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let _cursor_pos_in_buffer = self
                .buffer
                .get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);

            // Find the actual last character to delete (should be the last non-root entry in the buffer)
            // Since characters are always inserted at the end, delete the last character
            let buffer_len = self.buffer.len();
            if buffer_len > 1 {
                // Must have more than just the root
                let delete_buffer_index = buffer_len - 1; // Delete the last character

                info!(
                    "🗑️ Deleting character at buffer index {} (buffer length: {})",
                    delete_buffer_index, buffer_len
                );

                // Delete the character from the buffer
                let deleted_sort = self.buffer.delete(delete_buffer_index);
                if let Some(deleted) = deleted_sort {
                    info!(
                        "🗑️ Successfully deleted sort: glyph='{}'",
                        deleted.kind.glyph_name()
                    );
                }

                info!("🗑️ Buffer length after deletion: {}", self.buffer.len());

                // Update cursor position in the root
                // Since we're deleting from the end of the buffer, we need to count
                // how many actual text sorts (non-root) remain after deletion
                let mut text_sort_count = 0;
                for i in (root_index + 1)..self.buffer.len() {
                    if let Some(sort) = self.buffer.get(i) {
                        // Continue counting all text sorts in this buffer
                        // Count text sorts in this sequence
                        if sort.layout_mode == SortLayoutMode::LTRText
                            || sort.layout_mode == SortLayoutMode::RTLText
                        {
                            text_sort_count += 1;
                        }
                    }
                }

                // Cursor position should be after all remaining text sorts
                // Position 0 = before root, Position 1 = after root, Position 2 = after first character, etc.
                // So cursor position = text_sort_count + 1 (to account for the root)
                let new_cursor_pos = text_sort_count + 1;
                if let Some(root_sort) = self.buffer.get_mut(root_index) {
                    root_sort.buffer_cursor_position = Some(new_cursor_pos);
                    info!(
                        "📍 Updated cursor position to {} (text sorts remaining: {}, +1 for root)",
                        new_cursor_pos, text_sort_count
                    );
                }
            } else {
                // Only root remains - delete the entire buffer root and clear the text buffer
                info!("🗑️ Deleting root sort - clearing entire text buffer");
                let deleted_root = self.buffer.delete(root_index);
                if let Some(deleted) = deleted_root {
                    info!(
                        "🗑️ Successfully deleted root sort: glyph='{}'",
                        deleted.kind.glyph_name()
                    );
                }
                info!(
                    "🗑️ Text buffer cleared - buffer length after root deletion: {}",
                    self.buffer.len()
                );
            }
        } else {
            info!("🗑️ Cannot delete - no active buffer root found");
        }
    }

    /// Move cursor to a specific position (now works with per-buffer-root cursors)
    pub fn move_cursor_to(&mut self, position: usize) {
        // FIXED: Update the active buffer root's cursor position instead of global cursor
        if let Some(root_index) = self.find_active_buffer_root_index() {
            // Get max_pos before getting mutable reference to avoid borrow conflicts
            let max_pos = self.get_buffer_sequence_length(root_index);
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                // Clamp the position to valid range for this buffer sequence
                let clamped_position = position.min(max_pos);
                root_sort.buffer_cursor_position = Some(clamped_position);
                info!(
                    "Moved cursor to position {} in buffer root at index {}",
                    clamped_position, root_index
                );
            }
        }
    }

    /// Move cursor left by one position
    pub fn move_cursor_left(&mut self) {
        // FIXED: Update the active buffer root's cursor position
        if let Some(root_index) = self.find_active_buffer_root_index() {
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                let current_pos = root_sort.buffer_cursor_position.unwrap_or(0);
                if current_pos > 0 {
                    root_sort.buffer_cursor_position = Some(current_pos - 1);
                    info!(
                        "Moved cursor left to position {} in buffer root at index {}",
                        current_pos - 1,
                        root_index
                    );
                }
            }
        }
    }

    /// Move cursor right by one position
    pub fn move_cursor_right(&mut self) {
        // FIXED: Update the active buffer root's cursor position
        if let Some(root_index) = self.find_active_buffer_root_index() {
            // Get max_pos before getting mutable reference to avoid borrow conflicts
            let max_pos = self.get_buffer_sequence_length(root_index);
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                let current_pos = root_sort.buffer_cursor_position.unwrap_or(0);
                if current_pos < max_pos {
                    root_sort.buffer_cursor_position = Some(current_pos + 1);
                    info!(
                        "Moved cursor right to position {} in buffer root at index {}",
                        current_pos + 1,
                        root_index
                    );
                }
            }
        }
    }

    /// Move cursor up by one row (for buffer mode, move to previous buffer root)
    pub fn move_cursor_up(&mut self) {
        // FIXED: In buffer mode, move to previous buffer root
        if let Some(current_root_index) = self.find_active_buffer_root_index() {
            // Find the previous buffer by checking buffer_id
            for i in (0..current_root_index).rev() {
                if let Some(sort) = self.buffer.get(i) {
                    // Check if this sort belongs to a different buffer
                    if sort.buffer_id.is_some() {
                        // Deselect current buffer root and select previous one
                        if let Some(current_root) = self.buffer.get_mut(current_root_index) {
                            current_root.is_active = false;
                        }
                        // Get buffer_length before getting mutable reference to avoid borrow conflicts
                        let buffer_length = self.get_buffer_sequence_length(i);
                        if let Some(prev_root) = self.buffer.get_mut(i) {
                            prev_root.is_active = true;
                            // Set cursor to end of previous buffer
                            prev_root.buffer_cursor_position = Some(buffer_length);
                            info!(
                                "Moved up to buffer root at index {}, cursor at position {}",
                                i, buffer_length
                            );
                        }
                        return;
                    }
                }
            }
        }
    }

    /// Move cursor down by one row (for buffer mode, move to next buffer root)
    pub fn move_cursor_down(&mut self) {
        // FIXED: In buffer mode, move to next buffer root
        if let Some(current_root_index) = self.find_active_buffer_root_index() {
            // Find the next buffer by checking buffer_id
            for i in (current_root_index + 1)..self.buffer.len() {
                if let Some(sort) = self.buffer.get(i) {
                    // Check if this sort belongs to a different buffer
                    if sort.buffer_id.is_some() {
                        // Deselect current buffer root and select next one
                        if let Some(current_root) = self.buffer.get_mut(current_root_index) {
                            current_root.is_active = false;
                        }
                        if let Some(next_root) = self.buffer.get_mut(i) {
                            next_root.is_active = true;
                            // Set cursor to beginning of next buffer
                            next_root.buffer_cursor_position = Some(0);
                            info!(
                                "Moved down to buffer root at index {}, cursor at position 0",
                                i
                            );
                        }
                        return;
                    }
                }
            }
        }
    }

    /// Helper: Find the index of the active buffer root
    fn find_active_buffer_root_index(&self) -> Option<usize> {
        debug!(
            "Find active sort: Searching for active sort in {} buffer entries",
            self.buffer.len()
        );
        
        // Find any active sort
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_active {
                    debug!("Found active sort at index {}", i);
                    return Some(i);
                }
            }
        }

        // Find sort with cursor position
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.buffer_cursor_position.is_some() {
                    return Some(i);
                }
            }
        }

        // Return first sort as fallback
        if !self.buffer.is_empty() {
            return Some(0);
        }

        None
    }

    /// Helper: Get the length of a buffer sequence starting from a buffer root
    fn get_buffer_sequence_length(&self, root_index: usize) -> usize {
        let mut length = 0;
        for i in root_index..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                // A text sequence ends when we hit a non-text sort.
                if sort.layout_mode != SortLayoutMode::LTRText
                    && sort.layout_mode != SortLayoutMode::RTLText
                {
                    break;
                }

                // The root placeholder doesn't count towards the string's length.
                if i == root_index
                    && sort.kind.is_glyph()
                    && sort.kind.glyph_name() == "a"
                {
                    continue;
                }

                length += 1;
            } else {
                // End of the main buffer.
                break;
            }
        }
        length
    }

    pub fn create_text_root_with_glyph(
        &mut self,
        glyph_name: String,
        advance_width: f32,
        world_position: Vec2,
        codepoint: Option<char>,
    ) {
        // FIXED: Use the provided position instead of hardcoded position
        self.clear_all_states();

        let buffer_id = BufferId::new();
        let new_root = SortData {
            kind: SortKind::Glyph {
                codepoint,
                glyph_name,
                advance_width,
            },
            is_active: true,
            layout_mode: SortLayoutMode::LTRText,
            root_position: world_position,
            buffer_cursor_position: Some(1), // Cursor is after the typed character.
            buffer_id: Some(buffer_id),      // Assign unique buffer ID
        };

        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, new_root);
    }

    /// Insert a line break at the cursor position (for Enter key)
    pub fn insert_line_break_at_cursor(&mut self) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            debug!(
                "Insert line break: Found active root at index {}",
                root_index
            );

            // Get the layout mode from the buffer root
            let (root_layout_mode, root_buffer_id) = self
                .buffer
                .get(root_index)
                .map(|sort| (sort.layout_mode.clone(), sort.buffer_id))
                .unwrap_or((SortLayoutMode::LTRText, None));

            let new_sort = SortData {
                kind: SortKind::LineBreak,
                is_active: false,
                layout_mode: root_layout_mode,
                root_position: Vec2::ZERO,
                buffer_cursor_position: None,
                buffer_id: root_buffer_id, // Inherit buffer ID from root
            };

            // FIXED: Insert at the end of the buffer instead of using cursor position
            // The cursor position was getting out of sync with the actual buffer
            let insert_buffer_index = self.buffer.len();
            debug!(
                "Insert line break: Inserting at index {} (end of buffer)",
                insert_buffer_index
            );

            self.buffer.insert(insert_buffer_index, new_sort);
            debug!(
                "Insert line break: Insert successful - buffer now has {} entries",
                self.buffer.len()
            );
            info!(
                "🔤 Inserted line break at buffer index {}",
                insert_buffer_index
            );

            // Update the cursor position in the root to point after the line break
            let new_cursor_pos = self.buffer.len(); // Cursor AFTER the line break (at the position where next character will be inserted)
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(new_cursor_pos);
                debug!(
                    "Insert line break: Updated cursor position to {} (after line break)",
                    new_cursor_pos
                );
                info!(
                    "📍 Updated root cursor position to {} after line break",
                    new_cursor_pos
                );
            }
        } else {
            debug!("Insert line break: NO ACTIVE ROOT FOUND");
            warn!("Cannot insert line break - no active buffer root found");
        }
    }

    /// Move cursor up to the previous line (multi-line aware)
    pub fn move_cursor_up_multiline(&mut self) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let cursor_pos = self
                .buffer
                .get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);
            // Build line starts and x offsets
            let mut line_starts = vec![0];
            let mut x_offsets = vec![0.0];
            let mut x = 0.0;
            for (i, entry) in self.buffer.iter().enumerate().skip(root_index + 1) {
                match &entry.kind {
                    SortKind::Glyph { advance_width, .. } => {
                        x += *advance_width;
                        x_offsets.push(x);
                    }
                    SortKind::LineBreak => {
                        line_starts.push(i - root_index);
                        x = 0.0;
                        x_offsets.push(x);
                    }
                }
            }
            // Find current line
            let mut current_line = 0;
            for (i, &start) in line_starts.iter().enumerate() {
                if cursor_pos < start {
                    break;
                }
                current_line = i;
            }
            if current_line == 0 {
                return; // Already at top line
            }
            let prev_line_start = line_starts[current_line - 1];
            let curr_x = x_offsets.get(cursor_pos).copied().unwrap_or(0.0);
            // Find closest x in previous line
            let prev_line_end = if current_line < line_starts.len() {
                line_starts[current_line]
            } else {
                x_offsets.len()
            };
            let mut best_idx = prev_line_start;
            let mut best_dist = f32::MAX;
            for (offset_idx, &x_offset) in x_offsets
                .iter()
                .enumerate()
                .take(prev_line_end)
                .skip(prev_line_start)
            {
                let dist = (x_offset - curr_x).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = offset_idx;
                }
            }
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(best_idx);
            }
        }
    }
    /// Move cursor down to the next line (multi-line aware)
    pub fn move_cursor_down_multiline(&mut self) {
        if let Some(root_index) = self.find_active_buffer_root_index() {
            let cursor_pos = self
                .buffer
                .get(root_index)
                .and_then(|rs| rs.buffer_cursor_position)
                .unwrap_or(0);
            // Build line starts and x offsets
            let mut line_starts = vec![0];
            let mut x_offsets = vec![0.0];
            let mut x = 0.0;
            for (i, entry) in self.buffer.iter().enumerate().skip(root_index + 1) {
                match &entry.kind {
                    SortKind::Glyph { advance_width, .. } => {
                        x += *advance_width;
                        x_offsets.push(x);
                    }
                    SortKind::LineBreak => {
                        line_starts.push(i - root_index);
                        x = 0.0;
                        x_offsets.push(x);
                    }
                }
            }
            // Find current line
            let mut current_line = 0;
            for (i, &start) in line_starts.iter().enumerate() {
                if cursor_pos < start {
                    break;
                }
                current_line = i;
            }
            if current_line + 1 >= line_starts.len() {
                return; // Already at last line
            }
            let next_line_start = line_starts[current_line + 1];
            let next_line_end = if current_line + 2 < line_starts.len() {
                line_starts[current_line + 2]
            } else {
                x_offsets.len()
            };
            let curr_x = x_offsets.get(cursor_pos).copied().unwrap_or(0.0);
            // Find closest x in next line
            let mut best_idx = next_line_start;
            let mut best_dist = f32::MAX;
            for (offset_idx, &x_offset) in x_offsets
                .iter()
                .enumerate()
                .take(next_line_end)
                .skip(next_line_start)
            {
                let dist = (x_offset - curr_x).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = offset_idx;
                }
            }
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(best_idx);
            }
        }
    }
}

/// Helper function to check if a layout mode is right-to-left
fn is_rtl_layout_mode(layout_mode: &SortLayoutMode) -> bool {
    matches!(layout_mode, SortLayoutMode::RTLText)
}

/// Helper function to calculate cursor position based on text direction
fn _calculate_cursor_position_for_direction(
    layout_mode: &SortLayoutMode,
    text_sort_count: usize,
) -> usize {
    if is_rtl_layout_mode(layout_mode) {
        // For RTL: cursor should be at the insertion position (where next character will go)
        // Since RTL characters are inserted immediately after the root, the cursor stays at root_index + 1
        // This represents the left edge of the rightmost character in the RTL sequence
        1 // Always position 1 (immediately after root) for RTL continuous typing
    } else {
        // For LTR: cursor moves after all text sorts
        // Position 0 = before root, Position 1 = after root, Position 2 = after first character, etc.
        // So cursor position = text_sort_count + 1 (to account for the root)
        text_sort_count + 1
    }
}

/// Helper function to calculate insertion position based on text direction
fn _calculate_insertion_position_for_direction(
    layout_mode: &SortLayoutMode,
    root_index: usize,
    buffer_len: usize,
) -> usize {
    if is_rtl_layout_mode(layout_mode) {
        // For RTL: insert immediately after root
        root_index + 1
    } else {
        // For LTR: insert at end of buffer
        buffer_len
    }
}

/// Helper function to get appropriate default glyph and codepoint for text direction
pub fn get_default_glyph_for_direction(layout_mode: &SortLayoutMode) -> (String, char) {
    if is_rtl_layout_mode(layout_mode) {
        ("alef-ar".to_string(), '\u{0627}') // Arabic Alef
    } else {
        ("a".to_string(), 'a') // Latin lowercase a for LTR and Freeform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_activation_on_creation() {
        let mut text_editor = TextEditorState::default();

        // Test 1: Freeform sort should be activated when created
        text_editor.add_freeform_sort("a".to_string(), Vec2::new(100.0, 200.0), 500.0, Some('a'));

        // Verify the sort was created and activated
        assert_eq!(text_editor.buffer.len(), 1);
        if let Some(sort) = text_editor.buffer.get(0) {
            assert!(sort.is_active);
            assert_eq!(sort.kind.glyph_name(), "a");
            assert_eq!(sort.root_position, Vec2::new(100.0, 200.0));
        } else {
            panic!("Sort should exist at index 0");
        }

        // Test 2: Text sort should be activated when created
        text_editor.create_text_sort_at_position(
            "b".to_string(),
            Vec2::new(300.0, 400.0),
            600.0,
            SortLayoutMode::LTRText,
            Some('b'),
        );

        // Verify the new sort was created and activated, and the old one was deactivated
        assert_eq!(text_editor.buffer.len(), 2);

        // First sort should be deactivated
        if let Some(sort) = text_editor.buffer.get(0) {
            assert!(!sort.is_active);
        }

        // Second sort should be activated
        if let Some(sort) = text_editor.buffer.get(1) {
            assert!(sort.is_active);
            assert_eq!(sort.kind.glyph_name(), "b");
            assert_eq!(sort.root_position, Vec2::new(300.0, 400.0));
        } else {
            panic!("Sort should exist at index 1");
        }

        // Test 3: Text root should be activated when created
        text_editor.create_text_root(Vec2::new(500.0, 600.0), SortLayoutMode::LTRText);

        // Verify the new text root was created and activated, and others were deactivated
        assert_eq!(text_editor.buffer.len(), 3);

        // First two sorts should be deactivated
        for i in 0..2 {
            if let Some(sort) = text_editor.buffer.get(i) {
                assert!(!sort.is_active);
            }
        }

        // Third sort (text root) should be activated
        if let Some(sort) = text_editor.buffer.get(2) {
            assert!(sort.is_active);
            assert_eq!(sort.root_position, Vec2::new(500.0, 600.0));
        } else {
            panic!("Text root should exist at index 2");
        }
    }

    #[test]
    fn test_backspace_functionality() {
        let mut text_editor = TextEditorState::default();

        // Create a text root
        text_editor.create_text_root(Vec2::new(100.0, 200.0), SortLayoutMode::LTRText);
        assert_eq!(text_editor.buffer.len(), 1); // Should have root

        // Insert some characters
        text_editor.insert_sort_at_cursor("h".to_string(), 100.0, Some('h'));
        text_editor.insert_sort_at_cursor("e".to_string(), 100.0, Some('e'));
        text_editor.insert_sort_at_cursor("l".to_string(), 100.0, Some('l'));
        text_editor.insert_sort_at_cursor("l".to_string(), 100.0, Some('l'));
        text_editor.insert_sort_at_cursor("o".to_string(), 100.0, Some('o'));
        assert_eq!(text_editor.buffer.len(), 6); // Root + 5 characters

        // Test backspace - should delete characters properly
        text_editor.delete_sort_at_cursor(); // Delete 'o'
        assert_eq!(text_editor.buffer.len(), 5); // Root + 4 characters

        text_editor.delete_sort_at_cursor(); // Delete 'l'
        assert_eq!(text_editor.buffer.len(), 4); // Root + 3 characters

        text_editor.delete_sort_at_cursor(); // Delete 'l'
        assert_eq!(text_editor.buffer.len(), 3); // Root + 2 characters

        text_editor.delete_sort_at_cursor(); // Delete 'e'
        assert_eq!(text_editor.buffer.len(), 2); // Root + 1 character

        text_editor.delete_sort_at_cursor(); // Delete 'h'
        assert_eq!(text_editor.buffer.len(), 1); // Just root left

        // Delete the root - should clear the entire buffer
        text_editor.delete_sort_at_cursor();
        assert_eq!(text_editor.buffer.len(), 0); // Buffer should be empty
    }

    #[test]
    fn test_text_flow_calculation() {
        let mut text_editor = TextEditorState::default();

        // Create a text root at position (100, 200)
        text_editor.create_text_root(Vec2::new(100.0, 200.0), SortLayoutMode::LTRText);
        println!(
            "After create_text_root: buffer length = {}",
            text_editor.buffer.len()
        );

        // Insert some glyphs with known advance widths
        text_editor.insert_sort_at_cursor("a".to_string(), 100.0, Some('a'));
        println!(
            "After inserting 'a': buffer length = {}",
            text_editor.buffer.len()
        );
        text_editor.insert_sort_at_cursor("b".to_string(), 150.0, Some('b'));
        println!(
            "After inserting 'b': buffer length = {}",
            text_editor.buffer.len()
        );
        text_editor.insert_sort_at_cursor("c".to_string(), 120.0, Some('c'));
        println!(
            "After inserting 'c': buffer length = {}",
            text_editor.buffer.len()
        );

        // Print buffer contents
        println!("\nBuffer contents:");
        for (i, sort) in text_editor.buffer.iter().enumerate() {
            println!(
                "  [{}] '{}' (active: {}) at ({:.1}, {:.1})",
                i,
                sort.kind.glyph_name(),
                sort.is_active,
                sort.root_position.x,
                sort.root_position.y
            );
        }

        // Verify the text flow positions
        let font_metrics = FontMetrics::default();

        // Root (placeholder) is at index 0
        if let Some(pos) = text_editor.get_text_sort_flow_position(0, &font_metrics, 0.0) {
            println!(
                "Index 0 (root): calculated position = ({:.1}, {:.1})",
                pos.x, pos.y
            );
            assert_eq!(pos, Vec2::new(100.0, 200.0));
        } else {
            panic!("Should have flow position for root");
        }
        // First glyph after root is at index 1
        if let Some(pos) = text_editor.get_text_sort_flow_position(1, &font_metrics, 0.0) {
            println!(
                "Index 1 (first glyph): calculated position = ({:.1}, {:.1})",
                pos.x, pos.y
            );
            assert_eq!(pos, Vec2::new(200.0, 200.0)); // 100 + 100
        } else {
            panic!("Should have flow position for first glyph");
        }
        // Second glyph after root is at index 2
        if let Some(pos) = text_editor.get_text_sort_flow_position(2, &font_metrics, 0.0) {
            println!(
                "Index 2 (second glyph): calculated position = ({:.1}, {:.1})",
                pos.x, pos.y
            );
            assert_eq!(pos, Vec2::new(350.0, 200.0)); // 100 + 100 + 150
        } else {
            panic!("Should have flow position for second glyph");
        }
        // Third glyph after root is at index 2
        if let Some(pos) = text_editor.get_text_sort_flow_position(2, &font_metrics, 0.0) {
            println!(
                "Index 2 (third glyph): calculated position = ({:.1}, {:.1})",
                pos.x, pos.y
            );
            assert_eq!(pos, Vec2::new(350.0, 200.0)); // 100 + 100 + 150
        } else {
            panic!("Should have flow position for third glyph");
        }
    }
}
