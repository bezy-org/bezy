# TUI Implementation Plan for Bezy Font Editor

## Overview

This document outlines the plan to add a Terminal User Interface (TUI) to the Bezy font editor using ratatui. The TUI will run in the terminal when launching the application and provide tabs for different views, with log redirection to `.config/logs`.

## Goals

1. **TUI Interface**: Create a terminal-based interface that runs alongside the main Bevy application
2. **Tab System**: Implement multiple tabs for different views (glyphs, font info, logs, etc.)
3. **Glyph Navigation**: Allow selecting glyphs by Unicode codepoint from the TUI
4. **Log Management**: Redirect logs to `.config/logs` directory for better organization
5. **Bidirectional Communication**: Enable TUI actions to affect the main editor state

## Architecture

### Application Structure

```
src/
├── tui/                     # New TUI module
│   ├── mod.rs              # TUI module entry point
│   ├── app.rs              # TUI application state and logic
│   ├── ui.rs               # UI rendering functions
│   ├── events.rs           # Event handling and input
│   ├── tabs/               # Individual tab implementations
│   │   ├── mod.rs
│   │   ├── glyphs.rs       # Glyph browser tab
│   │   ├── font_info.rs    # Font metadata tab
│   │   ├── logs.rs         # Log viewer tab
│   │   └── help.rs         # Help/shortcuts tab
│   └── communication.rs    # Channel-based communication with main app
```

### Communication Design

The TUI will run in a separate thread and communicate with the main Bevy application through channels:

```rust
// Shared between TUI and main app
pub enum TuiMessage {
    SelectGlyph(String),        // Select glyph by codepoint
    ChangeZoom(f32),           // Adjust viewport zoom
    ShowGlyphInfo(String),     // Request glyph information
    // More commands as needed
}

pub enum AppMessage {
    CurrentGlyph(String),      // Current active glyph
    FontLoaded(FontInfo),      // Font metadata
    GlyphList(Vec<GlyphInfo>), // Available glyphs
    // More status updates
}
```

### Tab System

1. **Glyphs Tab**:
   - Lists all Unicode codepoints in the font
   - Shows glyph names and preview characters
   - Allows selection to change active glyph in editor
   - Search/filter functionality

2. **Font Info Tab**:
   - Display font metadata (name, version, etc.)
   - Show font metrics (ascender, descender, etc.)
   - Family and style information

3. **Logs Tab**:
   - Real-time log viewer
   - Filter by log level
   - Search within logs

4. **Help Tab**:
   - Keyboard shortcuts
   - Usage instructions
   - Quick reference

## Implementation Plan

### Phase 1: Basic Infrastructure

1. **Add Dependencies**:
   ```toml
   ratatui = "0.27"
   crossterm = "0.27"
   tokio = { version = "1.0", features = ["full"] }
   ```

2. **Create TUI Module Structure**:
   - Basic app state
   - Simple event loop
   - Terminal setup/cleanup

3. **CLI Integration**:
   - Add `--tui` flag to enable TUI mode
   - Modify main.rs to spawn TUI thread when enabled

### Phase 2: Tab System Implementation

1. **Create Tab Infrastructure**:
   - Tab enum and state management
   - Tab switching logic (Ctrl+Tab, number keys)
   - Basic tab rendering

2. **Implement First Tab (Glyphs)**:
   - List view of available glyphs
   - Basic navigation (arrow keys, page up/down)
   - Selection highlighting

### Phase 3: Communication Layer

1. **Channel Setup**:
   - MPSC channels for bidirectional communication
   - Message types for common operations
   - Error handling for channel communication

2. **State Synchronization**:
   - Font loading updates
   - Current glyph tracking
   - Editor state changes

### Phase 4: Advanced Features

1. **Log Redirection**:
   - Create `.config/bezy/logs` directory
   - Configure tracing to write to log files
   - Real-time log viewing in TUI

2. **Enhanced Glyph Browser**:
   - Search functionality
   - Unicode block navigation
   - Glyph preview (ASCII art or block representation)

## Technical Considerations

### Threading Model

- **Main Thread**: Bevy application
- **TUI Thread**: Ratatui interface
- **Communication**: MPSC channels with non-blocking sends

### Error Handling

- TUI should gracefully handle main app disconnection
- Main app should continue if TUI crashes
- Clear error messages for communication failures

### Performance

- Minimize message frequency to avoid overwhelming channels
- Use change detection to only send updates when needed
- Implement pagination for large glyph lists

### User Experience

- Consistent keyboard shortcuts across tabs
- Context-sensitive help
- Responsive layout for different terminal sizes
- Visual feedback for state changes

## Configuration

Add TUI-specific settings to the existing config system:

```json
{
  "tui": {
    "default_tab": "glyphs",
    "log_level": "info",
    "max_log_lines": 1000,
    "auto_start": false
  }
}
```

## Future Enhancements

1. **Additional Tabs**:
   - Outline viewer/editor
   - Kerning pairs
   - Feature definitions

2. **Advanced Features**:
   - Script for common operations
   - Plugin system for custom tabs
   - Export functionality

3. **Integration**:
   - Font validation results
   - Build/export status
   - Version control integration

## Testing Strategy

1. **Unit Tests**: Individual tab logic and message handling
2. **Integration Tests**: Channel communication and state sync
3. **Manual Testing**: Keyboard navigation and visual layout
4. **Performance Tests**: Large font handling and message throughput

## Dependencies

- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal manipulation
- **tokio**: Async runtime for channel handling
- **existing dependencies**: Leverage current font handling code

This plan provides a structured approach to implementing a powerful TUI interface that enhances the Bezy font editor with terminal-based interaction while maintaining clean separation from the main application.