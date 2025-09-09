# Bezy

A cross-platform bidirectional RTL/LTR font editor built with Rust. The core dependancies are: Bevy, HarfRust, Norad, Kurbo, Fontc, FontIR.

This editor is a spiritual successor of font editors like RoboFont and MFEK that are designed for customization and tool making.

Bezy is written in the Rust programming language using a game engine to create a performant, fun and aesthetic experience that keeps users in a flow state. Rust has great documentation, a friendly compiler with useful error messages, top-notch tooling, and an integrated package manager and build tool. With help from AI tools like Claude Code and Gemini CLI, it can be easier to use than Python. Don't be intimidated if you are not an expert programmer—this application is designed for students, designers, and artists to be able to customize it and make their own tools.

We aim to be an open and welcoming community that values working in the open and sharing knowledge. Contributors of all skill levels are welcome.

> “The enjoyment of one's tools is an essential ingredient of successful work.”  
> — Donald Knuth

## Installation

### Prerequisites

Install Rust by following the official instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

Verify installation:
```bash
rustc --version
cargo --version
```

### Building Bezy

#### Step 1: Clone the Repository
```bash
git clone https://github.com/bezy-org/bezy.git
cd bezy
```

#### Step 2: Build and Run
```bash
# Build and run with the default font
cargo run

# Build with optimizations (slower to compile, faster to run)
cargo run --release

# Build and run with a specific UFO file
cargo run -- --load-ufo path/to/your/font.ufo
```

#### First Time Build
The first build will take a few minutes as it downloads and compiles all dependencies. Subsequent builds will be much faster (usually under 30 seconds).

### Troubleshooting

**Performance issues:**
- Always use `cargo run --release` for the best performance

## Command Line Flags

Bezy supports several command-line options to customize your experience:

### Basic Usage
```bash
bezy [OPTIONS]
```

### Available Options

| Flag | Short | Description | Example |
|------|-------|-------------|---------|
| `--load-ufo <PATH>` | `-f` | Load a specific UFO file or designspace | `bezy --load-ufo MyFont.ufo` |
| `--theme <NAME>` | `-t` | Set the color theme | `bezy --theme lightmode` |
| `--no-default-buffer` | | Start without default text buffer | `bezy --no-default-buffer` |
| `--help` | `-h` | Show help information | `bezy --help` |
| `--version` | `-V` | Show version information | `bezy --version` |

### Themes
Available themes:
- `darkmode` (default) - Dark background with light text
- `lightmode` - Light background with dark text  
- `strawberry` - Pink/red/green theme
- `campfire` - Warm orange/brown theme

### Examples
```bash
# Load a specific font with light theme
bezy --load-ufo ~/Fonts/MyFont.ufo --theme lightmode

# Load a designspace for variable fonts
bezy --load-ufo ~/Fonts/MyVariable.designspace

# Use the built-in test font with strawberry theme
bezy --theme strawberry
```

## Keyboard Shortcuts

### Essential Shortcuts

| Shortcut | Action | Context |
|----------|--------|---------|
| `Cmd/Ctrl + Z` | Undo | Global |
| `Cmd/Ctrl + Shift + Z` | Redo | Global |
| `Cmd/Ctrl + S` | Save font | Global |
| `Cmd/Ctrl + G` | Show glyph palette | Global |
| `Escape` | Clear selection / Exit tool | Selection mode |
| `Cmd/Ctrl + A` | Select all points | Selection mode |

### Navigation

| Shortcut | Action |
|----------|--------|
| `Home` | Go to first glyph |
| `End` | Go to last glyph |
| `Cmd/Ctrl + Plus` | Zoom in |
| `Cmd/Ctrl + Minus` | Zoom out |
| `T` | Toggle zoom to cursor |

### Selection & Editing

| Shortcut | Action | Context |
|----------|--------|---------|
| `Arrow Keys` | Nudge selected points (1 unit) | Points selected |
| `Shift + Arrow Keys` | Nudge selected points (10 units) | Points selected |
| `Cmd/Ctrl + Arrow Keys` | Nudge selected points (100 units) | Points selected |
| Click + Drag | Select points with marquee | Selection tool |
| `Shift + Click` | Add to selection | Selection tool |

### Camera Controls

| Control | Action |
|---------|--------|
| Mouse Wheel | Zoom in/out |
| Left/Middle Mouse + Drag | Pan view |
| `T` | Toggle zoom to cursor on/off |

## Usage Guide

### Basic Workflow

1. **Launch Bezy**
   ```bash
   cargo run -- --load-ufo your-font.ufo
   ```

2. **Navigate Glyphs**
   - Use `Home`/`End` keys to jump to first/last glyph
   - Click on glyph cells in the grid to switch glyphs

3. **Edit Points**
   - Select the Selection tool (or press `V`)
   - Click and drag to select points
   - Use arrow keys to nudge selected points
   - Drag points directly to move them

4. **Save Your Work**
   - Press `Cmd/Ctrl + S` to save changes back to the UFO file

### Working with Tools

The toolbar on the left provides access to various editing tools. Each tool has specific behaviors:

- **Selection Tool**: Select and manipulate points
- **Pen Tool**: Add new points and curves
- **Knife Tool**: Cut paths at specific points
- **Text Tool**: Edit text samples for testing
- **Measure Tool**: Measure distances between points

### Tips for Beginners

1. **Start with the built-in font**: Run `cargo run` without arguments to explore with the default Bezy Grotesk font
2. **Use release mode for smoother performance**: `cargo run --release`
3. **Save frequently**: Use `Cmd/Ctrl + S` to save your changes
4. **Experiment with themes**: Try different themes to find what works best for your eyes
5. **Learn by doing**: The best way to learn is to open a font and start editing!

### File Format Support

Bezy currently supports:
- **UFO 3** (Unified Font Object) - Individual font files
- **Designspace** - Variable font sources

UFO files are directories (folders) containing XML files that describe your font. They're human-readable and version-control friendly.

# License

GPL v3.0

# Homepage

[https://bezy.org](https://bezy.org)
