# QA Tab Implementation Plan

## Overview

The QA (Quality Assurance) tab in Bezy's TUI will provide comprehensive font quality analysis using Fontspector, giving font designers immediate feedback on potential issues in their fonts.

## Goals

1. **Real-time QA feedback** - Show quality issues for the currently loaded font
2. **Automated analysis** - Run Fontspector checks automatically when fonts change
3. **Actionable reports** - Present QA results in a format that helps designers fix issues
4. **Persistent reports** - Store QA reports for comparison and tracking over time
5. **Integration with workflow** - Seamlessly integrate with Bezy's edit/save/export cycle

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                        QA Tab                               │
├─────────────────────────────────────────────────────────────┤
│  QA Display (TUI)     │  QA Engine        │  File Storage   │
│  ├─ Issue List        │  ├─ Fontspector   │  ├─ Reports     │
│  ├─ Severity Filter   │  ├─ FontC Compile │  ├─ Temp Files  │
│  ├─ Category Filter   │  ├─ File Monitor  │  └─ Cache       │
│  ├─ Detail View       │  └─ Report Parser │                 │
│  └─ Progress Bar      │                   │                 │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

```
UFO Font File → Save Trigger → FontC Compile → TTF/OTF → Fontspector → JSON Report → TUI Display
     ↑                                                      ↓
  Save Event ← User Save Action                       Cache & Store
```

### Directory Structure

```
~/.config/bezy/qa/
├── reports/
│   ├── {font-hash}/
│   │   ├── latest.json
│   │   ├── {timestamp}.json
│   │   └── summary.json
│   └── cache/
├── temp/
│   ├── compiled/
│   │   ├── {font-hash}.ttf
│   │   └── {font-hash}.otf
│   └── working/
└── config/
    ├── fontspector-profile.json
    └── qa-settings.toml
```

## Technical Implementation

### 1. QA Engine Module (`src/qa/`)

```rust
// src/qa/mod.rs
pub mod fontspector;
pub mod compiler;
pub mod monitor;
pub mod storage;

pub struct QAEngine {
    fontspector: FontspectorRunner,
    compiler: FontCompiler,
    monitor: FileMonitor,
    storage: ReportStorage,
}

// src/qa/fontspector.rs
pub struct FontspectorRunner {
    executable_path: PathBuf,
    profile: FontspectorProfile,
}

pub struct QAReport {
    pub font_path: PathBuf,
    pub timestamp: SystemTime,
    pub issues: Vec<QAIssue>,
    pub summary: QASummary,
}

pub struct QAIssue {
    pub severity: Severity, // ERROR, WARNING, INFO
    pub category: Category, // OUTLINES, METADATA, HINTING, etc.
    pub check_id: String,
    pub message: String,
    pub location: Option<Location>, // glyph, table, etc.
}
```

### 2. TUI Integration (`src/tui/tabs/qa.rs`)

```rust
pub struct QAState {
    pub current_report: Option<QAReport>,
    pub issues: Vec<QAIssue>,
    pub selected_issue: usize,
    pub filter_severity: Option<Severity>,
    pub filter_category: Option<Category>,
    pub is_running: bool,
    pub progress: f32,
    pub scroll_offset: usize,
}

pub enum QAView {
    IssueList,
    IssueDetail,
    Summary,
    Settings,
}
```

### 3. Font Compilation Pipeline

Use FontC to compile UFO → TTF/OTF for Fontspector analysis:

```rust
// src/qa/compiler.rs
pub async fn compile_for_qa(ufo_path: &Path) -> Result<PathBuf> {
    // 1. Create temp directory for this font
    // 2. Run fontc to compile UFO to TTF
    // 3. Return path to compiled font
    // 4. Cache compiled fonts with hash-based naming
}
```

### 4. Save Event Integration

Integrate QA analysis with save operations for optimal performance:

```rust
// src/qa/trigger.rs
pub struct QASaveTrigger {
    current_font: Option<PathBuf>,
    last_analysis_hash: Option<String>,
}

// Trigger QA analysis only when:
// - Font file is explicitly saved (Ctrl+S)
// - Font is exported (Ctrl+E)
// - Manual QA refresh is requested
```

## Fontspector Integration

### Current Implementation Status

**Phase 1 Complete**: QA module structure and TUI integration have been implemented with a placeholder system.

**Integration Challenge**: The Fontspector Rust crates require a protobuf compiler (`protoc`) for compilation, which creates deployment and development environment dependencies.

**Root Cause Analysis**:
- Fontspector main crate (v1.5.0) appears to be binary-only, not providing a library interface
- Profile crates (fontspector-profile-googlefonts, fontspector-profile-universal) depend on google-fonts-axisregistry
- google-fonts-axisregistry requires protobuf compiler during build process
- This creates a system dependency that complicates deployment and CI/CD pipelines

