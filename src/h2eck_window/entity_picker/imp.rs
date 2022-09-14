use std::sync::{Arc, Mutex};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, ListBox};
use gtk::AccessibleRole::Label;
use gtk::gio::Menu;
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::h2eck_window::entity_namer::EntityNamer;
use crate::h2eck_window::entity_picker;
use crate::renderer::H2eckRenderer;
use crate::worldmachine::WorldMachine;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/entity_picker.ui")]
pub struct EntityPicker {
    pub worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
    pub renderer: Arc<Mutex<Arc<Mutex<H2eckRenderer>>>>,

    // buttons
    #[template_child]
    pub cancel_button: TemplateChild<Button>,
    #[template_child]
    pub new_button: TemplateChild<Button>,
    #[template_child]
    pub ok_button: TemplateChild<Button>,

    // list of entities
    #[template_child]
    pub list_box: TemplateChild<ListBox>,
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
        self.setup(obj);
    }
}

pub fn add_entity(name: &str, worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>, renderer: Arc<Mutex<Arc<Mutex<H2eckRenderer>>>>) {
    let worldmachine = worldmachine.lock().unwrap();
    let worldmachine = worldmachine.as_ref().unwrap();
    let mut worldmachine = worldmachine.lock().unwrap();
    let renderer = renderer.lock().unwrap();
    let renderer = renderer.as_ref();
    let renderer = renderer.lock().unwrap();

    worldmachine.load_entity_def(name, renderer.camera.clone());
}

pub fn create_entity(name: &str, worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>) {
    let worldmachine = worldmachine.lock().unwrap();
    let worldmachine = worldmachine.as_ref().unwrap();
    let mut worldmachine = worldmachine.lock().unwrap();

    worldmachine.add_blank_entity(name);
}

impl EntityPicker {
    pub fn setup(&self, obj: &<EntityPicker as ObjectSubclass>::Type) {
        // cancel button
        self.cancel_button.connect_clicked(clone!(@weak obj => move |_| {
            obj.close();
        }));

        // ok button
        #[derive(Clone)]
        struct EntityPickerOkButtonData {
            list_box: ListBox,
            obj: entity_picker::EntityPicker,
            worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
            renderer: Arc<Mutex<Arc<Mutex<H2eckRenderer>>>>,
        }
        let data = EntityPickerOkButtonData {
            list_box: self.list_box.clone(),
            obj: obj.clone(),
            worldmachine: self.worldmachine.clone(),
            renderer: self.renderer.clone(),
        };
        self.ok_button.connect_clicked(move |_| {
            let selected_row = data.list_box.selected_row();
            if let Some(selected_row) = selected_row {
                let selected_row = selected_row.clone();
                let selected_row = selected_row.downcast::<gtk::ListBoxRow>().unwrap();
                let selected_row = selected_row.child().unwrap();
                let selected_row = selected_row.downcast::<gtk::Label>().unwrap();
                let selected_row = selected_row.label();
                debug!("selected row: {}", selected_row);
                add_entity(&selected_row, data.worldmachine.clone(), data.renderer.clone());
            }
            data.obj.close();
        });

        // new button
        let data = EntityPickerOkButtonData {
            list_box: self.list_box.clone(),
            obj: obj.clone(),
            worldmachine: self.worldmachine.clone(),
            renderer: self.renderer.clone(),
        };
        self.new_button.connect_clicked(move |_| {
            let entity_namer = EntityNamer::new();
            entity_namer.imp().cancel_button.connect_clicked(clone!(@weak entity_namer => move |_| {
                entity_namer.close();
            }));
            struct EntityNamerOkData {
                pub prev_data: EntityPickerOkButtonData,
                pub entity_namer: EntityNamer,
            }
            let data = EntityNamerOkData {
                prev_data: data.clone(),
                entity_namer: entity_namer.clone(),
            };
            entity_namer.imp().ok_button.connect_clicked(move |_| {
                let entity_name = data.entity_namer.imp().name_entry.text();
                create_entity(&entity_name, data.prev_data.worldmachine.clone());
                data.prev_data.obj.close();
                data.entity_namer.close();
                data.prev_data.obj.close();
            });
            entity_namer.show();
        });
    }

    pub fn populate_listbox(&self) {
        let worldmachine = self.worldmachine.lock().unwrap();
        let worldmachine = worldmachine.as_ref().unwrap();
        let worldmachine = worldmachine.lock().unwrap();

        let entities = worldmachine.list_possible_entities();

        for entity in entities {
            let row = gtk::ListBoxRow::new();
            let label = gtk::Label::new(Some(entity.as_str()));
            row.set_child(Some(&label));
            self.list_box.append(&row);
        }
    }
}

impl WidgetImpl for EntityPicker {}
impl WindowImpl for EntityPicker {}
impl ApplicationWindowImpl for EntityPicker {}