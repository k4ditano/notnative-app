# NotNative

Una aplicación de notas **nativa** para escritorio Linux con soporte para markdown, comandos estilo vim y diseñada para máxima velocidad y eficiencia.

## 🎯 Características

### ✅ Implementado (v0.1)

- **Buffer de texto ultrarrápido** basado en `ropey` con operaciones O(log n)
- **Sistema de comandos modal** inspirado en vim (Normal/Insert/Command/Visual)
- **Undo/Redo granular** con historial de 1000 operaciones
- **Interfaz nativa GTK4 + libadwaita** con soporte de temas claro/oscuro
- **Barra de estado** con indicador de modo y estadísticas
- **Eventos de teclado** integrados con el sistema de comandos
- **Sistema de archivos .md** - Cada nota se guarda como archivo markdown independiente
- **Autoguardado inteligente** - Guarda cada 5 segundos y al cerrar (solo si hay cambios)
- **Indicadores visuales** - Muestra `●` en título cuando hay cambios sin guardar
- **Persistencia automática** - Las notas se guardan en `~/.local/share/notnative/notes/`
- **Gestión de notas** - Crear, cargar, guardar y listar notas .md
- **Nota de bienvenida** - Se crea automáticamente la primera vez que se ejecuta la app
- **Título dinámico** - La ventana muestra el nombre de la nota actual
- **Renderizado markdown en tiempo real** - Estilos aplicados con GTK TextTags
- **Soporte de sintaxis** - Headings, bold, italic, código inline/bloque
- **Caracteres especiales** - Todos los caracteres especiales funcionan correctamente (.,!?:;/etc)
- **Autoguardado inteligente** - Guarda automáticamente cada 5 segundos solo si hay cambios
- **Guardado al cerrar** - Los cambios se guardan automáticamente al cerrar la aplicación
- **Indicador visual de cambios** - Muestra `●` en el título cuando hay cambios sin guardar
- **Márgenes optimizados** - Espaciado visual mejorado en TextView y HeaderBar
- **Modo 8BIT** - Cambia toda la interfaz a fuentes retro/pixeladas (VT323) con un solo clic

### 🚧 En desarrollo

- Renderizado markdown incremental con `pulldown-cmark`
- Vista previa markdown opcional
- Sidebar con árbol de carpetas y notas
- Búsqueda y filtrado de notas
- SQLite para indexación (búsqueda full-text, tags)
- Integración Hyprland (layer-shell, IPC, shortcuts globales)
- API de IA con OpenRouter (resúmenes, chat, MCP)

## 🚀 Instalación

### Requisitos

- Rust 1.70+
- GTK4
- libadwaita

### Fuentes (opcional, para Modo 8BIT)

Para usar el **Modo 8BIT** con fuentes retro, instala las fuentes incluidas:

```bash
./install-fonts.sh
```

Esto instalará VT323 (fuente de terminal retro) en tu sistema. Ver `fonts/README.md` para más detalles.

### Compilar

```bash
cargo build --release
```

### Ejecutar

```bash
cargo run --release
```

## ⌨️ Atajos de teclado

### Modo Normal (predeterminado)

- `i` - Entrar en modo INSERT
- `:` - Entrar en modo COMMAND
- `h/j/k/l` - Mover cursor (izq/abajo/arriba/der)
- `0` - Inicio de línea
- `$` - Fin de línea
- `gg` - Inicio del documento
- `G` - Fin del documento
- `x` - Eliminar carácter
- `dd` - Eliminar línea
- `u` - Deshacer
- `Ctrl+z` - Deshacer
- `Ctrl+r` - Rehacer
- `Ctrl+s` - Guardar
- `Ctrl+d` - Cambiar tema

### Modo Insert

- `Esc` - Volver a modo NORMAL
- `Ctrl+s` - Guardar
- Todas las teclas normales insertan texto

### Modo Command

- `:w` - Guardar
- `:q` - Salir
- `:wq` - Guardar y salir
- `:q!` - Salir sin guardar

### Interfaz

- **Botón 8BIT** (footer) - Activa/desactiva el modo retro con fuentes pixeladas
- **Menú Ajustes** (⚙️) - Acceso a preferencias y configuración
- **Indicador de modo** (footer izquierda) - Muestra el modo actual (NORMAL/INSERT)
- **Estadísticas** (footer derecha) - Líneas, palabras y cambios sin guardar

