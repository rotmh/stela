use std::collections::HashMap;

use chrono::Utc;
use futures::TryStreamExt;
use serde::Deserialize;
use tokio::sync::broadcast;
use tracing::{debug, error, info, trace};
use url::Url;
use zbus::{Connection, MatchRule, MessageStream, fdo, zvariant};

#[derive(Debug, Deserialize, zvariant::Type)]
#[allow(dead_code)]
struct Notification<'a> {
    app_name: &'a str,
    replaces_id: u32,
    app_icon: &'a str,
    summary: &'a str,
    body: &'a str,
    actions: Vec<&'a str>,
    hints: HashMap<&'a str, zvariant::Value<'a>>,
    expire_timeout: i32,
}

impl<'a> From<Notification<'a>> for crate::Notification {
    fn from(value: Notification) -> Self {
        let app_icon = Url::parse(value.app_icon)
            .ok()
            .map(|_url| value.app_icon.to_owned());
        debug!(?app_icon, %value.app_icon);
        Self {
            app_name: value.app_name.to_owned(),
            summary: value.summary.to_owned(),
            body: value.body.to_owned(),
            app_icon,
            created_at: Utc::now().naive_utc(),
        }
    }
}

#[tracing::instrument]
pub async fn listen(
    tx: broadcast::Sender<crate::Notification>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> zbus::Result<()> {
    let mut stream = stream().await?;

    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                info!("Shutdown signal received, exiting task...");
                return Ok(());
            }
            msg = stream.try_next() => match msg? {
                Some(msg) => handle_message(msg, &tx),
                None => break,
            }
        }
    }

    Ok(())
}

fn handle_message(
    msg: zbus::Message,
    tx: &broadcast::Sender<crate::Notification>,
) {
    match msg.body().deserialize::<Notification>() {
        Ok(notification) => {
            trace!(?notification, "Received notification");

            if let Err(error) = tx.send(notification.into()) {
                error!(%error, "Failed to broadcast notification");
            }
        }
        Err(error) => {
            error!(%error, "Failed to deserialize notification");
        }
    }
}

async fn stream() -> zbus::Result<MessageStream> {
    let connection = Connection::session().await?;
    let proxy = fdo::MonitoringProxy::builder(&connection)
        .destination("org.freedesktop.DBus")?
        .path("/org/freedesktop/DBus")?
        .build()
        .await?;

    let rule = MatchRule::builder()
        .msg_type(zbus::message::Type::MethodCall)
        .interface("org.freedesktop.Notifications")?
        .member("Notify")?
        .path("/org/freedesktop/Notifications")?
        .build();
    proxy.become_monitor(&[rule], 0).await?;

    info!("Created a message stream for notifications");

    Ok(MessageStream::from(&connection))
}
