use std::sync::{Arc, Mutex};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, Inhibit, GLArea};
use gtk::gdk::ffi::GdkGLContext;
use gtk::gio::Menu;
use gtk::glib::Value;
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::renderer::H2eckRenderer;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/editor.ui")]
pub struct Editor {
    pub renderer: Arc<Mutex<H2eckRenderer>>,
    #[template_child]
    pub main_view: TemplateChild<GLArea>,
}

#[glib::object_subclass]
impl ObjectSubclass for Editor {
    const NAME: &'static str = "Editor";
    type Type = super::Editor;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Editor {
    fn constructed(&self, obj: &Self::Type) {
        // call "constructed" on parent
        self.parent_constructed(obj);
        self.setup(obj);
    }
}

impl Editor {
    pub fn setup(&self, obj: &<Editor as ObjectSubclass>::Type) {
    }
}

impl WidgetImpl for Editor {}
impl WindowImpl for Editor {}
impl BoxImpl for Editor {}