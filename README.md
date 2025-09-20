# Bezy

Bezy is an in-development cross-platform font editor with a built-in bidirectional text editor.

![Bezy Font Editor Screenshot](https://bezy.org/images/bezy-screenshot-001.jpg)

Drawing inspiration from customizable editors like RoboFont and MFEK, Bezy reimagines font editing for contemporary Unix-like AI and CLI heavy workflows. The core dependencies are: [Bevy](https://bevy.org), [HarfRust](https://github.com/harfbuzz/harfrust), [Norad](https://github.com/linebender/norad), [Kurbo](https://github.com/linebender/kurbo), [Fontc](https://github.com/googlefonts/fontc), [FontIR](https://github.com/googlefonts/fontc).

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

You can install Bezy globally and use it as a command-line tool from anywhere on your system.

### Install from Source
```bash
# From within the bezy directory after cloning
cargo install --path .

# Or install directly from GitHub
cargo install --git https://github.com/bezy-org/bezy.git
```

### Post Installation 
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

```bash
# Edit a specific font source (UFO or designspace)
bezy --edit ~/Fonts/MyFont.ufo
```
The `--edit` flag intelligently handles both UFO directories and designspace files:
- **Single UFO**: Shows a clean interface without master selection controls
- **Designspace**: Shows master selector circles for switching between different masters

## Command Line Flags

Bezy is designed to be used as a command line tool in Unix-style workflows.

| Flag | Short | Description | Example |
|------|-------|-------------|---------|
| `--edit <PATH>` | `-e` | Edit a font source (UFO directory or .designspace file) | `bezy --edit MyFont.ufo` |
| `--theme <NAME>` | `-t` | Set the color theme | `bezy --theme lightmode` |
| `--new-config` | | Initialize user configuration directory | `bezy --new-config` |
| `--no-default-buffer` | | Start without default text buffer | `bezy --no-default-buffer` |
| `--help` | `-h` | Show help information | `bezy --help` |
| `--version` | `-V` | Show version information | `bezy --version` |

### Examples
```bash
# Initialize user configuration (run once)
bezy --new-config

# Edit a single UFO
bezy --edit ~/Fonts/MyFont.ufo

# Edit a designspace for variable fonts
bezy --edit ~/Fonts/MyVariable.designspace

# Use the strawberry theme
bezy --theme strawberry

# Combine as many flags as you need
bezy --edit ~/Fonts/MyFont.ufo --theme light

# Short form using -e
bezy -e MyFont.ufo
```

## Keyboard Shortcuts

| Shortcut | Action | Context |
|----------|--------|---------|
| `Cmd/Ctrl + S` | Save font | Global |
| `Escape` | Clear selection / Exit tool | Selection mode |


## Selection & Editing

| Shortcut | Action | Context |
|----------|--------|---------|
| `Arrow Keys` | Nudge selected points | Points selected |
| `Shift + Arrow Keys` | Nudge selected points more | Points selected |
| `Cmd/Ctrl + Arrow Keys` | Nudge selected points even more | Points selected |

## Camera Controls

| Control | Action |
|---------|--------|
| Mouse Wheel | Zoom in/out |
| Space | Temporary pan view |

## Working with Edit-Mode Tools

The edit-mode toolbar provides access to various editing tools. Each tool has specific behaviors:

- **Selection Tool**: Select and manipulate points
- **Pen Tool**: Add new points and contours
- **Knife Tool**: Cut contours at specific points
- **Text Tool**: A text editor built with editable type sorts 
- **Measure Tool**: Measure distances between contours

# Themes & Configuration

Bezy includes four built-in themes and supports user configuration and theme customization.

## Built-in Themes
- `dark` (default) - Dark background with light text
- `light` - Light background with dark text
- `strawberry` - Pink/red/green theme
- `campfire` - Warm orange/red/brown/black theme

## User Configuration

### Initialize Configuration (Optional)
You can set up a user configuration directory to customize settings and themes:

```bash
# Initialize your user config directory (run once)
bezy --new-config
```

This creates:
- `~/.config/bezy/settings.json` - User preferences like default theme
- `~/.config/bezy/themes/` - Editable copies of all themes

### Setting a Default Theme
After running `--new-config`, edit `~/.config/bezy/settings.json`:

```json
{
  "default_theme": "strawberry"
}
```

Now you can run `bezy` without the `--theme` flag and it will use your preferred theme.

### Theme Priority
Bezy uses this priority order for themes:
1. **CLI argument**: `bezy --theme light` (highest priority)
2. **Config file**: `default_theme` in `~/.config/bezy/settings.json`
3. **Built-in default**: "dark" theme (fallback)

## Custom Themes

### Theme Loading System
- **Without user config**: Uses embedded themes (no setup required)
- **With `~/.config/bezy/themes/`**: Uses themes from this directory

### Creating Custom Themes
```bash
# Initialize config to get editable theme files
bezy --new-config

# Now you can edit any theme file:
nano ~/.config/bezy/themes/strawberry.json

# Or create a new theme file:
cp ~/.config/bezy/themes/dark.json ~/.config/bezy/themes/mytheme.json
nano ~/.config/bezy/themes/mytheme.json

# Use your custom theme
bezy --theme mytheme

# Or set it as default in settings.json
```

### Hot Reload
When using custom themes from `~/.config/bezy/themes/`, changes are reflected instantly when you save theme files (in debug builds).

# License
[GNU GENERAL PUBLIC LICENSE](https://www.gnu.org/licenses/gpl-3.0.en.html) Version 3, 29 June 2007

This project is licensed under the GNU General Public License (GPL), which means you are free to use, study, share, and change the software. The only condition is that if you share it with others—whether in its original form or with your own changes—you must also make the source code available under the same license. In short: the GPL ensures that the software (and any improvements to it) always stays free and open for everyone.

# Homepage

[https://bezy.org](https://bezy.org)
