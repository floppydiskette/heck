use std::sync::{Arc, Mutex};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, Inhibit, GLArea};
use gtk::gdk::ffi::GdkGLContext;
use gtk::gio::Menu;
use gtk::glib::{Type, Value};
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::renderer::H2eckRenderer;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/editor.ui")]
pub struct Editor {
    pub renderer: Arc<Mutex<H2eckRenderer>>,
    #[template_child]
    pub main_view: TemplateChild<GLArea>,
    #[template_child]
    pub scene_browser: TemplateChild<gtk::TreeView>,
    #[template_child]
    pub entity_column: TemplateChild<gtk::TreeViewColumn>,
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
        // create a treemodel
        let model = gtk::TreeStore::new(&[Type::STRING]);
        let root = model.append(None);
        model.set(&root, &[(0, &Value::from("root"))]);
        let child = model.append(Some(&root));
        model.set(&child, &[(0, &Value::from("child"))]);
        self.scene_browser.set_model(Some(&model));
        // setup a cell renderer
        let cell = gtk::CellRendererText::new();
        self.entity_column.pack_start(&cell, true);
        self.entity_column.add_attribute(&cell, "text", 0);

    }
}

impl WidgetImpl for Editor {}
impl WindowImpl for Editor {}
impl BoxImpl for Editor {}