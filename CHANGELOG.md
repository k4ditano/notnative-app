# Changelog

All notable changes to NotNative will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - v0.1.8

### Added
- **🔗 Backlinks System**: Sistema completo de menciones entre notas usando `@`
  - Autocompletado inteligente al escribir `@` + texto
  - Navegación por click en menciones (Modo Normal)
  - Popover con hasta 8 sugerencias de notas
  - Muestra carpeta de origen en sugerencias
  - Cursor cambia a pointer sobre menciones clickeables
  
- **📂 Abrir en Explorador**: Nueva opción en menú contextual
  - Click derecho en notas/carpetas → "Abrir en explorador"
  - Abre carpetas directamente en explorador del sistema
  - Para notas, abre el directorio que las contiene
  - Compatible con todos los gestores de archivos Linux (vía xdg-open)
  
- **🔗 Detección Automática de URLs**: Conversión inteligente al pegar
  - URLs normales se convierten automáticamente a enlaces markdown
  - Formato generado: `[dominio](url_completa)`
  - Mantiene funcionalidad existente para YouTube e imágenes
  - Detecta URLs de 10+ caracteres sin espacios

### Technical Details
- Nuevo struct `NoteMentionSpan` para almacenar menciones
- Función `detect_note_mentions()` detecta menciones en texto renderizado
- Mensajes `CheckNoteMention` y `CompleteMention(String)` para autocompletado
- Widget `note_mention_popup` con lista scrollable de sugerencias
- Acción GTK `open_folder_action` para menú contextual
- Handler `OpenInFileManager(String, bool)` ejecuta xdg-open

### Documentation
- Nuevo archivo `docs/BACKLINKS_Y_MEJORAS.md` con documentación completa
- Actualizado README.md con nuevas funcionalidades
- Ejemplos de uso y casos de uso típicos

---

## [0.1.7] - 2024-11-09

### Added - Quality of Life Improvements
- Fixed frontmatter YAML tags clickability in Normal mode
- Tags now searchable immediately after creation (no need to restart app)
- Fixed note name resolution for database updates (folder handling)
- Improved search UX: sidebar auto-positions to current note when exiting search
- Auto-focus editor when mouse leaves sidebar (seamless keyboard navigation)
- Fixed hover preview in sidebar after canceling search

### Fixed
- Text regeneration on every cursor movement (scroll fix via caching)
- Cursor visibility in Normal mode
- Auto-selection of single note in expanded folders
- Tag detection in YAML frontmatter (• tag format)
- Immediate tag indexing without restart
- Sidebar positioning after search operations

---

## [0.1.6] - 2024-11-XX

### Added
- **🏷️ Smart Tag System**: #tags clickable anywhere, even at line start
- **🔍 YAML Tag Support**: Clickable tags in frontmatter lists (• tag format)
- **🔍 Precise Tag Search**: #tag searches only that specific tag in database
- **🔌 40+ MCP Tools**: Comprehensive automation toolkit on port 8788
- **🤖 Enhanced AI Workflows**: Advanced chat capabilities and integrations
- **🔄 Real-time Sync**: Automatic file watcher for instant updates

### Fixed
- Tags at beginning of lines not being detected
- Tag rendering now preserves # symbol in display
- Tag detection correctly distinguishes between #tag and # heading
- Clippy warning in CreateNewNote handler

### Documentation
- Added MCP Tools Reference (40+ tools)
- Added cURL examples for API usage
- Updated documentation with v0.1.6 features
- Cleaned up repository structure

---

## [0.1.5] - 2024-XX-XX

### Added
- System tray integration with minimize support
- Multi-language support (i18n): English and Spanish
- MCP server improvements and additional tools
- Background control script for system tray management

### Changed
- Improved theme synchronization with Omarchy themes
- Better integration with Aether and Omarchist apps

---

## [0.1.4] - 2024-XX-XX

### Added
- YouTube video embedding and transcription
- Music player with YouTube integration
- Playlist management (create, save, load)
- Background audio playback

---

## [0.1.3] - 2024-XX-XX

### Added
- MCP (Model Context Protocol) server
- REST API on port 8788
- External control via HTTP requests
- Initial automation tools

---

## [0.1.2] - 2024-XX-XX

### Added
- AI chat integration (OpenAI/OpenRouter)
- Context-aware conversations
- Note attachment for AI context

---

## [0.1.1] - 2024-XX-XX

### Added
- Full-text search with SQLite FTS5
- Tag system with auto-completion
- Folder organization
- Image preview support

---

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Vim-inspired modal editing (Normal, Insert, Visual, Command)
- Real-time Markdown rendering
- Interactive TODO checkboxes
- Basic note management (create, edit, delete, rename)
- Folder support with nested structure
- GTK4 interface with Omarchy theme integration

---

## Legend

- 🔗 Links & Navigation
- 📂 File Management
- 🏷️ Tags & Organization
- 🔍 Search & Discovery
- 🤖 AI & Automation
- 🎵 Media & Audio
- 🎨 UI/UX Improvements
- ⌨️ Keyboard & Input
- 🔧 Technical Changes
- 📚 Documentation

