use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Local};
use gtk4::{
    self as gtk, Align, Application, ApplicationWindow, Frame, GestureClick,
    Image, Label, Orientation,
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::Bytes,
    prelude::*,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

/// The gap between the notification popups.
const GAP: i32 = 10;
/// The margins between the popups stack and the screen's edges.
const MARGIN: i32 = 10;
/// The popup's width.
const WIDTH: i32 = 350;

const IMAGE_SIZE: i32 = 48;

pub struct Manager {
    application: Application,
    windows: Windows,
}

impl Manager {
    pub fn new(application: Application) -> Self {
        Self { application, windows: Windows::new() }
    }

    pub fn push(&mut self, notification: crate::notification::Notification) {
        let window = self.create_popup_window(notification);
        self.windows.push(window.clone());
        window.present();
        self.windows.adjust();
    }

    fn create_popup_window(
        &self,
        notification: crate::notification::Notification,
    ) -> ApplicationWindow {
        // *--------------------------------------------*
        // | ICON | APP_NAME |      <space>      | TIME |
        // |--------------------------------------------*
        // |            SUMMARY           |   <space>   |
        // *--------------------------------------------*
        // |                               |            |
        // |              BODY             |   IMAGE?   |
        // |                               |            |
        // *--------------------------------------------*

        // Contains the app name, icon, and the time.
        let header = Self::header();
        if !notification.app_icon.is_empty() {
            header.append(&Self::icon(&notification.app_icon));
        }
        header.append(&Self::app_name(&notification.app_name));
        header.append(&Self::time(Local::now()));

        // Contains the body and image.
        let content =
            gtk::Box::builder().orientation(Orientation::Horizontal).build();
        if !notification.body.is_empty() {
            content.append(&Self::body(&notification.body));
        }
        if let Some(image_data) = notification.hints.image_data {
            content.append(&Self::image(image_data));
        }

        // Contains the every thing in the popup.
        let container =
            gtk::Box::builder().orientation(Orientation::Vertical).build();

        container.append(&header);

        if !notification.summary.is_empty() {
            container.append(&Self::summary(&notification.summary));
        }
        // Check whether `content` is empty.
        if content.first_child().is_some() {
            container.append(&content);
        }

        let window = ApplicationWindow::builder()
            .application(&self.application)
            .child(&container)
            .default_width(WIDTH)
            .resizable(false)
            .decorated(true)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Right, MARGIN);
        window.set_margin(Edge::Top, MARGIN);

        let click = GestureClick::new();
        let window_clone = window.clone();
        let windows = self.windows.clone();

        click.connect_pressed(move |_click, _button, _x, _y| {
            window_clone.close();
            windows.remove(&window_clone);
            windows.adjust();
        });
        window.add_controller(click);

        window
    }

    fn image(image_data: crate::notification::ImageData) -> Frame {
        let pixbuf = Pixbuf::from_bytes(
            &Bytes::from_owned(image_data.data),
            Colorspace::Rgb,
            image_data.has_alpha,
            image_data.bits_per_sample,
            image_data.width,
            image_data.height,
            image_data.rowstride,
        );
        let image = Image::from_pixbuf(Some(&pixbuf));
        image.set_halign(Align::End);
        image.add_css_class("image");
        dbg!(image.icon_size(), image.pixel_size(), pixbuf.height());

        image.set_size_request(IMAGE_SIZE, IMAGE_SIZE);

        Frame::builder().css_classes(["image-frame"]).child(&image).build()
    }

    fn header() -> gtk::Box {
        let header = gtk::Box::builder()
            .css_classes(["header"])
            .halign(Align::Fill)
            .orientation(Orientation::Horizontal)
            .build();
        header.set_spacing(5);
        header
    }

    fn time(dt: DateTime<Local>) -> Label {
        let time =
            Label::builder().css_classes(["time"]).halign(Align::End).build();
        time.set_text(&dt.format("%H:%M").to_string());
        time
    }

    fn app_name(text: &str) -> Label {
        let app_name = Label::builder()
            .css_classes(["app-name"])
            .hexpand(true)
            .halign(Align::Start)
            .build();
        app_name.set_text(text);
        app_name
    }

    fn icon(uri: &str) -> Image {
        Image::builder()
            .css_classes(["app_icon"])
            .halign(Align::Start)
            .file(uri)
            .build()
    }

    fn body(text: &str) -> Label {
        let body = Label::builder()
            .css_classes(["body"])
            .max_width_chars(50)
            .wrap(true)
            .halign(Align::Start)
            .hexpand(true)
            .build();
        body.set_text(text);
        body
    }

    fn summary(text: &str) -> Label {
        let summary = Label::builder()
            .css_classes(["summary"])
            .wrap(true)
            .halign(Align::Start)
            .build();
        summary.set_text(text);
        summary
    }
}

#[derive(Clone)]
struct Windows(Rc<RefCell<Vec<ApplicationWindow>>>);

impl Windows {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(vec![])))
    }

    fn push(&self, window: ApplicationWindow) {
        self.0.borrow_mut().push(window);
    }

    fn remove(&self, window: &ApplicationWindow) {
        self.0.borrow_mut().retain(|w| !w.eq(window));
    }

    fn adjust(&self) {
        let mut offset = MARGIN;
        for window in self.0.borrow_mut().iter_mut().rev() {
            window.set_margin(Edge::Top, offset);

            // FIXME: I'm not sure if this is the optimal solution for finding
            // the full height of a window. So currently this is temporary.
            let (_minimum, natural, _minimum_baseline, _natural_baseline) =
                window.measure(Orientation::Vertical, WIDTH);
            let height = natural;

            offset += height + GAP;
        }
    }
}