**Current Solution**: A placeholder implementation provides realistic sample QA data to demonstrate the complete UI and workflow. The infrastructure is designed for easy replacement with actual Fontspector integration when:
1. Protobuf compiler dependencies are resolved, OR
2. Fontspector provides a library crate without protobuf build dependencies, OR
3. Alternative integration approaches become available

**Workaround Attempted**: Initially tried installing protobuf compiler but hit environment permission restrictions. Rather than create complex build dependencies, implemented placeholder system that maintains full functionality for development and testing.

### Installation Detection

```rust
// PLACEHOLDER IMPLEMENTATION - Replace when protobuf compiler available
// Fontspector crates require: sudo apt-get install protobuf-compiler

// Future integration approach:
// 1. Use fontspector = "1.5.0" as Rust dependency
// 2. Use fontspector-profile-googlefonts and fontspector-profile-universal
// 3. Direct API calls instead of external binary

pub fn detect_fontspector() -> Option<PathBuf> {
    // Try multiple detection methods
}
```

### Command Line Interface

```bash
# Basic QA run
fontspector check-font compiled_font.ttf --output qa_report.json

# With specific profile
fontspector check-font compiled_font.ttf --profile qa-profile.json --output report.json

# Exclude specific checks
fontspector check-font font.ttf --exclude-checks com.google.fonts/check/family_naming
```

### Report Format

Fontspector outputs JSON reports. Example structure:

```json
{
  "result": {
    "PASS": 42,
    "FAIL": 3,
    "WARN": 7,
    "INFO": 12,
    "SKIP": 8
  },
  "sections": [
    {
      "checks": [
        {
          "result": "FAIL",
          "check": "com.google.fonts/check/glyph_coverage",
          "message": "Font is missing required glyphs",
          "severity": 10,
          "filename": "font.ttf"
        }
      ]
    }
  ]
}
```

## UI/UX Design

### QA Tab Layout

```
┌─ QA ──────────────────────────────────────────────────────────────────────┐
│ ● Running QA Analysis... [████████████████████      ] 73%                 │
├─ Filters ─────────────────────────────────────────────────────────────────┤
│ Severity: [All] [ERROR] [WARN] [INFO]  Category: [All] [Outlines] [Meta]  │
├─ Issues (3 Errors, 7 Warnings, 12 Info) ────────────────────────────────┤
│ > ERROR   com.google.fonts/check/glyph_coverage                           │
│   WARN    com.google.fonts/check/outline_direction                        │
│   WARN    com.google.fonts/check/contour_count                           │
│   INFO    com.google.fonts/check/font_version                            │
│   ...                                                                      │
├─ Details ─────────────────────────────────────────────────────────────────┤
│ Check: Font glyph coverage                                                │
│ ID: com.google.fonts/check/glyph_coverage                                │
│ Severity: ERROR                                                           │
│ Message: Font is missing 12 required Unicode codepoints:                 │
│   U+0020 SPACE, U+00A0 NO-BREAK SPACE, U+00AD SOFT HYPHEN, ...          │
│                                                                           │
│ Recommendation: Add the missing glyphs to ensure proper text support.    │
└─ Status: Last run 2 minutes ago ─ Next run: Auto ─ [R]efresh [F]ilter ──┘
```

### Key Features

1. **Progress indicator** during QA analysis
2. **Filter controls** for severity and category
3. **Issue list** with severity icons and quick navigation
4. **Detail panel** showing full check information and recommendations
5. **Status bar** with timestamps and controls
6. **Keyboard shortcuts** for navigation and actions

## Task Implementation Plan

### Phase 1: Core Infrastructure
- [ ] Create QA module structure (`src/qa/`)
- [ ] Implement Fontspector detection and runner
- [ ] Create basic QA report data structures
- [ ] Set up QA storage directories (`~/.config/bezy/qa/`)

### Phase 2: Font Compilation
- [ ] Integrate FontC compilation pipeline
- [ ] Implement UFO → TTF compilation for QA
- [ ] Create temp file management system
- [ ] Add font hashing for cache management

### Phase 3: TUI Integration
- [ ] Enhance QA tab state management
- [ ] Implement QA report display UI
- [ ] Add filtering and navigation controls
- [ ] Create issue detail view

### Phase 4: Save Integration & Triggers
- [ ] Integrate QA triggers with save operations
- [ ] Add QA hooks to save/export workflow
- [ ] Create save-triggered QA analysis system
- [ ] Implement smart report caching and history

### Phase 5: Advanced Features
- [ ] Add QA profile customization
- [ ] Implement issue severity configuration
- [ ] Create QA report comparison
- [ ] Add export functionality for reports

