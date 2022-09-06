use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate};


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/window.ui")]
pub struct h2eckWindow {
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
    }
}

impl WidgetImpl for h2eckWindow {}
impl WindowImpl for h2eckWindow {}
impl ApplicationWindowImpl for h2eckWindow {}