## 🏗️ Arquitectura

```
src/
├── main.rs              # Bootstrap y configuración inicial
├── app.rs               # Lógica principal de UI con Relm4
└── core/
    ├── note_buffer.rs   # Buffer de texto con ropey
    ├── command.rs       # Parser de comandos y acciones
    ├── editor_mode.rs   # Definición de modos de edición
    └── note_file.rs     # Gestión de archivos .md
```

### Sistema de archivos

- **Directorio base**: `~/.local/share/notnative/notes/`
- **Formato**: Cada nota es un archivo `.md` independiente
- **Estructura**: Soporte para carpetas anidadas
- **Backup-friendly**: Los archivos son estándar markdown legible

### Stack tecnológico

- **Rust** - Lenguaje base
- **GTK4** - Toolkit nativo
- **libadwaita** - Componentes modernos y estilización
- **Relm4** - Framework reactivo para GTK4
- **ropey** - Estructura de datos para edición de texto
- **pulldown-cmark** - Parser markdown
- **SQLite** (próximamente) - Persistencia local

## 🎨 Diseño

NotNative está diseñado para ser:

1. **Rápido**: Operaciones de edición en O(log n), sin bloqueos en la UI
2. **Nativo**: Integración completa con el escritorio (Wayland, portales, D-Bus)
3. **Minimalista**: Interfaz limpia, navegación solo con teclado
4. **Extensible**: Arquitectura modular preparada para plugins

## 🔧 Desarrollo

### Tests

```bash
cargo test
```

### Estructura del buffer

El `NoteBuffer` usa `ropey::Rope` internamente:
- Inserciones/eliminaciones: O(log n)
- Conversiones línea↔carácter: O(log n)
- Acceso a líneas: O(log n)
- Undo/redo con stack de operaciones

### Sistema de comandos

```rust
KeyPress → CommandParser → EditorAction → NoteBuffer
```

Flujo:
1. `EventControllerKey` captura teclas
2. `CommandParser` convierte en `EditorAction`
3. `MainApp::execute_action` modifica el `NoteBuffer`
4. UI se actualiza reactivamente

## 📝 Roadmap

- [x] v0.1: Editor de texto funcional con sistema de archivos .md ✅
  - [x] Buffer de texto con ropey
  - [x] Sistema modal vim-style
  - [x] Interfaz GTK4 + libadwaita
  - [x] Persistencia en archivos .md
  - [x] Gestión básica de notas
  - [x] Autoguardado cada 5 segundos
  - [x] Renderizado markdown básico en tiempo real
  - [x] Soporte completo de caracteres especiales

- [ ] v0.2: Mejoras de markdown y navegación
  - [ ] Vista previa markdown en panel lateral (opcional)
  - [ ] Syntax highlighting mejorado para bloques de código
  - [ ] Links clickeables
  - [ ] Imágenes inline
  - [ ] Listas con bullets personalizados

- [ ] v0.3: Sistema de navegación estilo macOS
  - [ ] Sidebar deslizante con árbol de carpetas/notas
  - [ ] Animaciones suaves (fade, slide) con GTK4
  - [ ] Barra de búsqueda superior extensible
  - [ ] Transiciones fluidas entre vistas
  - [ ] Gestos y shortcuts para mostrar/ocultar sidebar

- [ ] v0.4: Indexación y búsqueda avanzada
  - [ ] SQLite para indexación full-text
  - [ ] Búsqueda rápida por contenido y tags
  - [ ] Sistema de tags con autocompletado
  - [ ] Carpetas virtuales y favoritos

- [ ] v0.5: Integración Hyprland
  - [ ] Layer-shell para overlay mode
  - [ ] IPC con Hyprland para window management
  - [ ] Global shortcuts configurables

- [ ] v0.6: API IA con OpenRouter
  - [ ] Resúmenes automáticos de notas
  - [ ] Chat con LLM integrado
  - [ ] Sugerencias contextuales

- [ ] v0.7: MCP server integration
  - [ ] Conectar con Model Context Protocol
  - [ ] Extensiones via MCP

- [ ] v0.8: Sincronización cloud opcional
- [ ] v1.0: Estabilización y empaquetado

## 📜 Licencia

MIT

## 🤝 Contribuir

Las contribuciones son bienvenidas. Por favor, abre un issue primero para discutir cambios mayores.
