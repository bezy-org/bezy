# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-09-27

### Added
- Initial public release of Bezy font editor
- Core font editing capabilities:
  - UFO format support (load, edit, save)
  - Bezier curve editing with on-curve and off-curve points
  - Multiple selection modes (individual, marquee, all)
  - Smooth curve constraints for maintaining curve continuity
  - Undo/redo functionality
  - Grid snapping and axis locking

- Editing tools:
  - Select tool for point manipulation
  - Pen tool for drawing paths
  - Knife tool for cutting contours
  - Pan tool for navigation
  - Measure tool for distance/angle measurements
  - Shape tools (rectangle, ellipse)
  - Text tool for text placement
  - AI-powered editing capabilities
  - Metaball-based shape generation

- User interface:
  - Customizable toolbar system
  - Theme support (dark/light modes with color variations)
  - Keyboard shortcuts for all major operations
  - Visual indicators for point types and selections
  - Zoom-aware scaling for consistent UI at all zoom levels

- Typography features:
  - Arabic text support with RTL layout
  - Text shaping with HarfRust
  - Font metrics visualization
  - Glyph navigation system

- Export capabilities:
  - TTF export via fontc compilation
  - UFO format preservation

- Developer features:
  - ECS-based architecture using Bevy
  - FontIR integration for font data management
  - Extensible tool system
  - Theme customization via TOML configuration

### Known Limitations
- Limited to UFO format for font editing (no direct TTF/OTF editing)
- Experimental AI features require additional setup
- Some advanced OpenType features not yet supported
- Performance optimization needed for fonts with many glyphs

### Technical Details
- Built with Rust and Bevy 0.16
- Uses Norad for UFO parsing
- Integrates fontc for font compilation
- GPU-accelerated rendering with WGPU
- Cross-platform support (Linux, macOS, Windows)

[0.1.0]: https://github.com/bezy-org/bezy/releases/tag/v0.1.0
