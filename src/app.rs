use relm4::adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, adw, component, gtk};
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::{CommandParser, EditorAction, EditorMode, KeyModifiers, NoteBuffer, NotesDirectory, NoteFile, MarkdownParser, StyleType, NotesConfig};

/// Shared user-facing application identifier used by GTK.
pub const APP_ID: &str = "com.notnative.app";

/// High-level preference for the current visual theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    FollowSystem,
    Light,
    Dark,
}

#[derive(Debug)]
pub struct MainApp {
    theme: ThemePreference,
    buffer: NoteBuffer,
    mode: Rc<RefCell<EditorMode>>,
    command_parser: CommandParser,
    cursor_position: usize,
    text_buffer: gtk::TextBuffer,
    mode_label: gtk::Label,
    stats_label: gtk::Label,
    window_title: gtk::Label,
    notes_dir: NotesDirectory,
    current_note: Option<NoteFile>,
    has_unsaved_changes: bool,
    markdown_enabled: bool,
    bit8_mode: bool,
    text_view: gtk::TextView,
    split_view: gtk::Paned,
    notes_list: gtk::ListBox,
    sidebar_visible: bool,
    expanded_folders: std::collections::HashSet<String>,
    is_populating_list: Rc<RefCell<bool>>,
    context_menu: gtk::PopoverMenu,
    context_item_name: Rc<RefCell<String>>,
    context_is_folder: Rc<RefCell<bool>>,
    renaming_item: Rc<RefCell<Option<(String, bool)>>>, // (nombre, es_carpeta)
}

#[derive(Debug)]
pub enum AppMsg {
    ToggleTheme,
    #[allow(dead_code)]
    SetTheme(ThemePreference),
    Toggle8BitMode,
    ToggleSidebar,
    OpenSidebarAndFocus,
    ShowCreateNoteDialog,
    ToggleFolder(String),
    ShowContextMenu(f64, f64, String, bool), // x, y, nombre, es_carpeta
    DeleteItem(String, bool), // nombre, es_carpeta
    RenameItem(String, bool), // nombre, es_carpeta
    RefreshSidebar,
    KeyPress {
        key: String,
        modifiers: KeyModifiers,
    },
    ProcessAction(EditorAction),
    SaveCurrentNote,
    AutoSave,
    LoadNote(String),
    CreateNewNote(String),
    UpdateCursorPosition(usize),
}

#[component(pub)]
impl SimpleComponent for MainApp {
    type Input = AppMsg;
    type Output = ();
    type Init = ThemePreference;

    view! {
        main_window = adw::ApplicationWindow {
            set_title: Some("NotNative"),
            set_default_width: 920,
            set_default_height: 680,

            add_css_class: "compact",

            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                append = header_bar = &adw::HeaderBar {
                    pack_start = &gtk::Button {
                        set_icon_name: "view-list-symbolic",
                        set_tooltip_text: Some("Mostrar/ocultar lista de notas"),
                        add_css_class: "flat",
                        connect_clicked => AppMsg::ToggleSidebar,
                    },
                    
                    #[wrap(Some)]
                    set_title_widget = window_title = &gtk::Label {
                        set_label: "NotNative",
                    },
                },

                append = split_view = &gtk::Paned {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_position: 0,
                    set_vexpand: true,
                    set_wide_handle: false,
                    set_shrink_start_child: true,
                    set_resize_start_child: false,
                    
                    #[wrap(Some)]
                    set_start_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 0,
                        add_css_class: "sidebar",
                        
                        append = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            set_margin_all: 12,
                            
                            append = &gtk::Label {
                                set_label: "Notas",
                                set_xalign: 0.0,
                                set_hexpand: true,
                                add_css_class: "heading",
                            },
                            
                            append = &gtk::Button {
                                set_icon_name: "list-add-symbolic",
                                set_tooltip_text: Some("Nueva nota"),
                                add_css_class: "flat",
                                add_css_class: "circular",
                            },
                        },
                        
                        append = &gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_hexpand: true,
                            set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                            
                            #[wrap(Some)]
                            set_child = notes_list = &gtk::ListBox {
                                add_css_class: "navigation-sidebar",
                                set_selection_mode: gtk::SelectionMode::Single,
                                set_activate_on_single_click: false,
                            },
                        },
                    },
                    
                    #[wrap(Some)]
                    set_end_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,

