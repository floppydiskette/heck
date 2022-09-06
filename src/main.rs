use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, gio};
use crate::h2eck_window::h2eckWindow;

pub mod h2eck_window;

const APP_ID: &str = "com.realmicrosoft.h2eck";

fn main() {
    // register resources
    gio::resources_register_include!("ct.gresource")
        .expect("failed to register ui resources");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let window = h2eckWindow::new(app);
    window.present();
}