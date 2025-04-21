use std::collections::HashMap;

use futures::TryStreamExt;
use serde::Deserialize;
use zbus::{Connection, MatchRule, MessageStream, fdo, zvariant};

#[derive(Debug, Deserialize, zvariant::Type)]
pub struct Notification<'a> {
    app_name: &'a str,
    replaces_id: u32,
    app_icon: &'a str,
    summary: &'a str,
    body: &'a str,
    actions: Vec<&'a str>,
    hints: HashMap<&'a str, zvariant::Value<'a>>,
    expire_timeout: i32,
}

pub async fn listen(tx: async_channel::Sender<String>) -> zbus::Result<()> {
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

    let mut stream = MessageStream::from(&connection);

    while let Some(msg) = stream.try_next().await? {
        match msg.body().deserialize::<Notification>() {
            Ok(notification) => {
                tx.send(notification.summary.to_owned()).await.unwrap()
            }
            Err(error) => {}
        }
    }

    Ok(())
}
