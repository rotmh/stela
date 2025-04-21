use gtk4::{Application, Window, glib, prelude::*};
use tracing::{Level, error};

use stela::{notification, ui};

const APP_ID: &str = "dev.rotmh.stela";

#[tokio::main]
async fn main() -> std::process::ExitCode {
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();

    let (tx, _rx) = tokio::sync::broadcast::channel(10);

    let app = Application::builder().application_id(APP_ID).build();

    let tx_listener = tx.clone();
    let handle =
        tokio::spawn(async move { notification::listen(tx_listener).await });

    app.connect_activate(activate);

    app.connect_startup(move |app| {
        let app = app.clone();
        let mut rx = tx.subscribe();

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
    if let Err(error) = ui::style::load() {
        error!(error = ?error, "Failed to load styles");
    }

    // We need to create a window here for some reason.
    let w = Window::new();
    w.set_application(Some(app));
}
