# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
`jterm` is a terminal-based Japan prefecture tracking application built in Rust using the `ratatui` framework. Users can track their travel experiences across all 47 Japanese prefectures with experience levels 0-5.

## Common Commands
- `cargo run` - Run the application
- `cargo check` - Check for compilation errors
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version

## Architecture

### Core Dependencies
- `ratatui = "0.29"` - Terminal UI framework (primary rendering)
- `crossterm = "0.28"` - Terminal control and events
- `serde/serde_json` - Data persistence to ~/.jterm/progress.json

### Application Structure
The entire application is contained in `src/main.rs` (1,367 lines) with a single `JTermApp` struct managing all state:

```rust
struct JTermApp {
    prefectures: Vec<Prefecture>,      // 47 prefecture data
    user_progress: UserProgress,       // HashMap of prefecture -> level (0-5)
    show_alt_map: bool,               // Enhanced map view (key 'w')
    // ... other view state flags
}
```

### View System
The app has 4 main view modes controlled by boolean flags:
1. **List View** (default) - Scrollable prefecture list
2. **Regional Map View** (`show_map`, key 'm') - Text-based regional groupings  
3. **Statistics View** (`show_stats`, key 's') - Progress analytics
4. **Enhanced Overview Map** (`show_alt_map`, key 'w') - Geographic layout with colored squares

### Enhanced Map Implementation
The enhanced map (`render_alt_map_view()`) uses:
- 60x20 character grid (`map_grid`)
- Hardcoded geographic coordinates for each prefecture
- Experience level colored squares: ⬜🟥🟨🟩🟪🟦 (levels 0-5)
- Geographic positioning that roughly matches Japan's actual layout

### Data Model
Prefecture experience levels:
- 0: Never been (⬜)
- 1: Passed there (🟥) 
- 2: Alighted there (🟨)
- 3: Visited there (🟩)
- 4: Stayed there (🟪)
- 5: Lived there (🟦)

### Assets
- `/img/` directory contains SVG and PNG files of Japan maps (currently unused)
- Files include: japanex.svg, japanex_final.svg, japanex_optimized.svg, etc.

### Key Functions
- `render_alt_map_view()` - Enhanced map rendering (lines 1059-1219)
- `get_color_square()` - Maps experience level to colored emoji
- Geographic positioning uses hardcoded coordinates like `map_grid[row][col + offset_x]`