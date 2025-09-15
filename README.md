# Bezy

Bezy is an in-development cross-platform font editor with a built-in bidirectional text editor.

![Bezy Font Editor Screenshot](https://bezy.org/images/bezy-screenshot-001.jpg)

Drawing inspiration from technical-user-oriented customizable editors like RoboFont and MFEK, Bezy reimagins font editing for contemporary Unix-like AI and CLI heavy workflows. The core dependancies are: [Bevy](https://bevy.org), [HarfRust](https://github.com/harfbuzz/harfrust), [Norad](https://github.com/linebender/norad), [Kurbo](https://github.com/linebender/kurbo), [Fontc](https://github.com/googlefonts/fontc), [FontIR](https://github.com/googlefonts/fontc).

Bezy is written in the Rust programming language using a [game engine](https://bevy.org) to create a performant and fun experience that keeps users in a flow state. It is designed to be a visually pleasing environment where design work is done, not just a non-aesthetic production tool.

Rust has great [documentation](https://doc.rust-lang.org/book/), a friendly compiler with useful error messages, top-notch tooling, and an integrated package manager and build tool. With help from AI tools like [Claude Code](https://www.anthropic.com/claude-code) and [Gemini CLI](https://github.com/google-gemini/gemini-cli), it can be easier to use than Python. Don't be intimidated if you are not an expert programmer—this application is designed for students, designers, and artists to be able to customize it and make their own tools and other additions.

The project aims to be a welcoming community that values working in the open, sharing knowledge, and helping people become better programmers. Contributors of all skill levels are welcome.

> “The enjoyment of one's tools is an essential ingredient of successful work.”  
>—Donald Knuth

>“Many have tried to replace FontForge—all have failed. I might fail, in fact, history says I probably will. Yet, the current state of affairs is so bad I feel I must try.”
>—Fredrick Brennan

# Installation & Building from Source

## Prerequisites

Install Rust by following the [official instructions](https://www.rust-lang.org/tools/install) at [rust-lang.org](https://www.rust-lang.org).

### Verify installation:
```bash
cargo --version
```

## Building from Source

### Step 1: Clone the Repository
```bash
git clone https://github.com/bezy-org/bezy.git
cd bezy
```

### Step 2: Build and Run
```bash
# Build and run
cargo run

# Build with optimizations (slower to compile, faster to run)
cargo run --release

# Build and run with a specific font source (UFO or designspace)
cargo run -- --edit path/to/your/font.ufo
```

## Installing as a Command Line Tool

You can install Bezy globally to use it as a command-line tool from anywhere on your system.

### Install from Source
```bash
# From within the bezy directory after cloning
cargo install --path .

# Or install directly from GitHub
cargo install --git https://github.com/bezy-org/bezy.git
```

### After Installation 
Once installed, you can run Bezy from anywhere on your system.
```bash
# Launch the editor without loading a source file 
bezy

# Check the version
bezy --version

# Check the installation location
which bezy
```

### Updating Bezy
To update after making changes or pulling new updates:
```bash
# If you're working from the cloned repository
cd path/to/bezy
git pull
cargo install --path .

# Or reinstall directly from GitHub
cargo install --git https://github.com/bezy-org/bezy.git

# If you are having trouble updating try with --force
cargo install --path . --force
```

### Uninstalling
```bash
# To remove the globally installed version
cargo uninstall bezy
```

# How to Use

## Basics 
```bash
# Edit a specific font source (UFO or designspace)
bezy --edit ~/Fonts/MyFont.ufo

# Use with any command line flags
bezy --theme lightmode --edit MyFont.ufo
```
The `--edit` flag intelligently handles both UFO directories and designspace files:
- **Single UFO**: Shows a clean interface without master selection controls
- **Designspace**: Shows master selector circles for switching between different masters

## Command Line Flags

Bezy is designed to be used as a command line tool in Unix-style workflows. If you aren't familiar with working this way, it's worth learning and will save you time in the long run. 

### Common Flag Options

| Flag | Short | Description | Example |
|------|-------|-------------|---------|
| `--edit <PATH>` | `-e` | Edit a font source (UFO directory or .designspace file) | `bezy --edit MyFont.ufo` |
| `--theme <NAME>` | `-t` | Set the color theme | `bezy --theme lightmode` |
| `--no-default-buffer` | | Start without default text buffer | `bezy --no-default-buffer` |
| `--help` | `-h` | Show help information | `bezy --help` |
| `--version` | `-V` | Show version information | `bezy --version` |

### Themes

Bezy includes four built-in themes embedded in the application and supports optional custom theme creation:

#### Built-in Themes
- `darkmode` (default) - Dark background with light text
- `lightmode` - Light background with dark text
- `strawberry` - Pink/red/green theme
- `campfire` - Warm orange/brown theme

These themes are embedded in the application and work in both development and installed modes.

#### Custom Themes (Optional)
You can optionally create and edit your own themes by creating a `~/.bezy/themes/` directory. The theme system works as follows:

**Default behavior**: Uses embedded themes (no setup required)
**With `~/.bezy/themes/`**: Uses themes from this directory when it exists

##### Theme Loading Priority:
1. If `~/.bezy/themes/` exists → Load themes from `~/.bezy/themes/*.json`
2. If not → Use embedded themes (default)
3. Hot-reload: When using custom themes, changes are reflected instantly when you save the file

##### Creating Custom Themes:
```bash
# Optional: Create themes directory for customization
mkdir -p ~/.bezy/themes

# Copy built-in theme as starting point (get from src/ui/themes/ in source)
# Or create your own JSON file following the theme structure

# Use your custom theme
bezy --theme mytheme
```

**Note**: You don't need to create `~/.bezy/themes/` unless you want to customize themes. The built-in themes work out of the box.

### Examples
```bash
# Edit a single UFO (no master selector shown)
bezy --edit ~/Fonts/MyFont.ufo

# Edit a designspace for variable fonts (master selector shown)
bezy --edit ~/Fonts/MyVariable.designspace

# Use the built-in test font with strawberry theme
bezy --theme strawberry

# Combine as many flags as you need
bezy --edit ~/Fonts/MyFont.ufo --theme lightmode

# Short form using -e
bezy -e MyFont.ufo
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

### Working with Edit-Mode Tools

The edit-mode toolbar provides access to various editing tools. Each tool has specific behaviors:

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
5. **Read the code**: If you don't understand how something works, read the source code. Use a tool like Claude Code if you need help. This project is designed to be understandable and modifyable by regular users.

### File Format Support

Bezy currently supports:
- **UFO 3** (Unified Font Object) - Individual font files
- **Designspace** - Variable font sources

UFO files are directories (folders) containing XML files that describe your font. They're human-readable, version-control friendly, and application independant.

# License
GNU GENERAL PUBLIC LICENSE
Version 3, 29 June 2007

The GNU General Public License is a copyleft license for software and other kinds of works.

# Homepage

[https://bezy.org](https://bezy.org)
