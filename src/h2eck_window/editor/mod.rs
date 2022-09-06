mod imp;

use glib::Object;
use gtk::{gio, glib, prelude::*, Application};
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct Editor(ObjectSubclass<imp::Editor>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl Editor {
    pub fn new() -> Self {
        Object::new(&[]).expect("failed to create editor box")
    }
}