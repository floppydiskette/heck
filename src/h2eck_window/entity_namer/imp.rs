use std::sync::{Arc, Mutex};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, ListBox};
use gtk::AccessibleRole::Label;
use gtk::gio::Menu;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/entity_namer.ui")]
pub struct EntityNamer {
    // buttons
    #[template_child]
    pub cancel_button: TemplateChild<Button>,
    #[template_child]
    pub ok_button: TemplateChild<Button>,

    // text entry
    #[template_child]
    pub name_entry: TemplateChild<gtk::Entry>,
}

#[glib::object_subclass]
impl ObjectSubclass for EntityNamer {
    const NAME: &'static str = "h2eckEntityNamer";
    type Type = super::EntityNamer;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for EntityNamer {
    fn constructed(&self, obj: &Self::Type) {
        // call "constructed" on parent
        self.parent_constructed(obj);
        self.setup(obj);
    }
}

impl EntityNamer {
    pub fn setup(&self, obj: &<EntityNamer as ObjectSubclass>::Type) {
    }
}

impl WidgetImpl for EntityNamer {}
impl WindowImpl for EntityNamer {}
impl ApplicationWindowImpl for EntityNamer {}