                        append = &gtk::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Automatic),

                        #[wrap(Some)]
                        set_child = text_view = &gtk::TextView::builder()
                            .monospace(true)
                            .wrap_mode(gtk::WrapMode::WordChar)
                            .editable(true)
                            .cursor_visible(true)
                            .accepts_tab(false)
                            .left_margin(16)
                            .right_margin(16)
                            .top_margin(12)
                            .bottom_margin(12)
                            .build(),
                    },

                        append = status_bar = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            set_margin_all: 6,
                            add_css_class: "status-bar",

                            append = mode_label = &gtk::Label {
                                set_markup: "<b>NORMAL</b>",
                                set_xalign: 0.0,
                            },

                            append = &gtk::Label {
                                set_hexpand: true,
                                set_label: "",
                            },

                            append = stats_label = &gtk::Label {
                                set_label: "0 líneas | 0 palabras",
                                set_xalign: 1.0,
                            },

                            append = &gtk::Box {
                                set_spacing: 4,
                                
                                append = &gtk::Separator {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_margin_start: 8,
                                    set_margin_end: 8,
                                },

                                // TODO: Botón 8BIT desactivado temporalmente
                                // append = bit8_button = &gtk::ToggleButton {
                                //     set_label: "8BIT",
                                //     set_tooltip_text: Some("Modo retro 8-bit"),
                                //     add_css_class: "flat",
                                //     connect_toggled[sender] => move |btn| {
                                //         if btn.is_active() {
                                //             sender.input(AppMsg::Toggle8BitMode);
                                //         } else {
                                //             sender.input(AppMsg::Toggle8BitMode);
                                //         }
                                //     },
                                // },

                                append = settings_button = &gtk::MenuButton {
                                    set_icon_name: "emblem-system-symbolic",
                                    set_tooltip_text: Some("Ajustes"),
                                    add_css_class: "flat",
                                    set_direction: gtk::ArrowType::Up,
                                    
                                    #[wrap(Some)]
                                    set_popover = &gtk::Popover {
                                        add_css_class: "menu",
                                        
                                        #[wrap(Some)]
                                        set_child = &gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_spacing: 0,
                                            set_margin_all: 6,
                                            
                                            append = &gtk::Button {
                                                set_label: "Preferencias",
                                                add_css_class: "flat",
                                                set_halign: gtk::Align::Fill,
                                            },
                                            
                                            append = &gtk::Button {
                                                set_label: "Atajos de teclado",
                                                add_css_class: "flat",
                                                set_halign: gtk::Align::Fill,
                                            },
                                            
                                            append = &gtk::Separator {
                                                set_orientation: gtk::Orientation::Horizontal,
                                                set_margin_top: 4,
                                                set_margin_bottom: 4,
                                            },
                                            
                                            append = &gtk::Button {
                                                set_label: "Acerca de",
                                                add_css_class: "flat",
                                                set_halign: gtk::Align::Fill,
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        theme: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let style_manager = adw::StyleManager::default();
        
        let widgets = view_output!();
        
        let text_buffer = widgets.text_view.buffer();
        let mode = Rc::new(RefCell::new(EditorMode::Normal));
        
        // Inicializar directorio de notas (por defecto ~/.local/share/notnative/notes)
        let notes_dir = NotesDirectory::default();
        
        // Crear menú contextual para el sidebar (sin parent inicialmente)
        let menu = gtk::gio::Menu::new();
        menu.append(Some("Renombrar"), Some("item.rename"));
        menu.append(Some("Eliminar"), Some("item.delete"));
        
        let context_menu = gtk::PopoverMenu::from_model(Some(&menu));
        context_menu.set_has_arrow(false);
        context_menu.add_css_class("context-menu");
        
        // Intentar cargar la nota "bienvenida" o crearla si no existe
        let (initial_buffer, current_note) = match notes_dir.find_note("bienvenida") {
            Ok(Some(note)) => {
                match note.read() {
                    Ok(content) => {
                        println!("Nota 'bienvenida' cargada");
                        (NoteBuffer::from_text(&content), Some(note))
                    }
                    Err(_) => (NoteBuffer::new(), None)
                }
            }
            _ => {
                // Crear nota de bienvenida
                let welcome_content = "# Bienvenido a NotNative

Esta es tu primera nota. NotNative guarda cada nota como un archivo .md independiente.

## Comandos básicos

- `i` → Modo INSERT (editar)
- `Esc` → Modo NORMAL
- `h/j/k/l` → Navegar (izquierda/abajo/arriba/derecha)
- `x` → Eliminar carácter
- `u` → Deshacer
- `Ctrl+S` → Guardar

Las notas se guardan automáticamente en: ~/.local/share/notnative/notes/
";
                match notes_dir.create_note("bienvenida", welcome_content) {
                    Ok(note) => {
                        println!("Nota de bienvenida creada");
                        (NoteBuffer::from_text(welcome_content), Some(note))
                    }
                    Err(_) => (NoteBuffer::new(), None)
                }
            }
        };
        
        let model = MainApp {
            theme,
            buffer: initial_buffer,
            mode: mode.clone(),
            command_parser: CommandParser::new(),
            cursor_position: 0,
            text_buffer: text_buffer.clone(),
            mode_label: widgets.mode_label.clone(),
            stats_label: widgets.stats_label.clone(),
            window_title: widgets.window_title.clone(),
            notes_dir,
            current_note,
            has_unsaved_changes: false,
            markdown_enabled: true, // Ahora con parser robusto usando offsets de pulldown-cmark
            bit8_mode: false,
            text_view: widgets.text_view.clone(),
            split_view: widgets.split_view.clone(),
            notes_list: widgets.notes_list.clone(),
            sidebar_visible: false,
            expanded_folders: std::collections::HashSet::new(),
            is_populating_list: Rc::new(RefCell::new(false)),
            context_menu: context_menu.clone(),
            context_item_name: Rc::new(RefCell::new(String::new())),
            context_is_folder: Rc::new(RefCell::new(false)),
            renaming_item: Rc::new(RefCell::new(None)),
        };

        model.apply_theme(&style_manager);
        
        // Crear acciones para el menú contextual
        let rename_action = gtk::gio::SimpleAction::new("rename", None);
        rename_action.connect_activate(gtk::glib::clone!(@strong sender, @strong model.context_item_name as item_name, @strong model.context_is_folder as is_folder => move |_, _| {
            sender.input(AppMsg::RenameItem(item_name.borrow().clone(), *is_folder.borrow()));
        }));
        
        let delete_action = gtk::gio::SimpleAction::new("delete", None);
        delete_action.connect_activate(gtk::glib::clone!(@strong sender, @strong model.context_item_name as item_name, @strong model.context_is_folder as is_folder => move |_, _| {
            sender.input(AppMsg::DeleteItem(item_name.borrow().clone(), *is_folder.borrow()));
        }));
        
        let action_group = gtk::gio::SimpleActionGroup::new();
        action_group.add_action(&rename_action);
        action_group.add_action(&delete_action);
        context_menu.insert_action_group("item", Some(&action_group));
        
        // Crear tags de estilo para markdown
        model.create_text_tags();
        
        // Sincronizar contenido inicial con la vista
        model.sync_to_view();
        model.update_status_bar(&sender);
        
        // Configurar autoguardado cada 5 segundos
        gtk::glib::timeout_add_seconds_local(5, gtk::glib::clone!(@strong sender => move || {
            sender.input(AppMsg::AutoSave);
            gtk::glib::ControlFlow::Continue
        }));

        let action_group = gtk::gio::SimpleActionGroup::new();
        let toggle_action = gtk::gio::SimpleAction::new("toggle-theme", None);
        toggle_action.connect_activate(gtk::glib::clone!(@strong sender => move |_, _| {
            sender.input(AppMsg::ToggleTheme);
        }));
        action_group.add_action(&toggle_action);
        widgets
            .main_window
            .insert_action_group("app", Some(&action_group));

        let shortcuts = gtk::ShortcutController::new();
        shortcuts.set_scope(gtk::ShortcutScope::Local);
        if let (Some(trigger), Some(action)) = (
            gtk::ShortcutTrigger::parse_string("<Primary>d"),
            gtk::ShortcutAction::parse_string("activate app.toggle-theme"),
        ) {
            let shortcut = gtk::Shortcut::new(Some(trigger), Some(action));
            shortcuts.add_shortcut(shortcut);
        }
        widgets.main_window.add_controller(shortcuts);
        
        // Conectar señal de cierre para guardar antes de cerrar
        widgets.main_window.connect_close_request(gtk::glib::clone!(@strong sender => move |_| {
            sender.input(AppMsg::SaveCurrentNote);
            gtk::glib::Propagation::Proceed
        }));

        // Conectar eventos de teclado al TextView
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(
            gtk::glib::clone!(@strong sender, @strong mode => move |_controller, keyval, _keycode, modifiers| {
                let key_name = keyval.name().map(|s| s.to_string()).unwrap_or_default();
                
                let key_mods = KeyModifiers {
                    ctrl: modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK),
                    alt: modifiers.contains(gtk::gdk::ModifierType::ALT_MASK),
                    shift: modifiers.contains(gtk::gdk::ModifierType::SHIFT_MASK),
                };

                sender.input(AppMsg::KeyPress {
                    key: key_name,
                    modifiers: key_mods,
                });

                // Siempre bloqueamos las teclas para manejarlas nosotros
                // Ya sincronizamos manualmente con el TextView
                gtk::glib::Propagation::Stop
            })
        );
        widgets.text_view.add_controller(key_controller);
        
        // Conectar eventos de clic para actualizar posición del cursor
        let click_controller = gtk::GestureClick::new();
        click_controller.connect_released(
            gtk::glib::clone!(@strong sender, @strong text_buffer => move |_gesture, _n_press, _x, _y| {
                // Obtener la posición del cursor después del clic
                let cursor_mark = text_buffer.get_insert();
                let cursor_iter = text_buffer.iter_at_mark(&cursor_mark);
                let cursor_pos = cursor_iter.offset() as usize;
                
                // Notificar al modelo para actualizar su cursor_position
                sender.input(AppMsg::UpdateCursorPosition(cursor_pos));
            })
        );
        widgets.text_view.add_controller(click_controller);
        
        // Poblar la lista de notas
        model.populate_notes_list(&sender);
        *model.is_populating_list.borrow_mut() = false;
        
        // Conectar evento de cambio de selección en el ListBox
        let is_populating_for_select = model.is_populating_list.clone();
        widgets.notes_list.connect_row_selected(
            gtk::glib::clone!(@strong sender => move |_list_box, row| {
                // No cargar notas si se está repoblando la lista
                if *is_populating_for_select.borrow() {
                    return;
                }
                
                if let Some(row) = row {
                    // Solo cargar si es una fila seleccionable (notas, no carpetas)
                    if !row.is_selectable() {
                        return;
                    }
                    
                    // Obtener el nombre de la nota desde el label del row
                    if let Some(child) = row.child() {
                        if let Ok(box_widget) = child.downcast::<gtk::Box>() {
                            // El label es el segundo hijo (después del icono)
                            if let Some(label_widget) = box_widget.first_child().and_then(|w| w.next_sibling()) {
                                if let Ok(label) = label_widget.downcast::<gtk::Label>() {
                                    let note_name = label.text().to_string();
                                    sender.input(AppMsg::LoadNote(note_name));
                                }
                            }
                        }
                    }
                }
            })
        );
        
        // Conectar click en carpetas para expandir/colapsar
        let folder_click = gtk::GestureClick::new();
        folder_click.connect_released(
            gtk::glib::clone!(@strong widgets.notes_list as notes_list, @strong sender => move |gesture, _n_press, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                
                // Obtener la fila bajo el click
                if let Some(row) = notes_list.row_at_y(y as i32) {
                    // Solo procesar carpetas (no seleccionables)
                    if !row.is_selectable() {
                        if let Some(child) = row.child() {
                            if let Ok(box_widget) = child.downcast::<gtk::Box>() {
                                // Buscar el label de la carpeta
                                let mut current_child = box_widget.first_child();
                                
                                while let Some(widget) = current_child {
                                    let next = widget.next_sibling();
                                    
                                    if let Ok(label) = widget.clone().downcast::<gtk::Label>() {
                                        if label.has_css_class("heading") {
                                            let folder_name = label.text().to_string();
                                            sender.input(AppMsg::ToggleFolder(folder_name));
                                            break;
                                        }
                                    }
                                    current_child = next;
                                }
                            }
                        }
                    }
                }
            })
        );
        widgets.notes_list.add_controller(folder_click);
        
        // Agregar click derecho para menú contextual
        let right_click = gtk::GestureClick::new();
        right_click.set_button(3); // Botón derecho
        right_click.connect_released(
            gtk::glib::clone!(@strong widgets.notes_list as notes_list, @strong sender => move |_, _n_press, x, y| {
                // Obtener la fila bajo el click
                if let Some(row) = notes_list.row_at_y(y as i32) {
                    if let Some(child) = row.child() {
                        if let Ok(box_widget) = child.downcast::<gtk::Box>() {
                            // Buscar el label (nota o carpeta)
                            let mut current_child = box_widget.first_child();
                            
                            while let Some(widget) = current_child {
                                let next = widget.next_sibling();
                                
                                if let Ok(label) = widget.clone().downcast::<gtk::Label>() {
                                    let item_name = label.text().to_string();
                                    let is_folder = label.has_css_class("heading");
                                    sender.input(AppMsg::ShowContextMenu(x, y, item_name, is_folder));
                                    break;
                                }
                                current_child = next;
                            }
                        }
                    }
                }
            })
        );
        widgets.notes_list.add_controller(right_click);
        
        // Agregar hover para cargar notas al pasar el ratón
        let motion_controller = gtk::EventControllerMotion::new();
        let is_populating_clone = model.is_populating_list.clone();
        motion_controller.connect_motion(
            gtk::glib::clone!(@strong widgets.notes_list as notes_list, @strong sender => move |_controller, _x, y| {
                // No cargar notas si se está repoblando la lista
                if *is_populating_clone.borrow() {
                    return;
                }
                
                // Obtener la fila bajo el cursor
                if let Some(row) = notes_list.row_at_y(y as i32) {
                    if row.is_selectable() {
                        // Seleccionar la fila visualmente
                        notes_list.select_row(Some(&row));
                        
                        // Cargar la nota
                        if let Some(child) = row.child() {
                            if let Ok(box_widget) = child.downcast::<gtk::Box>() {
                                if let Some(label_widget) = box_widget.first_child().and_then(|w| w.next_sibling()) {
                                    if let Ok(label) = label_widget.downcast::<gtk::Label>() {
                                        let note_name = label.text().to_string();
                                        sender.input(AppMsg::LoadNote(note_name));
                                    }
                                }
                            }
                        }
                    }
                }
            })
        );
        widgets.notes_list.add_controller(motion_controller);
        
        // Agregar control de teclado al ListBox para navegación con j/k
        let notes_key_controller = gtk::EventControllerKey::new();
        notes_key_controller.connect_key_pressed(
            gtk::glib::clone!(@strong widgets.notes_list as notes_list, @strong sender => move |_controller, keyval, _keycode, _modifiers| {
                let key_name = keyval.name().map(|s| s.to_string()).unwrap_or_default();
                
                match key_name.as_str() {
                    "j" | "Down" => {
                        // Mover a la siguiente nota
                        if let Some(selected_row) = notes_list.selected_row() {
                            let index = selected_row.index();
                            if let Some(next_row) = notes_list.row_at_index(index + 1) {
                                notes_list.select_row(Some(&next_row));
                            }
                        }
                        return gtk::glib::Propagation::Stop;
                    }
                    "k" | "Up" => {
                        // Mover a la nota anterior
                        if let Some(selected_row) = notes_list.selected_row() {
                            let index = selected_row.index();
                            if index > 0 {
                                if let Some(prev_row) = notes_list.row_at_index(index - 1) {
                                    notes_list.select_row(Some(&prev_row));
                                }
                            }
                        }
                        return gtk::glib::Propagation::Stop;
                    }
                    "l" | "Right" | "Escape" => {
                        // Cerrar sidebar y volver al editor
                        sender.input(AppMsg::ToggleSidebar);
                        return gtk::glib::Propagation::Stop;
                    }
                    _ => {}
                }
                
                gtk::glib::Propagation::Proceed
            })
        );
        widgets.notes_list.add_controller(notes_key_controller);
        
        // Dar foco inicial al TextView para que detecte teclas inmediatamente
        widgets.text_view.grab_focus();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: AppMsg, sender: ComponentSender<Self>) {
        match message {
            AppMsg::ToggleTheme => {
                self.theme = match self.theme {
                    ThemePreference::FollowSystem => ThemePreference::Dark,
                    ThemePreference::Light => ThemePreference::Dark,
                    ThemePreference::Dark => ThemePreference::Light,
                };
                self.refresh_style_manager();
            }
            AppMsg::SetTheme(theme) => {
                self.theme = theme;
                self.refresh_style_manager();
            }
            AppMsg::Toggle8BitMode => {
                self.bit8_mode = !self.bit8_mode;
                self.apply_8bit_font();
            }
            AppMsg::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                let target_position = if self.sidebar_visible { 250 } else { 0 };
                self.animate_sidebar(target_position);
                
                // Si estamos cerrando el sidebar, devolver foco al editor
                if !self.sidebar_visible {
                    let text_view = self.text_view.clone();
                    gtk::glib::source::timeout_add_local(
                        std::time::Duration::from_millis(160),
                        move || {
                            text_view.grab_focus();
                            gtk::glib::ControlFlow::Break
                        }
                    );
                }
            }
            AppMsg::OpenSidebarAndFocus => {
                // Abrir sidebar si está cerrado
                if !self.sidebar_visible {
                    self.sidebar_visible = true;
                    self.animate_sidebar(250);
                }
                
                // Dar foco al ListBox después de un pequeño delay para que termine la animación
                let notes_list = self.notes_list.clone();
                gtk::glib::source::timeout_add_local(
                    std::time::Duration::from_millis(160),
                    move || {
                        notes_list.grab_focus();
                        // Seleccionar el primer elemento si no hay nada seleccionado
                        if notes_list.selected_row().is_none() {
                            if let Some(first_row) = notes_list.row_at_index(0) {
                                notes_list.select_row(Some(&first_row));
                            }
                        }
                        gtk::glib::ControlFlow::Break
                    }
                );
            }
            AppMsg::KeyPress { key, modifiers } => {
                let current_mode = *self.mode.borrow();
                let action = match current_mode {
                    EditorMode::Normal => self.command_parser.parse_normal_mode(&key, modifiers),
                    EditorMode::Insert => self.command_parser.parse_insert_mode(&key, modifiers),
                    EditorMode::Command => {
                        // En modo comando, acumular input hasta Enter
                        // Por ahora, simplificamos
                        EditorAction::None
                    }
                    EditorMode::Visual => EditorAction::None,
                };

                if action != EditorAction::None {
                    sender.input(AppMsg::ProcessAction(action));
                }
            }
            AppMsg::ProcessAction(action) => {
                self.execute_action(action, &sender);
            }
            AppMsg::SaveCurrentNote => {
                self.save_current_note();
            }
            AppMsg::AutoSave => {
                // Solo guardar si hay cambios sin guardar
                if self.has_unsaved_changes {
                    self.save_current_note();
                    println!("Autoguardado ejecutado");
                }
            }
            AppMsg::LoadNote(name) => {
                if let Err(e) = self.load_note(&name) {
                    eprintln!("Error cargando nota '{}': {}", name, e);
                } else {
                    // Sincronizar vista y actualizar UI
                    self.sync_to_view();
                    self.update_status_bar(&sender);
                    self.window_title.set_label(&name);
                    self.has_unsaved_changes = false;
                }
            }
            AppMsg::CreateNewNote(name) => {
                if let Err(e) = self.create_new_note(&name) {
                    eprintln!("Error creando nota '{}': {}", name, e);
                } else {
                    // Sincronizar vista y actualizar UI
                    self.sync_to_view();
                    self.update_status_bar(&sender);
                    self.window_title.set_label(&name);
                    
                    // Refrescar lista de notas en el sidebar
                    self.populate_notes_list(&sender);
                    *self.is_populating_list.borrow_mut() = false;
                    
                    // Cambiar a modo Insert para empezar a escribir
                    *self.mode.borrow_mut() = EditorMode::Insert;
                }
            }
            AppMsg::UpdateCursorPosition(pos) => {
                // Actualizar la posición del cursor cuando el usuario hace clic
                self.cursor_position = pos;
            }
            AppMsg::ShowCreateNoteDialog => {
                self.show_create_note_dialog(&sender);
            }
            
            AppMsg::ToggleFolder(folder_name) => {
                // Forzar desactivación del flag si había uno pendiente
                *self.is_populating_list.borrow_mut() = false;
                
                // Toggle el estado de la carpeta
                if self.expanded_folders.contains(&folder_name) {
                    self.expanded_folders.remove(&folder_name);
                } else {
                    self.expanded_folders.insert(folder_name);
                }
                
                // Refrescar la lista para mostrar/ocultar las notas
                self.populate_notes_list(&sender);
                
                // Mantener el flag activo brevemente para evitar hover inmediato
                let is_populating_clone = self.is_populating_list.clone();
                gtk::glib::source::timeout_add_local(
                    std::time::Duration::from_millis(50),
                    move || {
                        *is_populating_clone.borrow_mut() = false;
                        gtk::glib::ControlFlow::Break
                    }
                );
            }
            
            AppMsg::ShowContextMenu(x, y, item_name, is_folder) => {
                *self.context_item_name.borrow_mut() = item_name;
                *self.context_is_folder.borrow_mut() = is_folder;
                
                // Establecer parent solo cuando se va a mostrar
                self.context_menu.set_parent(&self.notes_list);
                
                let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                self.context_menu.set_pointing_to(Some(&rect));
                self.context_menu.popup();
            }
            
            AppMsg::DeleteItem(item_name, is_folder) => {
                self.context_menu.popdown();
                self.context_menu.unparent();
                
                if is_folder {
                    println!("Eliminar carpeta: {}", item_name);
                    // TODO: Implementar eliminación de carpeta
                } else {
                    println!("Eliminar nota: {}", item_name);
                    if let Ok(Some(note)) = self.notes_dir.find_note(&item_name) {
                        if let Err(e) = std::fs::remove_file(note.path()) {
                            eprintln!("Error al eliminar nota: {}", e);
                        } else {
                            // Si era la nota actual, limpiar el editor
                            if let Some(current) = &self.current_note {
                                if current.name() == item_name {
                                    self.current_note = None;
                                    self.buffer = NoteBuffer::new();
                                    self.sync_to_view();
                                    self.window_title.set_label("NotNative");
                                }
                            }
                            // Refrescar sidebar
                            self.populate_notes_list(&sender);
                            *self.is_populating_list.borrow_mut() = false;
                        }
                    }
                }
            }
            
            AppMsg::RenameItem(item_name, is_folder) => {
                self.context_menu.popdown();
                self.context_menu.unparent();
                
                // Activar modo de renombrado
                *self.renaming_item.borrow_mut() = Some((item_name, is_folder));
                
                // Repoblar la lista para mostrar el Entry editable
                self.populate_notes_list(&sender);
            }
            
            AppMsg::RefreshSidebar => {
                self.populate_notes_list(&sender);
                *self.is_populating_list.borrow_mut() = false;
            }
        }
    }
}

