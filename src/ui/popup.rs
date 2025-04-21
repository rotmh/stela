use std::{cell::RefCell, rc::Rc};

use gtk4::{
    self as gtk, Align, Application, ApplicationWindow, GestureClick, Label,
    Orientation, prelude::*,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

/// The gap between the notification popups.
const GAP: i32 = 10;
/// The margins between the popups stack and the screen's edges.
const MARGIN: i32 = 20;
/// The popup's width.
const WIDTH: i32 = 400;

pub struct Manager {
    application: Application,
    windows: Windows,
}

impl Manager {
    pub fn new(application: Application) -> Self {
        Self { application, windows: Windows::new() }
    }

    pub fn push(&mut self, notification: crate::Notification) {
        let window = self.create_popup_window(notification);
        self.windows.push(window.clone());
        window.present();
        self.windows.adjust();
    }

    fn create_popup_window(
        &self,
        notification: crate::Notification,
    ) -> ApplicationWindow {
        // *--------------------------------------------*
        // | ICON | APP_NAME |      <space>      | TIME |
        // |--------------------------------------------*
        // |            SUMMARY           |   <space>   |
        // *--------------------------------------------*
        // |                                  |         |
        // |                BODY              | <space> |
        // |                                  |         |
        // *--------------------------------------------*

        let app_name = Label::builder()
            .css_classes(["app-name"])
            .hexpand(true)
            .halign(Align::Start)
            .build();
        app_name.set_text(&notification.app_name);

        let time = Label::builder().halign(Align::End).build();
        time.set_text(&notification.created_at.format("%H:%M").to_string());

        let header = gtk::Box::builder()
            .css_classes(["header"])
            .halign(Align::Fill)
            .orientation(Orientation::Horizontal)
            .build();
        header.set_spacing(5);
        header.append(&app_name);
        header.append(&time);

        let summary = Label::builder()
            .css_classes(["summary"])
            .halign(Align::Start)
            .build();
        summary.set_text(&notification.summary);

        let container =
            gtk::Box::builder().orientation(Orientation::Vertical).build();

        container.append(&header);

        if !notification.summary.is_empty() {
            container.append(&Self::summary(&notification.summary));
        }
        if !notification.body.is_empty() {
            container.append(&Self::body(&notification.body));
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

    fn body(text: &str) -> Label {
        let body =
            Label::builder().css_classes(["body"]).halign(Align::Start).build();
        body.set_text(text);
        body
    }

    fn summary(text: &str) -> Label {
        let summary = Label::builder()
            .css_classes(["summary"])
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
