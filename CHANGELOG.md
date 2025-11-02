# Changelog

All notable changes to NotNative will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-11-02

### Fixed
- **Critical**: Fixed Omarchy theme not loading when installed from AUR
  - CSS loading order improved: Omarchy theme variables now load before app styles
  - Synchronized `load_theme_css()` between `main.rs` and `app.rs`
  - Theme watcher now properly reloads CSS in correct order
  - Better theme detection and error messages
- Removed invalid `text-align` CSS property that caused GTK warnings
- Improved debug output to help diagnose theme loading issues
- **Real-time theme switching now works**: Change Omarchy theme and see NotNative update instantly

### Changed
- Enhanced theme loading mechanism for better reliability
- Added detailed troubleshooting section in README

### Documentation
- Complete translation of README.md to English
- Added comprehensive Arch Linux installation instructions (AUR, manual, from source)
- Highlighted Omarchy OS integration features
- Added troubleshooting guide for theme issues

## [0.1.0] - 2025-10-26

### Added
- **Vim-inspired modal editor** with Normal, Insert, Command, and Visual modes
- **Lightning-fast text buffer** powered by ropey (O(log n) operations)
- **Real-time markdown rendering** with dual view modes
- **Omarchy OS theme integration** with automatic theme watching
- **Sidebar with folder organization** and keyboard navigation
- **Smart autosave** every 5 seconds with visual indicators
- **Native GTK4 interface** without libadwaita
- **Undo/Redo system** with 1000-operation history
- **Interactive markdown links** (clickable, open in browser)
- **Keyboard-first workflow** with extensive vim-like shortcuts

### Features Implemented
- Complete vim navigation (h/j/k/l, 0/$, gg/G)
- Editing commands (x, dd, i, u, Ctrl+z, Ctrl+r)
- Command mode (:w, :q, :wq, :q!)
- Markdown syntax support (headings, bold, italic, code, links, lists, blockquotes)
- Folder system with expandable folders
- Hover-to-load notes in sidebar
- Context menu (right-click) for notes
- Status bar with mode indicator and statistics
- Theme watcher for automatic updates
- Accent composition support (á, é, í, ó, ú, ñ)

### Technical
- Built with Rust 2024 Edition
- GTK4 + Relm4 0.10 reactive framework
- ropey 1.6 for efficient text editing
- pulldown-cmark 0.10 for markdown parsing
- notify 6 for file system watching
- Notes stored as plain .md files in `~/.local/share/notnative/notes/`

### Known Issues
- Note renaming UI not fully functional
- Nested folders display issues
- Folder deletion not implemented
- Some GTK parent/unparent warnings in context menu

---

**Legend:**
- Added: New features
- Changed: Changes in existing functionality
- Fixed: Bug fixes
- Deprecated: Soon-to-be removed features
- Removed: Removed features
- Security: Security fixes