impl MainApp {
    fn execute_action(&mut self, action: EditorAction, sender: &ComponentSender<Self>) {
        // Verificar si hay una selección activa
        let selection_bounds = self.text_buffer.selection_bounds();
        let has_selection = selection_bounds.is_some();
        
        match action {
            EditorAction::ChangeMode(new_mode) => {
                *self.mode.borrow_mut() = new_mode;
                println!("Cambiado a modo: {:?}", new_mode);
            }
            EditorAction::InsertChar(ch) => {
                // Si hay selección, primero borrarla
                if has_selection {
                    self.delete_selection();
                }
                self.buffer.insert(self.cursor_position, &ch.to_string());
                self.cursor_position += 1;
                self.has_unsaved_changes = true;
            }
            EditorAction::InsertNewline => {
                // Si hay selección, primero borrarla
                if has_selection {
                    self.delete_selection();
                }
                self.buffer.insert(self.cursor_position, "\n");
                self.cursor_position += 1;
                self.has_unsaved_changes = true;
            }
            EditorAction::DeleteCharBefore => {
                if has_selection {
                    // Borrar selección
                    self.delete_selection();
                } else if self.cursor_position > 0 {
                    // Borrar un carácter antes del cursor
                    self.buffer.delete(self.cursor_position - 1..self.cursor_position);
                    self.cursor_position -= 1;
                    self.has_unsaved_changes = true;
                }
            }
            EditorAction::DeleteCharAfter => {
                if has_selection {
                    // Borrar selección
                    self.delete_selection();
                } else if self.cursor_position < self.buffer.len_chars() {
                    // Borrar un carácter después del cursor
                    self.buffer.delete(self.cursor_position..self.cursor_position + 1);
                    self.has_unsaved_changes = true;
                }
            }
            EditorAction::DeleteSelection => {
                if has_selection {
                    self.delete_selection();
                }
            }
            EditorAction::MoveCursorLeft => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            EditorAction::MoveCursorRight => {
                if self.cursor_position < self.buffer.len_chars() {
                    self.cursor_position += 1;
                }
            }
            EditorAction::MoveCursorUp => {
                // Obtener la línea actual y columna
                let line = self.buffer.rope().char_to_line(self.cursor_position);
                if line > 0 {
                    // Ir a la línea anterior
                    let prev_line = line - 1;
                    let line_start = self.buffer.rope().line_to_char(prev_line);
                    let line_end = if prev_line < self.buffer.len_lines() - 1 {
                        self.buffer.rope().line_to_char(prev_line + 1).saturating_sub(1)
                    } else {
                        self.buffer.len_chars()
                    };
                    
                    // Intentar mantener la columna, pero no exceder el largo de la línea
                    let current_line_start = self.buffer.rope().line_to_char(line);
                    let col_in_line = self.cursor_position - current_line_start;
                    let prev_line_len = line_end - line_start;
                    
                    self.cursor_position = line_start + col_in_line.min(prev_line_len);
                }
            }
            EditorAction::MoveCursorDown => {
                // Obtener la línea actual y columna
                let line = self.buffer.rope().char_to_line(self.cursor_position);
                if line < self.buffer.len_lines() - 1 {
                    // Ir a la línea siguiente
                    let next_line = line + 1;
                    let line_start = self.buffer.rope().line_to_char(next_line);
                    let line_end = if next_line < self.buffer.len_lines() - 1 {
                        self.buffer.rope().line_to_char(next_line + 1).saturating_sub(1)
                    } else {
                        self.buffer.len_chars()
                    };
                    
                    // Intentar mantener la columna, pero no exceder el largo de la línea
                    let current_line_start = self.buffer.rope().line_to_char(line);
                    let col_in_line = self.cursor_position - current_line_start;
                    let next_line_len = line_end - line_start;
                    
                    self.cursor_position = line_start + col_in_line.min(next_line_len);
                }
            }
            EditorAction::MoveCursorLineStart => {
                let line = self.buffer.rope().char_to_line(self.cursor_position);
                self.cursor_position = self.buffer.rope().line_to_char(line);
            }
            EditorAction::MoveCursorLineEnd => {
                let line = self.buffer.rope().char_to_line(self.cursor_position);
                let line_start = self.buffer.rope().line_to_char(line);
                let line_end = if line < self.buffer.len_lines() - 1 {
                    self.buffer.rope().line_to_char(line + 1).saturating_sub(1)
                } else {
                    self.buffer.len_chars()
                };
                self.cursor_position = line_end;
            }
            EditorAction::MoveCursorDocStart => {
                self.cursor_position = 0;
            }
            EditorAction::MoveCursorDocEnd => {
                self.cursor_position = self.buffer.len_chars();
            }
            EditorAction::Undo => {
                self.buffer.undo();
                self.has_unsaved_changes = true;
            }
            EditorAction::Redo => {
                self.buffer.redo();
                self.has_unsaved_changes = true;
            }
            EditorAction::Save => {
                sender.input(AppMsg::SaveCurrentNote);
            }
            EditorAction::OpenSidebar => {
                sender.input(AppMsg::OpenSidebarAndFocus);
            }
            EditorAction::CreateNote => {
                sender.input(AppMsg::ShowCreateNoteDialog);
            }
            _ => {
                println!("Acción no implementada: {:?}", action);
            }
        }
        
        // Sincronizar el buffer con GTK TextView
        self.sync_to_view();
        
        // Actualizar barra de estado
        self.update_status_bar(sender);
    }
    
