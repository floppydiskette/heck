mod imp;

use glib::Object;
use gtk::{gio, glib, prelude::*, Application};

glib::wrapper! {
    pub struct AboutWindow(ObjectSubclass<imp::AboutWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl AboutWindow {
    pub fn new() -> Self {
        Object::new(&[]).expect("failed to create about window")
    }
}