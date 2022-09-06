use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar};
use gtk::gio::Menu;
use crate::gio::glib::clone;
use crate::gio::SimpleAction;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/about.ui")]
pub struct AboutWindow {
}

#[glib::object_subclass]
impl ObjectSubclass for AboutWindow {
    const NAME: &'static str = "h2eckAbout";
    type Type = super::AboutWindow;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for AboutWindow {
    fn constructed(&self, obj: &Self::Type) {
        // call "constructed" on parent
        self.parent_constructed(obj);
    }
}

impl WidgetImpl for AboutWindow {}
impl WindowImpl for AboutWindow {}
impl ApplicationWindowImpl for AboutWindow {}