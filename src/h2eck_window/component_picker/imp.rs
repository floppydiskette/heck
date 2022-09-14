use std::sync::{Arc, Mutex};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, ListBox};
use gtk::AccessibleRole::Label;
use gtk::gio::Menu;
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::h2eck_window::{component_picker, entity_picker};
use crate::renderer::H2eckRenderer;
use crate::worldmachine::WorldMachine;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/component_picker.ui")]
pub struct ComponentPicker {
    pub worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,

    // buttons
    #[template_child]
    pub cancel_button: TemplateChild<Button>,
    #[template_child]
    pub ok_button: TemplateChild<Button>,

    // list of entities
    #[template_child]
    pub list_box: TemplateChild<ListBox>,

    // current selected entity
    pub selected_entity_id: Arc<Mutex<u64>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ComponentPicker {
    const NAME: &'static str = "h2eckComponentPicker";
    type Type = super::ComponentPicker;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ComponentPicker {
    fn constructed(&self, obj: &Self::Type) {
        // call "constructed" on parent
        self.parent_constructed(obj);
        self.setup(obj);
    }
}

pub fn add_component(name: &str, id: u64, worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>) {
    let worldmachine = worldmachine.lock().unwrap();
    let worldmachine = worldmachine.as_ref().unwrap();
    let mut worldmachine = worldmachine.lock().unwrap();

    let component = worldmachine.new_component_from_name(name);
    if component.is_none() {
        error!("failed to create component: {}", name);
        return;
    }
    let component = component.unwrap();
    worldmachine.give_component_to_entity(id, component);
}

impl ComponentPicker {
    pub fn setup(&self, obj: &<ComponentPicker as ObjectSubclass>::Type) {
        // cancel button
        self.cancel_button.connect_clicked(clone!(@weak obj => move |_| {
            obj.close();
        }));

        // ok button
        struct ComponentPickerOkButtonData {
            list_box: ListBox,
            obj: component_picker::ComponentPicker,
            worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
            entity_id: Arc<Mutex<u64>>,
        }
        let data = ComponentPickerOkButtonData {
            list_box: self.list_box.clone(),
            obj: obj.clone(),
            worldmachine: self.worldmachine.clone(),
            entity_id: self.selected_entity_id.clone(),
        };
        self.ok_button.connect_clicked(move |_| {
            let selected_row = data.list_box.selected_row();
            if let Some(selected_row) = selected_row {
                let selected_row = selected_row.clone();
                let selected_row = selected_row.downcast::<gtk::ListBoxRow>().unwrap();
                let selected_row = selected_row.child().unwrap();
                let selected_row = selected_row.downcast::<gtk::Label>().unwrap();
                let selected_row = selected_row.label();
                let selected_entity_id = *data.entity_id.lock().unwrap();
                debug!("selected row: {}", selected_row);
                debug!("selected entity: {}", selected_entity_id);
                add_component(&selected_row, selected_entity_id, data.worldmachine.clone());
            }
            data.obj.close();
        });
    }

    pub fn populate_listbox(&self) {
        let worldmachine = self.worldmachine.lock().unwrap();
        let worldmachine = worldmachine.as_ref().unwrap();
        let worldmachine = worldmachine.lock().unwrap();

        let components = worldmachine.list_all_component_types();

        for component in components {
            let row = gtk::ListBoxRow::new();
            let label = gtk::Label::new(Some(component.as_str()));
            row.set_child(Some(&label));
            self.list_box.append(&row);
        }
    }
}

impl WidgetImpl for ComponentPicker {}
impl WindowImpl for ComponentPicker {}
impl ApplicationWindowImpl for ComponentPicker {}