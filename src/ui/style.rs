use gtk4::{self as gtk, CssProvider, gdk::Display};

#[derive(Debug)]
pub enum Error {
    Display,
}

/// Load the styles for the GTK GUI.
pub fn load() -> Result<(), Error> {
    let display = Display::default().ok_or(Error::Display)?;

    let provider = CssProvider::new();
    provider.load_from_path("resources/main.css");

    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    Ok(())
}
