use std::ops::Deref;
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
use crate::worldmachine::{World, WorldMachine};


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

    pub treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
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
        let mut model = self.treestore.lock().unwrap();
        *model = Some(gtk::TreeStore::new(&[Type::STRING]));
        let model = model.as_ref().unwrap();
        let root = model.append(None);
        model.set(&root, &[(0, &Value::from("worldmachine"))]);
        self.scene_browser.set_model(Some(model));
        // setup a cell renderer
        let cell = gtk::CellRendererText::new();
        self.entity_column.pack_start(&cell, true);
        self.entity_column.add_attribute(&cell, "text", 0);
    }

    pub fn regen_model_from_world(&self, wm: &mut World) {
        let mut model = self.treestore.lock().unwrap();
        let model = model.as_ref().unwrap();
        model.clear();
        let root = model.append(None);
        model.set(&root, &[(0, &Value::from("worldmachine"))]);
        for entity in wm.clone().entities {
            let entity_node = model.append(Some(&root));
            model.set(&entity_node, &[(0, &Value::from(entity.get_name().as_str()))]);
            for component in entity.get_components() {
                let component_node = model.append(Some(&entity_node));
                model.set(&component_node, &[(0, &Value::from(component.get_name().as_str()))]);
            }
        }
    }
}

impl WidgetImpl for Editor {}
impl WindowImpl for Editor {}
impl BoxImpl for Editor {}