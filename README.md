# tcalc

A terminal-based calculator with TUI interface built in Rust.

![Demo](tcalc.gif)

## Features

- **Dual Modes**: Basic and Scientific calculator modes
- **Interactive UI**: Button navigation with keyboard and mouse support
- **Typing Mode**: Direct expression input with live evaluation
- **Calculation History**: View and recall previous calculations
- **Function Graphing**: Visualize mathematical expressions
- **Secondary Functions**: Access variables and constants via 2nd function key
- **Professional Interface**: Clean, color-coded button layout

## Controls

### Navigation
- `↑↓←→` - Navigate buttons
- `Enter`/`Space` - Press selected button
- Mouse click - Press button directly

### Modes
- `` ` `` - Toggle typing mode
- `m` - Switch between Basic/Scientific modes
- `h` - Toggle calculation history
- `2nd` - Access secondary functions (variables, constants)

### Operations
- `c` - Clear current expression
- `C` - Clear all (expression and history)
- `⌫` - Backspace
- `r` - Recall from history (when in history view)
- `Ctrl+g` - Graph current expression
- `?` - Show help modal

### Graphing
- `↑↓←→` - Pan graph view
- `+/-` - Zoom in/out
- `r` - Reset view to default range
- `c` - Toggle coordinate display
- `Esc` - Exit graph mode

### Exit
- `q` or `Esc` - Quit application

## Installation

```bash
git clone <repository-url>
cd tcalc
cargo build --release
./target/release/tcalc
```

## Dependencies

- `anyhow` - Error handling
- `chrono` - Timestamp formatting
- `crossterm` - Terminal control
- `ratatui` - TUI framework

## Usage

Run the application and use keyboard navigation or mouse clicks to interact with the calculator buttons. 

### Basic Operations
Switch to typing mode for direct expression input, or use button navigation for traditional calculator operation.

### Graphing Functions
1. Build an expression using variables (x, y, z, a, b, c) and constants (π, e)
2. Press `Ctrl+g` or use the Graph button to visualize the expression
3. Use arrow keys to pan, +/- to zoom, and `r` to reset the view

### Variables and Constants
- **Variables**: x, y, z, a, b, c (available in 2nd function mode)
- **Constants**: π (3.14159), e (2.71828)
- **Scientific functions**: sin, cos, tan, log, ln, sqrt, exp, abs (in Scientific mode)

## License

This project is licensed under the [MIT License](LICENSE).
