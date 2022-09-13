mod imp;

use glib::Object;
use gtk::{gio, glib, prelude::*, Application};

glib::wrapper! {
    pub struct EntityPicker(ObjectSubclass<imp::EntityPicker>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl EntityPicker {
    pub fn new() -> Self {
        Object::new(&[]).expect("failed to create about window")
    }
}