use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, gio, show_about_dialog};
use gio::prelude::*;
use gtk::gdk::Display;
use gtk::glib::clone;
use crate::h2eck_window::{about_window, h2eckWindow};

pub mod h2eck_window;
pub mod renderer;

const APP_ID: &str = "com.realmicrosoft.h2eck";

fn main() {
    // register resources
    gio::resources_register_include!("ct.gresource")
        .expect("failed to register ui resources");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|app| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_bytes!("../assets/styles.css"));

        // Add the provider to the default screen
        gtk::StyleContext::add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        build_accelerators(app);
    });

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let window = h2eckWindow::new(app);

    build_system_menu(app);
    build_appwide_actions(app, &window);

    window.present();
    window.set_show_menubar(true);
}

fn build_system_menu(app: &Application) {
    let menu_bar = gio::Menu::new();
    let app_menu = gio::Menu::new();
    let file_menu = gio::Menu::new();
    let help_menu = gio::Menu::new();

    app_menu.append(Some("About"), Some("app.about"));
    app_menu.append(Some("Quit"), Some("app.quit"));

    file_menu.append(Some("Quit"), Some("app.quit"));

    help_menu.append(Some("About"), Some("app.about"));

    menu_bar.append_submenu(Some("File"), &file_menu);
    menu_bar.append_submenu(Some("Help"), &help_menu);

    app.set_menubar(Some(&menu_bar));
}

fn build_appwide_actions(app: &Application, window: &h2eckWindow) {
    let about_action = gio::SimpleAction::new("about", None);
    // show the about dialog
    about_action.connect_activate(clone!(@strong window => move |_, _| {
        let about_window = about_window::AboutWindow::new();
        about_window.show();
    }));

    let quit_action = gio::SimpleAction::new("quit", None);
    quit_action.connect_activate(clone!(@strong app => move |_, _| {
        app.quit();
    }));

    app.add_action(&about_action);
    app.add_action(&quit_action);
}

fn build_accelerators(app: &Application) {
    app.set_accels_for_action("app.about", &["<Primary>a"]);
    app.set_accels_for_action("app.quit", &["<Primary>q"]);
}