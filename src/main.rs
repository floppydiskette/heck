use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

const APP_ID: &str = "com.realmicrosoft.h2eck";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .width_request(1280)
        .height_request(720)
        .title("Heck")
        .build();

    window.present();
}