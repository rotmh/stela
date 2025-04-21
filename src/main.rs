mod notification;
mod ui;

use std::sync::Arc;

use gtk4::{
    self as gtk, Application, ApplicationWindow, Window, glib, prelude::*,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use tracing::error;

const APP_ID: &str = "dev.rotmh.stela";

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let (tx, rx) = async_channel::bounded(1);

    let handle = tokio::spawn(async move { notification::listen(tx).await });

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(activate);

    app.connect_startup(move |app| {
        let rx = rx.clone();
        let app = app.clone();

        glib::spawn_future_local(async move {
            let mut popup_manager = ui::popup::Manager::new(app);

            while let Ok(notification) = rx.recv().await {
                popup_manager.push(notification);
            }
        });
    });

    let exit_code = app.run();

    if let Err(error) = handle.await {
        error!(error = %error, "Failed to join the notifications listener");
        std::process::ExitCode::FAILURE
    } else {
        match exit_code {
            glib::ExitCode::SUCCESS => std::process::ExitCode::SUCCESS,
            _ => std::process::ExitCode::FAILURE,
        }
    }
}

fn activate(app: &Application) {
    ui::style::load();

    // We need to create a window here for some reason.
    let w = Window::new();
    w.set_application(Some(app));
}
