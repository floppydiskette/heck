use std::sync::{Arc, Mutex};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar};
use gtk::gio::Menu;
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::worldmachine::WorldMachine;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/entity_picker.ui")]
pub struct EntityPicker {
    pub worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for EntityPicker {
    const NAME: &'static str = "h2eckEntityPicker";
    type Type = super::EntityPicker;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for EntityPicker {
    fn constructed(&self, obj: &Self::Type) {
        // call "constructed" on parent
        self.parent_constructed(obj);
    }
}

impl WidgetImpl for EntityPicker {}
impl WindowImpl for EntityPicker {}
impl ApplicationWindowImpl for EntityPicker {}