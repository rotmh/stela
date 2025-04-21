use gtk4::{self as gtk, Application, ApplicationWindow, Label, prelude::*};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

/// The gap between the notification popups.
const GAP: i32 = 10;
/// The margins between the popups stack and the screen's edges.
const MARGIN: i32 = 20;
/// The popup's width.
const WIDTH: i32 = 400;

pub struct Manager {
    application: Application,
    windows: Vec<ApplicationWindow>,
}

impl Manager {
    pub fn new(application: Application) -> Self {
        Self { application, windows: vec![] }
    }

    pub fn push(&mut self, notification: String) {
        let window = self.create_popup_window(notification);
        self.adjust_all(window.height());
        window.present();
        self.windows.push(window);
    }

    fn create_popup_window(&self, notification: String) -> ApplicationWindow {
        let label = Label::builder()
            .label(&notification)
            .use_markup(true)
            .halign(gtk::Align::Start)
            .wrap(true)
            .build();
        label.set_text(&notification);

        let window = ApplicationWindow::builder()
            .application(&self.application)
            .child(&label)
            .default_width(WIDTH)
            .resizable(false)
            .margin_top(MARGIN)
            .decorated(true)
            .margin_end(MARGIN)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Top, true);

        window
    }

    fn adjust_all(&mut self, new_window_height: i32) {
        let mut offset = MARGIN + new_window_height;
        for window in self.windows.iter_mut().rev() {
            offset += GAP;
            window.set_margin(Edge::Top, offset);
            offset += window.height();
        }
    }
}
