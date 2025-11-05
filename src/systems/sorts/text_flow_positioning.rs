use crate::core::state::text_editor::{SortData, SortKind, SortLayoutMode};
use bevy::math::Vec2;

/// Calculate text flow offset for a position within a buffer
///
/// This is the SINGLE SOURCE OF TRUTH for text positioning in both LTR and RTL modes.
/// Used by both:
/// - Sort entity visual positioning
/// - Cursor positioning
///
/// This ensures cursor and text are always perfectly aligned.
pub fn calculate_text_flow_offset(
    buffer_sorts: &[&SortData],
    target_index: usize,
    line_height: f32,
    layout_mode: &SortLayoutMode,
) -> Vec2 {
    match layout_mode {
        SortLayoutMode::RTLText => {
            calculate_rtl_offset(buffer_sorts, target_index, line_height)
        }
        _ => calculate_ltr_offset(buffer_sorts, target_index, line_height),
    }
}

/// Calculate LTR text flow offset
///
/// LTR POSITIONING LOGIC:
/// - Start at (0, 0) relative to buffer root
/// - Move RIGHT by adding advance widths for each glyph before target
/// - Line breaks reset to x=0 and move down by line_height
fn calculate_ltr_offset(
    buffer_sorts: &[&SortData],
    target_index: usize,
    line_height: f32,
) -> Vec2 {
    use bevy::prelude::warn;

    let mut x_offset = 0.0;
    let mut y_offset = 0.0;

    warn!("📐 CALCULATE_LTR_OFFSET: buffer_sorts.len()={}, target_index={}", buffer_sorts.len(), target_index);

    // Accumulate advance widths for all sorts BEFORE the target index
    for (i, sort) in buffer_sorts.iter().enumerate() {
        warn!("  📊 LTR LOOP: i={}, target_index={}, checking i >= target_index...", i, target_index);
        if i >= target_index {
            warn!("  ⛔ BREAKING: i={} >= target_index={}", i, target_index);
            break;
        }

        match &sort.kind {
            SortKind::Glyph { advance_width, glyph_name, .. } => {
                warn!("  ✅ PROCESSING GLYPH '{}' at index {}: advance_width={:.1}, accumulating to x_offset", glyph_name, i, advance_width);
                x_offset += advance_width;
                warn!("     → x_offset is now: {:.1}", x_offset);
            }
            SortKind::LineBreak => {
                warn!("  📏 LINE BREAK at index {}: resetting x_offset to 0, moving down by {:.1}", i, line_height);
                x_offset = 0.0;
                y_offset -= line_height;
            }
        }
    }

    warn!("📐 CALCULATE_LTR_OFFSET RESULT: offset=({:.1}, {:.1})", x_offset, y_offset);
    Vec2::new(x_offset, y_offset)
}

/// Calculate RTL text flow offset
///
/// RTL POSITIONING LOGIC:
/// - Start at (0, 0) relative to buffer root (which is the RIGHT edge)
/// - Move LEFT by subtracting advance widths for glyphs AT OR AFTER target
/// - Line breaks reset to x=0 and move down by line_height
fn calculate_rtl_offset(
    buffer_sorts: &[&SortData],
    target_index: usize,
    line_height: f32,
) -> Vec2 {
    let mut x_offset = 0.0;
    let mut y_offset = 0.0;

    // RTL: Process sorts AT OR AFTER target to move cursor leftward
    for (i, sort) in buffer_sorts.iter().enumerate() {
        if i < target_index {
            continue;
        }

        match &sort.kind {
            SortKind::LineBreak => {
                if i == target_index {
                    y_offset -= line_height;
                    break;
                }
            }
            SortKind::Glyph { advance_width, .. } => {
                x_offset -= advance_width;
            }
        }
    }

    Vec2::new(x_offset, y_offset)
}