    fn sync_to_view(&self) {
        let buffer_text = self.buffer.to_string();
        let current_mode = *self.mode.borrow();
        
        // En modo Normal, mostrar texto limpio (sin símbolos markdown)
        // En modo Insert, mostrar texto crudo con todos los símbolos
        let display_text = if current_mode == EditorMode::Normal && self.markdown_enabled {
            self.render_clean_markdown(&buffer_text)
        } else {
            buffer_text.clone()
        };
        
        // Solo actualizar si el texto es diferente
        let current_text = self.text_buffer.text(
            &self.text_buffer.start_iter(),
            &self.text_buffer.end_iter(),
            false
        ).to_string();
        
        if current_text != display_text {
            // Calcular posición del cursor en el texto mostrado
            let cursor_offset = if current_mode == EditorMode::Normal && self.markdown_enabled {
                // Mapear posición del buffer original al texto limpio
                self.map_buffer_pos_to_display(&buffer_text, self.cursor_position)
            } else {
                self.cursor_position.min(self.buffer.len_chars())
            };
            
            // Bloquear señales GTK durante la actualización
            self.text_buffer.begin_user_action();
            
            // Reemplazar todo el contenido
            let start_iter = self.text_buffer.start_iter();
            let end_iter = self.text_buffer.end_iter();
            self.text_buffer.delete(&mut start_iter.clone(), &mut end_iter.clone());
            self.text_buffer.insert(&mut self.text_buffer.start_iter(), &display_text);
            
            // Restaurar cursor ANTES de terminar la acción de usuario
            let mut iter = self.text_buffer.start_iter();
            iter.set_offset(cursor_offset as i32);
            self.text_buffer.place_cursor(&iter);
            
            self.text_buffer.end_user_action();
            
            // Aplicar estilos markdown DESPUÉS de terminar la acción de usuario
            // Solo en modo Normal (en Insert mode no aplicamos estilos)
            if current_mode == EditorMode::Normal && self.markdown_enabled {
                self.apply_markdown_styles_to_clean_text(&display_text);
            }
        } else {
            // Aunque el texto no cambió, actualizar la posición del cursor
            let cursor_offset = if current_mode == EditorMode::Normal && self.markdown_enabled {
                // Mapear posición del buffer original al texto limpio
                self.map_buffer_pos_to_display(&buffer_text, self.cursor_position)
            } else {
                self.cursor_position.min(self.buffer.len_chars())
            };
            let mut iter = self.text_buffer.start_iter();
            iter.set_offset(cursor_offset as i32);
            self.text_buffer.place_cursor(&iter);
        }
    }
    
