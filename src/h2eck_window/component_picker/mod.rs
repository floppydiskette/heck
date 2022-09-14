mod imp;

use glib::Object;
use gtk::{gio, glib, prelude::*, Application};

glib::wrapper! {
    pub struct ComponentPicker(ObjectSubclass<imp::ComponentPicker>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl ComponentPicker {
    pub fn new() -> Self {
        Object::new(&[]).expect("failed to create component picker window")
    }
}