## Configuration

### QA Settings (`~/.config/bezy/qa/config/qa-settings.toml`)

```toml
[fontspector]
executable_path = "/usr/local/bin/fontspector"
default_profile = "googlefonts"
run_on_save = true
run_on_export = true
manual_refresh_only = false

[compilation]
output_format = "ttf"  # or "otf"
optimize = true
cache_compiled_fonts = true

[ui]
default_severity_filter = "all"  # "errors", "warnings", "all"
show_progress_notifications = true
auto_refresh_interval = 30  # seconds

[storage]
keep_report_history = true
max_reports_per_font = 10
cleanup_temp_files = true
```

### Fontspector Profile (`~/.config/bezy/qa/config/fontspector-profile.json`)

```json
{
  "configuration": {
    "profile": "custom",
    "checks": {
      "exclude": [
        "com.google.fonts/check/family_naming",
        "com.google.fonts/check/license"
      ],
      "severity_overrides": {
        "com.google.fonts/check/outline_direction": "WARN"
      }
    }
  }
}
```

## Integration Points

### With Bezy Core
- **Font Loading**: Trigger QA when new font is loaded
- **Font Saving**: Auto-run QA after save operations
- **Font Export**: Include QA check in export pipeline
- **Glyph Editing**: Mark font as needing QA re-analysis

### With TUI System
- **Tab Navigation**: Integrate with existing tab system
- **Message Passing**: Use existing TUI communication system
- **Status Updates**: Show QA progress in status bar
- **Notifications**: Alert user to critical QA issues

### With File System
- **Save Event Detection**: Monitor for save operations
- **Cache Management**: Intelligent cleanup of temp files
- **Report Storage**: Organized storage of QA history
- **Configuration**: User-customizable QA settings

## Error Handling

### Fontspector Not Found
- Graceful degradation with helpful error message
- Installation instructions for different platforms
- Option to specify custom Fontspector path

### Compilation Failures
- Clear error reporting from FontC
- Fallback to original font file if compilation fails
- Log compilation errors for debugging

### QA Analysis Failures
- Handle Fontspector crashes gracefully
- Retry mechanism for transient failures
- Partial results display when possible

## Performance Considerations

### Caching Strategy
- Cache compiled fonts by content hash
- Store QA reports with timestamps
- Invalidate cache when source UFO changes

### Background Processing
- Run QA analysis in background threads
- Non-blocking UI during analysis
- Progressive result updates

### Resource Management
- Limit concurrent QA processes
- Clean up temporary files automatically
- Configurable cache size limits

## Future Enhancements

1. **Custom Check Development**: Allow users to write custom Fontspector checks
2. **CI/CD Integration**: Export QA reports for continuous integration
3. **Batch Processing**: Run QA on multiple fonts simultaneously
4. **Visual Debugging**: Show problematic glyphs visually in the editor
5. **Fix Suggestions**: Automatic fixes for common issues
6. **Team Collaboration**: Share QA profiles and reports across teams

## Implementation Summary

### What Was Delivered

**Complete QA Module Infrastructure**:
- Full module structure in `src/qa/` with fontspector, compiler, trigger, and storage components
- Comprehensive TUI integration with multiple view modes (Issue List, Detail, Summary)
- Save-triggered QA analysis system (performance optimized)
- Report caching and history management
- Proper error handling and user feedback

**Key Files Created/Modified**:
- `src/qa/mod.rs` - Core QA engine and data structures
- `src/qa/fontspector.rs` - Placeholder runner (ready for real Fontspector integration)
- `src/qa/compiler.rs` - FontC integration for UFO→TTF compilation
- `src/qa/trigger.rs` - Save-event triggering system
- `src/qa/storage.rs` - Report storage and caching
- `src/tui/tabs/qa.rs` - Complete QA tab UI implementation
- `src/tui/app.rs` - QA tab integration
- `Cargo.toml` - Dependencies (commented out pending protobuf resolution)

**Current Status**: Fully functional QA tab with placeholder data. Ready for Fontspector integration when dependency issues are resolved.

### Deployment Considerations

**For Production Deployment**:
1. **Option A**: Install protobuf compiler in deployment environment
   ```bash
   apt-get install protobuf-compiler  # Debian/Ubuntu
   yum install protobuf-compiler      # RHEL/CentOS
   ```

2. **Option B**: Use Docker with protobuf pre-installed

3. **Option C**: Wait for Fontspector library crate without protobuf dependencies

4. **Option D**: Implement direct FontBakery Python integration as alternative

**For Development**: Current placeholder system provides full functionality for UI/UX development and testing.

---

This document serves as the comprehensive plan for implementing QA functionality in Bezy. It will be updated as development progresses and requirements evolve.