    /// Renderiza el texto markdown sin los símbolos de formato
    fn render_clean_markdown(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                // Encabezados: remover #
                '#' if result.is_empty() || result.ends_with('\n') => {
                    // Contar cuántos # hay
                    let mut hash_count = 1;
                    while chars.peek() == Some(&'#') {
                        chars.next();
                        hash_count += 1;
                    }
                    // Saltar espacio después de #
                    if chars.peek() == Some(&' ') {
                        chars.next();
                    }
                }
                // Negrita: remover **
                '*' if chars.peek() == Some(&'*') => {
                    chars.next(); // Consumir el segundo *
                }
                // Cursiva: remover *
                '*' => {
                    // Omitir el *
                }
                // Código inline: remover `
                '`' => {
                    // Omitir el `
                }
                // Todo lo demás: mantener
                _ => result.push(ch),
            }
        }
        
        result
    }
    
    /// Mapea una posición del buffer original al texto limpio (sin símbolos markdown)
    fn map_buffer_pos_to_display(&self, original_text: &str, buffer_pos: usize) -> usize {
        let mut display_pos = 0;
        let mut original_pos = 0;
        let mut chars = original_text.chars().peekable();
        
        while original_pos < buffer_pos && chars.peek().is_some() {
            let ch = chars.next().unwrap();
            original_pos += 1;
            
            match ch {
                // Encabezados: saltar #
                '#' if display_pos == 0 || original_text[..original_pos - 1].ends_with('\n') => {
                    // Contar cuántos # hay
                    while chars.peek() == Some(&'#') && original_pos < buffer_pos {
                        chars.next();
                        original_pos += 1;
                    }
                    // Saltar espacio después de #
                    if chars.peek() == Some(&' ') && original_pos < buffer_pos {
                        chars.next();
                        original_pos += 1;
                    }
                }
                // Negrita: saltar **
                '*' if chars.peek() == Some(&'*') => {
                    if original_pos < buffer_pos {
                        chars.next();
                        original_pos += 1;
                    }
                }
                // Cursiva o código: saltar * o `
                '*' | '`' => {
                    // No incrementar display_pos
                }
                // Todo lo demás: mantener
                _ => {
                    display_pos += 1;
                }
            }
        }
        
        display_pos.min(self.render_clean_markdown(original_text).chars().count())
    }
    
    /// Aplica estilos markdown al texto limpio (sin símbolos)
    fn apply_markdown_styles_to_clean_text(&self, clean_text: &str) {
        let start = self.text_buffer.start_iter();
        let end = self.text_buffer.end_iter();
        self.text_buffer.remove_all_tags(&start, &end);
        
        // Aplicar estilos línea por línea
        let mut line_start_offset = 0;
        for line in clean_text.lines() {
            let line_len = line.chars().count();
            
            // Detectar encabezados por tamaño de fuente (ya no tienen #)
            // Necesitamos usar el texto original para detectar el nivel
            let original_text = self.buffer.to_string();
            let original_lines: Vec<&str> = original_text.lines().collect();
            let clean_lines: Vec<&str> = clean_text.lines().collect();
            
            if let Some(line_idx) = clean_lines.iter().position(|&l| l == line) {
                if let Some(original_line) = original_lines.get(line_idx) {
                    let tag_name = if original_line.starts_with("### ") {
                        Some("h3")
                    } else if original_line.starts_with("## ") {
                        Some("h2")
                    } else if original_line.starts_with("# ") {
                        Some("h1")
                    } else {
                        None
                    };
                    
                    if let Some(tag) = tag_name {
                        let mut start_iter = self.text_buffer.start_iter();
                        start_iter.set_offset(line_start_offset as i32);
                        let mut end_iter = start_iter.clone();
                        end_iter.forward_chars(line_len as i32);
                        
                        if let Some(text_tag) = self.text_buffer.tag_table().lookup(tag) {
                            self.text_buffer.apply_tag(&text_tag, &start_iter, &end_iter);
                        }
                    }
                }
            }
            
            line_start_offset += line_len + 1; // +1 para el \n
        }
    }
    
    fn create_text_tags(&self) {
        let tag_table = self.text_buffer.tag_table();
        
        // Heading 1 - Más grande y en negrita
        let h1_tag = gtk::TextTag::new(Some("h1"));
        h1_tag.set_weight(800);
        h1_tag.set_scale(1.8);
        tag_table.add(&h1_tag);
        
        // Heading 2
        let h2_tag = gtk::TextTag::new(Some("h2"));
        h2_tag.set_weight(700);
        h2_tag.set_scale(1.5);
        tag_table.add(&h2_tag);
        
        // Heading 3
        let h3_tag = gtk::TextTag::new(Some("h3"));
        h3_tag.set_weight(700);
        h3_tag.set_scale(1.25);
        tag_table.add(&h3_tag);
        
        // Bold
        let bold_tag = gtk::TextTag::new(Some("bold"));
        bold_tag.set_weight(700);
        tag_table.add(&bold_tag);
        
        // Italic
        let italic_tag = gtk::TextTag::new(Some("italic"));
        italic_tag.set_style(gtk::pango::Style::Italic);
        tag_table.add(&italic_tag);
        
        // Code inline - fondo gris sutil
        let code_tag = gtk::TextTag::new(Some("code"));
        code_tag.set_family(Some("monospace"));
        code_tag.set_background_rgba(Some(&gtk::gdk::RGBA::new(0.5, 0.5, 0.5, 0.15)));
        code_tag.set_size_points(10.0);
        tag_table.add(&code_tag);
        
        // Code block
        let codeblock_tag = gtk::TextTag::new(Some("codeblock"));
        codeblock_tag.set_family(Some("monospace"));
        codeblock_tag.set_background_rgba(Some(&gtk::gdk::RGBA::new(0.5, 0.5, 0.5, 0.15)));
        codeblock_tag.set_left_margin(20);
        codeblock_tag.set_size_points(10.0);
        tag_table.add(&codeblock_tag);
    }
    
    fn apply_markdown_styles(&self) {
        // Primero remover todos los tags existentes
        let start = self.text_buffer.start_iter();
        let end = self.text_buffer.end_iter();
        self.text_buffer.remove_all_tags(&start, &end);
        
        let text = self.buffer.to_string();
        let parser = MarkdownParser::new(text.clone());
        let styles = parser.parse();
        
        for style in styles {
            // Convertir byte offset a char offset
            let char_start = text[..style.start.min(text.len())]
                .chars()
                .count();
            let char_end = text[..style.end.min(text.len())]
                .chars()
                .count();
            
            let mut start_iter = self.text_buffer.start_iter();
            start_iter.set_offset(char_start as i32);
            
            let mut end_iter = self.text_buffer.start_iter();
            end_iter.set_offset(char_end as i32);
            
            let tag_name = match style.style_type {
                StyleType::Heading1 => "h1",
                StyleType::Heading2 => "h2",
                StyleType::Heading3 => "h3",
                StyleType::Bold => "bold",
                StyleType::Italic => "italic",
                StyleType::Code => "code",
                StyleType::CodeBlock => "codeblock",
                _ => continue,
            };
            
            if let Some(tag) = self.text_buffer.tag_table().lookup(tag_name) {
                self.text_buffer.apply_tag(&tag, &start_iter, &end_iter);
            }
        }
    }
    
    fn update_status_bar(&self, _sender: &ComponentSender<Self>) {
        let line_count = self.buffer.len_lines();
        let word_count = self.buffer.to_string().split_whitespace().count();
        let current_mode = *self.mode.borrow();
        
        // Actualizar etiqueta de modo
        let mode_text = match current_mode {
            EditorMode::Normal => "<b>NORMAL</b>",
            EditorMode::Insert => "<b>INSERT</b>",
            EditorMode::Command => "<b>COMMAND</b>",
            EditorMode::Visual => "<b>VISUAL</b>",
        };
        self.mode_label.set_markup(mode_text);
        
        // Actualizar estadísticas con indicador de cambios sin guardar
        let unsaved_indicator = if self.has_unsaved_changes { " •" } else { "" };
        self.stats_label.set_label(&format!("{} líneas | {} palabras{}", line_count, word_count, unsaved_indicator));
        
        // Actualizar título de ventana con nombre de nota e indicador de cambios
        let title = if let Some(note) = &self.current_note {
            let modified_marker = if self.has_unsaved_changes { "● " } else { "" };
            format!("{}{} - NotNative", modified_marker, note.name())
        } else {
            "Sin título - NotNative".to_string()
        };
        self.window_title.set_text(&title);
        
        println!("Modo: {:?} | {} líneas | {} palabras", current_mode, line_count, word_count);
    }

    fn refresh_style_manager(&self) {
        let style_manager = adw::StyleManager::default();
        self.apply_theme(&style_manager);
    }

    fn apply_8bit_font(&self) {
        if self.bit8_mode {
            // Modo 8BIT activado - aplicar fuente retro a toda la app
            let css = r#"
                /* Fuentes 8-bit para toda la aplicación */
                window, textview, textview text, label, button, headerbar {
                    font-family: "VT323", "Press Start 2P", "Px437 IBM VGA8", "Perfect DOS VGA 437", "unifont", monospace;
                }
                
                /* TextView con fuente 8-bit - tamaño ajustado para VT323 */
                textview, textview text {
                    font-size: 13px;
                    line-height: 1.5;
                    letter-spacing: 0px;
                }
                
                /* Labels del footer más grandes y legibles */
                .status-bar label {
                    font-size: 1.15em;
                    letter-spacing: 0.5px;
                }
                
                /* Botones y header */
                headerbar, button {
                    font-size: 1.0em;
                }
                
                /* Togglebutton 8BIT específico */
                .status-bar togglebutton {
                    font-size: 1.15em;
                    letter-spacing: 0.5px;
                }
            "#;
            
            let css_provider = gtk::CssProvider::new();
            css_provider.load_from_data(css);
            
            gtk::style_context_add_provider_for_display(
                &gtk::gdk::Display::default().unwrap(),
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_USER,
            );
            
            println!("Modo 8BIT activado - Fuentes retro aplicadas");
        } else {
            // Modo normal - restaurar fuentes por defecto
            let css = r#"
                /* Restaurar fuentes normales */
                window, label, button, headerbar {
                    font-family: inherit;
                }
                
                textview, textview text {
                    font-family: monospace;
                    font-size: 11pt;
                    line-height: 1.5;
                    letter-spacing: 0px;
                }
                
                .status-bar label {
                    font-size: 0.8em;
                    letter-spacing: 0px;
                }
                
                headerbar, button {
                    font-family: inherit;
                    font-size: inherit;
                }
                
                .status-bar togglebutton {
                    font-size: inherit;
                    letter-spacing: 0px;
                }
            "#;
            
            let css_provider = gtk::CssProvider::new();
            css_provider.load_from_data(css);
            
            gtk::style_context_add_provider_for_display(
                &gtk::gdk::Display::default().unwrap(),
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_USER,
            );
            
            println!("Modo normal restaurado");
        }
    }

    fn apply_theme(&self, style_manager: &adw::StyleManager) {
        use ThemePreference::*;

        let scheme = match self.theme {
            FollowSystem => adw::ColorScheme::Default,
            Light => adw::ColorScheme::ForceLight,
            Dark => adw::ColorScheme::ForceDark,
        };

        style_manager.set_color_scheme(scheme);
    }
    
    fn animate_sidebar(&self, target_position: i32) {
        let split_view = self.split_view.clone();
        let current_position = split_view.position();
        let distance = (target_position - current_position).abs();
        let steps = 15;
        let step_size = distance / steps;
        let direction = if target_position > current_position { 1 } else { -1 };
        
        let mut step_count = 0;
        gtk::glib::source::timeout_add_local(
            std::time::Duration::from_millis(10),
            move || {
                step_count += 1;
                let current = split_view.position();
                let next_position = if step_count >= steps {
                    target_position
                } else {
                    current + (step_size * direction)
                };
                
                split_view.set_position(next_position);
                
                if step_count >= steps {
                    gtk::glib::ControlFlow::Break
                } else {
                    gtk::glib::ControlFlow::Continue
                }
            }
        );
    }
    
    /// Borra el texto seleccionado
    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.text_buffer.selection_bounds() {
            let start_offset = start.offset() as usize;
            let end_offset = end.offset() as usize;
            
            // Borrar el rango del buffer interno
            self.buffer.delete(start_offset..end_offset);
            
            // Mover el cursor al inicio de la selección
            self.cursor_position = start_offset;
            
            self.has_unsaved_changes = true;
        }
    }
    
    /// Guarda la nota actual en su archivo .md
    fn save_current_note(&mut self) {
        if let Some(note) = &self.current_note {
            let content = self.buffer.to_string();
            if let Err(e) = note.write(&content) {
                eprintln!("Error guardando nota: {}", e);
            } else {
                println!("Nota guardada: {}", note.name());
                self.has_unsaved_changes = false;
            }
        } else {
            // Si no hay nota actual, crear una nueva con timestamp
            let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
            let name = format!("nota_{}", timestamp);
            if let Err(e) = self.create_new_note(&name) {
                eprintln!("Error creando nota automática: {}", e);
            }
        }
    }
    
    /// Carga una nota desde archivo
    fn load_note(&mut self, name: &str) -> anyhow::Result<()> {
        let note = self.notes_dir.find_note(name)?
            .ok_or_else(|| anyhow::anyhow!("Nota no encontrada: {}", name))?;
        
        let content = note.read()?;
        self.buffer = NoteBuffer::from_text(&content);
        self.cursor_position = 0;
        self.current_note = Some(note);
        
        println!("Nota cargada: {}", name);
        Ok(())
    }
    
    /// Crea una nueva nota
    fn create_new_note(&mut self, name: &str) -> anyhow::Result<()> {
        // Contenido inicial vacío para nueva nota
        let initial_content = format!("# {}\n\n", name.split('/').last().unwrap_or(name));
        
        let note = if name.contains('/') {
            // Si contiene '/', separar carpeta y nombre
            let parts: Vec<&str> = name.rsplitn(2, '/').collect();
            let note_name = parts[0];
            let folder = parts[1];
            self.notes_dir.create_note_in_folder(folder, note_name, &initial_content)?
        } else {
            // Crear en la raíz
            self.notes_dir.create_note(name, &initial_content)?
        };
        
        // Cargar la nueva nota en el buffer
        self.buffer = NoteBuffer::from_text(&initial_content);
        self.cursor_position = initial_content.len();
        self.current_note = Some(note.clone());
        self.has_unsaved_changes = false;
        
        println!("Nueva nota creada: {}", name);
        Ok(())
    }
    
    /// Rellena la lista de notas en el sidebar
    fn populate_notes_list(&self, sender: &ComponentSender<Self>) {
        use std::collections::HashMap;
        
        // Activar flag para evitar que el hover cargue notas durante la repoblación
        *self.is_populating_list.borrow_mut() = true;
        
        // Guardar la nota actual para re-seleccionarla después
        let current_note_name = self.current_note.as_ref().map(|n| n.name().to_string());
        
        // Deseleccionar cualquier fila actual
        self.notes_list.select_row(gtk::ListBoxRow::NONE);
        
        // Limpiar lista actual (solo ListBoxRows, no el popover)
        let mut child = self.notes_list.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling();
            if widget.type_().name() == "GtkListBoxRow" {
                self.notes_list.remove(&widget);
            }
            child = next;
        }
        
        // Obtener todas las notas
        if let Ok(notes) = self.notes_dir.list_notes() {
            // Organizar por carpetas
            let mut by_folder: HashMap<String, Vec<String>> = HashMap::new();
            
            for note in notes {
                let note_name = note.name().to_string();
                let relative_path = note.path()
                    .strip_prefix(self.notes_dir.root())
                    .ok()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.to_str())
                    .unwrap_or("");
                
                let folder = if relative_path.is_empty() {
                    String::from("/")
                } else {
                    relative_path.to_string()
                };
                
                by_folder.entry(folder).or_insert_with(Vec::new).push(note_name);
            }
            
            // Ordenar carpetas y notas
            let mut folders: Vec<_> = by_folder.keys().cloned().collect();
            folders.sort();
            
            for folder in folders {
                if let Some(notes_in_folder) = by_folder.get(&folder) {
                    // Si no es la raíz, mostrar carpeta como encabezado expandible
                    if folder != "/" {
                        let is_expanded = self.expanded_folders.contains(&folder);
                        let arrow_icon = if is_expanded { "pan-down-symbolic" } else { "pan-end-symbolic" };
                        
                        let folder_row = gtk::Box::builder()
                            .orientation(gtk::Orientation::Horizontal)
                            .spacing(6)
                            .margin_start(8)
                            .margin_end(12)
                            .margin_top(6)
                            .margin_bottom(4)
                            .build();
                        
                        let arrow = gtk::Image::builder()
                            .icon_name(arrow_icon)
                            .pixel_size(12)
                            .build();
                        
                        let folder_icon = gtk::Image::builder()
                            .icon_name("folder-symbolic")
                            .pixel_size(16)
                            .build();
                        
                        let folder_label = gtk::Label::builder()
                            .label(&folder)
                            .xalign(0.0)
                            .hexpand(true)
                            .build();
                        
                        folder_label.add_css_class("heading");
                        
                        folder_row.append(&arrow);
                        folder_row.append(&folder_icon);
                        folder_row.append(&folder_label);
                        
                        // Agregar como row activatable para click
                        let list_row = gtk::ListBoxRow::builder()
                            .selectable(false)
                            .activatable(true)
                            .build();
                        list_row.set_child(Some(&folder_row));
                        self.notes_list.append(&list_row);
                        
                        // Si no está expandida, no mostrar las notas
                        if !is_expanded {
                            continue;
                        }
                    }
                    
                    // Mostrar notas de esta carpeta (solo si está expandida)
                    let mut sorted_notes = notes_in_folder.clone();
                    sorted_notes.sort();
                    
                    for note_name in sorted_notes {
                        let row = gtk::Box::builder()
                            .orientation(gtk::Orientation::Horizontal)
                            .spacing(8)
                            .margin_start(if folder == "/" { 12 } else { 32 })
                            .margin_end(12)
                            .margin_top(3)
                            .margin_bottom(3)
                            .build();
                        
                        let icon = gtk::Image::builder()
                            .icon_name("text-x-generic-symbolic")
                            .pixel_size(14)
                            .build();
                        
                        row.append(&icon);
                        
                        // Clonar note_name para uso posterior
                        let note_name_str = note_name.as_str();
                        let note_name_owned = note_name.to_string();
                        
                        // Verificar si esta nota está siendo renombrada
                        let is_renaming = self.renaming_item.borrow()
                            .as_ref()
                            .map(|(name, is_folder)| !is_folder && name.as_str() == note_name_str)
                            .unwrap_or(false);
                        
                        if is_renaming {
                            // Mostrar Entry editable
                            let entry = gtk::Entry::builder()
                                .text(&note_name_owned)
                                .hexpand(true)
                                .build();
                            
                            // Al presionar Enter, renombrar
                            let old_name = note_name_owned.clone();
                            let renaming_clone = self.renaming_item.clone();
                            let notes_dir = self.notes_dir.clone();
                            let sender_clone = sender.clone();
                            
                            entry.connect_activate(move |entry| {
                                let new_name = entry.text().to_string().trim().to_string();
                                if !new_name.is_empty() && new_name != old_name {
                                    // Renombrar archivo
                                    if let Ok(Some(note)) = notes_dir.find_note(&old_name) {
                                        let old_path = note.path();
                                        
                                        // Construir nuevo path (misma carpeta)
                                        let new_path = if let Some(parent) = old_path.parent() {
                                            parent.join(format!("{}.md", new_name))
                                        } else {
                                            notes_dir.root().join("notes").join(format!("{}.md", new_name))
                                        };
                                        
                                        if let Err(e) = std::fs::rename(old_path, &new_path) {
                                            eprintln!("Error al renombrar: {}", e);
                                        }
                                    }
                                }
                                
                                // Desactivar modo renombrado
                                *renaming_clone.borrow_mut() = None;
                                
                                // Refrescar sidebar
                                sender_clone.input(AppMsg::RefreshSidebar);
                            });
                            
                            // Al perder foco, cancelar renombrado
                            let focus_controller = gtk::EventControllerFocus::new();
                            let renaming_clone2 = self.renaming_item.clone();
                            let sender_clone2 = sender.clone();
                            focus_controller.connect_leave(move |_| {
                                *renaming_clone2.borrow_mut() = None;
                                sender_clone2.input(AppMsg::RefreshSidebar);
                            });
                            entry.add_controller(focus_controller);
                            
                            row.append(&entry);
                            
                            // Dar foco al entry
                            gtk::glib::source::timeout_add_local(
                                std::time::Duration::from_millis(50),
                                gtk::glib::clone!(@strong entry => move || {
                                    entry.grab_focus();
                                    entry.select_region(0, -1);
                                    gtk::glib::ControlFlow::Break
                                })
                            );
                        } else {
                            // Mostrar Label normal
                            let label = gtk::Label::builder()
                                .label(&note_name_owned)
                                .xalign(0.0)
                                .hexpand(true)
                                .ellipsize(gtk::pango::EllipsizeMode::End)
                                .build();
                            
                            row.append(&label);
                        }
                        
                        self.notes_list.append(&row);
                    }
                }
            }
        }
        
        // Re-seleccionar la nota actual si existía
        if let Some(note_name) = current_note_name {
            // Buscar la fila con esta nota
            let mut current_row = self.notes_list.first_child();
            while let Some(row) = current_row {
                if let Ok(list_row) = row.clone().downcast::<gtk::ListBoxRow>() {
                    if list_row.is_selectable() {
                        if let Some(child) = list_row.child() {
                            if let Ok(box_widget) = child.downcast::<gtk::Box>() {
                                if let Some(label_widget) = box_widget.first_child().and_then(|w| w.next_sibling()) {
                                    if let Ok(label) = label_widget.downcast::<gtk::Label>() {
                                        if label.text() == note_name {
                                            self.notes_list.select_row(Some(&list_row));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                current_row = row.next_sibling();
            }
        }
        
        // NO desactivar flag aquí - se hace con timeout en ToggleFolder
        // o manualmente en otros contextos
    }
    
    /// Muestra un diálogo para crear una nueva nota
    fn show_create_note_dialog(&self, sender: &ComponentSender<Self>) {
        let dialog = gtk::Window::builder()
            .title("Nueva nota")
            .modal(true)
            .default_width(400)
            .default_height(150)
            .resizable(false)
            .build();
        
        let content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(16)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .build();
        
        let label = gtk::Label::builder()
            .label("Nombre de la nota (usa '/' para carpetas)")
            .xalign(0.0)
            .build();
        
        let entry = gtk::Entry::builder()
            .placeholder_text("ejemplo: proyectos/nueva-idea")
            .build();
        
        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .halign(gtk::Align::End)
            .build();
        
        let cancel_button = gtk::Button::builder()
            .label("Cancelar")
            .build();
        
        let create_button = gtk::Button::builder()
            .label("Crear")
            .build();
        
        create_button.add_css_class("suggested-action");
        
        button_box.append(&cancel_button);
        button_box.append(&create_button);
        
        content_box.append(&label);
        content_box.append(&entry);
        content_box.append(&button_box);
        
        dialog.set_child(Some(&content_box));
        
        // Conectar botones
        cancel_button.connect_clicked(gtk::glib::clone!(@strong dialog => move |_| {
            dialog.close();
        }));
        
        create_button.connect_clicked(
            gtk::glib::clone!(@strong sender, @strong entry, @strong dialog => move |_| {
                let text = entry.text();
                let name = text.trim();
                
                if !name.is_empty() {
                    sender.input(AppMsg::CreateNewNote(name.to_string()));
                    dialog.close();
                }
            })
        );
        
        // Enter también crea la nota
        entry.connect_activate(
            gtk::glib::clone!(@strong sender, @strong dialog => move |entry| {
                let text = entry.text();
                let name = text.trim();
                
                if !name.is_empty() {
                    sender.input(AppMsg::CreateNewNote(name.to_string()));
                    dialog.close();
                }
            })
        );
        
        dialog.present();
        
        // Dar foco al entry
        gtk::glib::source::timeout_add_local(
            std::time::Duration::from_millis(50),
            move || {
                entry.grab_focus();
                gtk::glib::ControlFlow::Break
            }
        );
    }
}
