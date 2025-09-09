# Bezy

![Bezy Font Editor Screenshot](https://bezy.org/images/bezy-screenshot-001.jpg)

Bezy is a cross-platform, Rust-based font editor with a built-in bidirectional (RTL/LTR) text editor. It draws inspiration from technical-user oriented customizable editors like RoboFont and MFEK, while reimagining font editing for contemporary Unix-like AI and CLI heavy workflows. The core dependancies are: [Bevy](https://bevy.org/), [HarfRust](https://github.com/harfbuzz/harfrust), [Norad](https://github.com/linebender/norad), [Kurbo](https://github.com/linebender/kurbo), [Fontc](https://github.com/googlefonts/fontc), [FontIR](https://github.com/googlefonts/fontc).

Bezy is written in the Rust programming language using a game engine to create a performant, fun, and aesthetic experience that keeps users in a flow state. Rust has great [documentation](https://doc.rust-lang.org/book/), a friendly compiler with useful error messages, top-notch tooling, and an integrated package manager and build tool. With help from AI tools like [Claude Code](https://www.anthropic.com/claude-code) and [Gemini CLI](https://github.com/google-gemini/gemini-cli), it can be easier to use than Python. Don't be intimidated if you are not an expert programmer—this application is designed for students, designers, and artists to be able to customize it and make their own tools. This project is a spiritual sucsessor to the [RoboFont design principles](https://robofont.com/documentation/topics/robofont-design-principles/):

>A typeface designer (who does not want to do any scripting and has no means of letting someone else do that work), has to deal with the available user interface, tool set, feature set, and the list of bugs in the typeface design applications they use. This makes the designer a captive to the development of applications that are often merely production tools and not intended for design. Not being (partly) the designer of your own tools puts a designer in a vulnerable position.

We aim to be an open and welcoming community that values working in the open and sharing knowledge. Contributors of all skill levels are welcome.

> “The enjoyment of one's tools is an essential ingredient of successful work.”  
>—Donald Knuth

>“Many have tried to replace FontForge—all have failed. I might fail, in fact, history says I probably will. Yet, the current state of affairs is so bad I feel I must try.”
>—Fredrick Brennan

## Installation

### Prerequisites

Install Rust by following the official instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

Verify installation:
```bash
cargo --version
```

### Building from Source

#### Step 1: Clone the Repository
```bash
git clone https://github.com/bezy-org/bezy.git
cd bezy
```

#### Step 2: Build and Run
```bash
# Build and run
cargo run

# Build with optimizations (slower to compile, faster to run)
cargo run --release

# Build and run with a specific UFO file
cargo run -- --load-ufo path/to/your/font.ufo
```

#### First Time Build
The first build will take a few minutes as it downloads and compiles all dependencies. Subsequent builds will be much faster (usually under 30 seconds).

#### Troubleshooting Performance Issues
- Use `cargo run --release` for the best performance

### Installing as a Command Line Tool

You can install Bezy globally to use it as a command-line tool from anywhere on your system.

#### Install from Source
```bash
# From within the bezy directory after cloning
cargo install --path .

# Or install directly from GitHub
cargo install --git https://github.com/bezy-org/bezy.git
```

#### Using the Installed Command
Once installed, you can run Bezy from anywhere:
```bash
# Run with default settings
bezy

# Load a specific font
bezy --load-ufo ~/Fonts/MyFont.ufo

# Use with any command line flags
bezy --theme lightmode --load-ufo MyFont.ufo
```

#### Updating Bezy
To update Bezy after making changes or pulling new updates:
```bash
# If you're working from the cloned repository
cd /path/to/bezy
git pull
cargo install --path .

# Or reinstall directly from GitHub
cargo install --git https://github.com/bezy-org/bezy.git

# If you are having trouble updating try with --force
cargo install --path . --force
```

#### Uninstalling
To remove the globally installed version:
```bash
cargo uninstall bezy
```

#### Installation Location
By default, Cargo installs binaries to `~/.cargo/bin/`. Make sure this directory is in your PATH. If it's not, add this to your shell configuration file (`.bashrc`, `.zshrc`, etc.):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Command Line Flags

Bezy is designed to be used as a command line tool in Unix-style workflows. If you aren't familiar with working this way, it's worth learning and will save you time in the long run. 

### Basic Usage
```bash
bezy [OPTIONS]
```

### Common Flag Options

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
# Load a designspace for variable fonts
bezy --load-ufo ~/Fonts/MyVariable.designspace

# Use the built-in test font with strawberry theme
bezy --theme strawberry

# Combine as many flags as you need
bezy --load-ufo ~/Fonts/MyFont.ufo --theme lightmode
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
