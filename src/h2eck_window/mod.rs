mod imp;
pub mod about_window;
pub mod editor;

use glib::Object;
use gtk::{gio, glib, prelude::*, Application};
use gtk::subclass::prelude::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct h2eckWindow(ObjectSubclass<imp::h2eckWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl h2eckWindow {
    pub fn new(app: &Application) -> Self {
        Object::new(&[("application", app)]).expect("failed to create h2eckWindow")
    }

    pub fn render(&self) {
        (*self.imp().renderer.lock().unwrap()).render();
    }
}