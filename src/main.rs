use gtk4::{Application, Window, glib, prelude::*};
use tokio::sync::broadcast;
use tracing::{Level, error};

use stela::{Notification, config::Config, notification, ui};

const APP_ID: &str = "dev.rotmh.stela";

#[tokio::main]
async fn main() -> std::process::ExitCode {
    dotenvy::dotenv().unwrap();
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();

    let cfg = Config::init();

    let (tx, _rx) = broadcast::channel(10);

    let app = Application::builder().application_id(APP_ID).build();

    let handle = tokio::spawn(notification::listen(tx.clone()));

    app.connect_startup(move |app| startup(app.to_owned(), tx.subscribe()));
    app.connect_activate(activate);

    let exit_code = app.run();

    if let Err(error) = handle.await {
        error!(%error, "Failed to join the notifications listener");
        std::process::ExitCode::FAILURE
    } else {
        match exit_code {
            glib::ExitCode::SUCCESS => std::process::ExitCode::SUCCESS,
            _ => std::process::ExitCode::FAILURE,
        }
    }
}

fn startup(app: Application, mut rx: broadcast::Receiver<Notification>) {
    glib::spawn_future_local(async move {
        let mut popup_manager = ui::popup::Manager::new(app);

        while let Ok(notification) = rx.recv().await {
            popup_manager.push(notification);
        }
    });
}

fn activate(app: &Application) {
    if let Err(error) = ui::style::load() {
        error!(?error, "Failed to load styles");
    }

    // We need to create a window here for some reason.
    let w = Window::new();
    w.set_application(Some(app));
}
