mod app;
mod core;

use relm4::{
    RelmApp, adw,
    gtk::{self, gio, glib, prelude::*},
};

use crate::app::{APP_ID, MainApp, ThemePreference};

fn main() -> anyhow::Result<()> {
    // Ensure libadwaita registers its resources for consistent styling.
    adw::init()?;
    glib::set_application_name("NotNative");

    // Cargar estilos CSS personalizados
    let css_provider = gtk::CssProvider::new();
    if let Ok(css_data) = std::fs::read_to_string("assets/style.css") {
        css_provider.load_from_data(&css_data);
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("No se pudo obtener el display"),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    let relm_app = RelmApp::from_app(app);

    relm_app.run::<MainApp>(ThemePreference::FollowSystem);

    Ok(())
}
