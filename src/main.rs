use anyhow::{Context, anyhow};
use gtk4::{Application, Window, glib, prelude::*};
use tokio::sync::broadcast;
use tracing::{Level, error, info};

use stela::{
    Notification, config::Config, notification, persistence::Persistence, ui,
};

const APP_ID: &str = "dev.rotmh.stela";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().context("Failed to load .env file")?;
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();

    let (tx, _rx) = broadcast::channel(10);
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let ctrl_c = tokio::spawn(ctrl_c_handler(shutdown_tx));

    // Listen to notifications and broadcast them.
    let listen =
        tokio::spawn(notification::listen(tx.clone(), shutdown_rx.clone()));

    // Persist the broadcasted notifications.
    let cfg = Config::init();
    let persistence = Persistence::new(&cfg).await?;
    let persistence =
        tokio::spawn(persistence.persist(tx.subscribe(), shutdown_rx.clone()));

    // Show a popup for each broadcasted notification.
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| startup(app.to_owned(), tx.subscribe()));
    app.connect_activate(activate);
    app.connect_activate(move |app| {
        shutdown_handler(app.to_owned(), shutdown_rx.clone())
    });
    let exit_code = app.run();

    join_task("persistence", persistence).await?;
    join_task("listen", listen).await?;
    join_task("ctrl_c_handler", ctrl_c).await?;

    match exit_code {
        glib::ExitCode::SUCCESS => Ok(()),
        _ => Err(anyhow!("Got a FAILURE exit code from the application")),
    }
}

async fn join_task<T, E>(
    name: &str,
    task: tokio::task::JoinHandle<Result<T, E>>,
) -> anyhow::Result<()>
where
    E: std::fmt::Display,
{
    match task.await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => {
            error!(%error, "Task `{name}` failed");
            Err(anyhow!("Task `{name}` failed: {error}"))
        }
        Err(error) => {
            error!(%error, "Join error in task `{name}`");
            Err(anyhow!("Join error in task `{name}`: {error}"))
        }
    }
}

async fn ctrl_c_handler(
    tx: tokio::sync::watch::Sender<bool>,
) -> anyhow::Result<()> {
    tokio::signal::ctrl_c()
        .await
        .context("Failed to install Ctrl-C handler")?;
    tx.send(true)?;
    Ok(())
}

fn startup(app: Application, mut rx: broadcast::Receiver<Notification>) {
    glib::spawn_future_local(async move {
        let mut popup_manager = ui::popup::Manager::new(app);

        while let Ok(notification) = rx.recv().await {
            popup_manager.push(notification);
        }
    });
}

#[tracing::instrument]
fn shutdown_handler(
    app: Application,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    glib::spawn_future_local(async move {
        if let Err(error) = shutdown.changed().await {
            error!(%error, "Failed to await the shutdown ");
        }
        info!("Shutdown signal received, quiting application...");
        app.quit();
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
