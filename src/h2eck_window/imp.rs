use std::ffi::CString;
use std::sync::{Arc, Mutex};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, GLArea, Inhibit};
use gtk::ffi::*;
use gtk::gio::Menu;
use gtk::glib::translate::ToGlibPtr;
use gtk::glib::Type;
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::h2eck_window::editor::Editor;
use crate::renderer::H2eckRenderer;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/window.ui")]
pub struct h2eckWindow {
    pub renderer: Arc<Mutex<H2eckRenderer>>,
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
}

#[glib::object_subclass]
impl ObjectSubclass for h2eckWindow {
    const NAME: &'static str = "h2eckWindow";
    type Type = super::h2eckWindow;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for h2eckWindow {
    fn constructed(&self, obj: &Self::Type) {
        // call "constructed" on parent
        self.parent_constructed(obj);
        self.setup(obj);
    }
}

impl h2eckWindow {
    pub fn setup(&self, obj: &<h2eckWindow as ObjectSubclass>::Type) {
        let editor_obj = Editor::new();

        let renderer = obj.clone().imp().renderer.clone();

        // connect render signal
        editor_obj.imp().main_view.connect_render(move |_, ctx| {
            {
                let mut renderer_inner = renderer.lock().unwrap();
                println!("rendering");
                renderer_inner.render();
                // force unlock
                drop(renderer_inner);
            }
            Inhibit(false)
        });

        editor_obj.show();

        self.stack.add_child(&editor_obj);
        self.stack.set_visible_child(&editor_obj);
    }
}

impl WidgetImpl for h2eckWindow {}
impl WindowImpl for h2eckWindow {}
impl ApplicationWindowImpl for h2eckWindow {}