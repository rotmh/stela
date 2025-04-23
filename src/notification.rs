use std::collections::HashMap;

use futures::TryStreamExt;
use serde::Deserialize;
use tokio::sync::broadcast;
use tracing::{debug, error, info, trace};
use zbus::{
    Connection, MatchRule, MessageStream, fdo,
    message::Body,
    zvariant::{self, Value},
};

// Ref: <https://specifications.freedesktop.org/notification-spec/latest/basic-design.html#id-1.3.6>.
#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub hints: Hints,
    pub expire_timeout: i32,
}

impl TryFrom<Body> for Notification {
    type Error = zbus::Error;

    fn try_from(value: Body) -> Result<Self, Self::Error> {
        #[derive(Deserialize, zvariant::Type)]
        struct Tmp<'a> {
            app_name: &'a str,
            replaces_id: u32,
            app_icon: &'a str,
            summary: &'a str,
            body: &'a str,
            actions: Vec<&'a str>,
            hints: HashMap<&'a str, zvariant::Value<'a>>,
            expire_timeout: i32,
        }

        let mut tmp: Tmp = value.deserialize()?;

        let image_data = if let Some((_key, image_data)) =
            tmp.hints.remove_entry("image-data")
        {
            Some(image_data.try_into().map_err(zbus::Error::Failure)?)
        } else {
            None
        };

        let hints = Hints { image_data };

        Ok(Self {
            app_name: tmp.app_name.to_owned(),
            replaces_id: tmp.replaces_id,
            app_icon: tmp.app_icon.to_owned(),
            summary: tmp.summary.to_owned(),
            body: tmp.body.to_owned(),
            actions: tmp.actions.into_iter().map(ToOwned::to_owned).collect(),
            hints,
            expire_timeout: tmp.expire_timeout,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Hints {
    pub image_data: Option<ImageData>,
}

// Ref: <https://specifications.freedesktop.org/notification-spec/latest/icons-and-images.html#icons-and-images-formats>.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Width of image in pixels.
    pub width: i32,
    /// Height of image in pixels.
    pub height: i32,
    /// Distance in bytes between row starts.
    pub rowstride: i32,
    /// Whether the image has an alpha channel.
    pub has_alpha: bool,
    /// Must always be 8.
    pub bits_per_sample: i32,
    /// If has_alpha is TRUE, must be 4, otherwise 3.
    pub channels: i32,
    /// The image data, in RGB byte order.
    pub data: Vec<u8>,
}

impl ImageData {
    // Ref: <https://specifications.freedesktop.org/notification-spec/latest/icons-and-images.html#icons-and-images-formats>.
    const SIGNATURE: &'static str = "(iiibiiay)";

    // TODO: use this.
    #[allow(dead_code)]
    const KEYS: &[&'static str] = &["image-data", "image-path", "icon_data"];
}

impl<'a> TryFrom<Value<'a>> for ImageData {
    type Error = String;

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        let Value::Structure(structure) = value else {
            return Err("Expected `Value` to be `Structure`".to_owned());
        };
        if structure.signature() != Self::SIGNATURE {
            return Err(format!(
                "Expected signature to be `{}`",
                Self::SIGNATURE
            ));
        }

        let mut fields = structure.into_fields();
        if fields.len() != 7 {
            return Err("Expected `Structure` to have 7 fields".to_owned());
        }

        let Ok(data) = fields.remove(6).try_into() else {
            return Err("Failed to deserialize `data`".to_owned());
        };
        let Ok(channels) = fields.remove(5).try_into() else {
            return Err("Failed to deserialize `channels`".to_owned());
        };
        let Ok(bits_per_sample) = fields.remove(4).try_into() else {
            return Err("Failed to deserialize `bits_per_sample`".to_owned());
        };
        let Ok(has_alpha) = fields.remove(3).try_into() else {
            return Err("Failed to deserialize `has_alpha`".to_owned());
        };
        let Ok(rowstride) = fields.remove(2).try_into() else {
            return Err("Failed to deserialize `rowstride`".to_owned());
        };
        let Ok(height) = fields.remove(1).try_into() else {
            return Err("Failed to deserialize `height`".to_owned());
        };
        let Ok(width) = fields.remove(0).try_into() else {
            return Err("Failed to deserialize `width`".to_owned());
        };

        debug!(?width, ?height);

        Ok(Self {
            width,
            height,
            rowstride,
            has_alpha,
            bits_per_sample,
            channels,
            data,
        })
    }
}

#[tracing::instrument(skip_all)]
pub async fn listen(
    tx: broadcast::Sender<Notification>,
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

fn handle_message(msg: zbus::Message, tx: &broadcast::Sender<Notification>) {
    match msg.body().try_into() {
        Ok(notification) => {
            trace!(?notification, "Received notification");

            if let Err(error) = tx.send(notification) {
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
