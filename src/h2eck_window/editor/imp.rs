use std::any::Any;
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use gfx_maths::{Quaternion, Vec3};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, Inhibit, GLArea, gdk, pango, MessageDialog, DialogFlags, MessageType, ButtonsType, ResponseType, TreePath};
use gtk::gdk::ffi::GdkGLContext;
use gtk::gio::Menu;
use gtk::glib::{Type, Value};
use crate::gio;
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::h2eck_window::component_picker::ComponentPicker;
use crate::h2eck_window::entity_namer::EntityNamer;
use crate::h2eck_window::entity_picker::EntityPicker;
use crate::renderer::H2eckRenderer;
use crate::worldmachine::{World, WorldMachine};
use crate::worldmachine::ecs::{Component, COMPONENT_TYPES, ParameterValue};


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/editor.ui")]
pub struct Editor {
    pub renderer: Arc<Mutex<Arc<Mutex<H2eckRenderer>>>>,
    #[template_child]
    pub main_view: TemplateChild<GLArea>,
    #[template_child]
    pub scene_browser: TemplateChild<gtk::TreeView>,
    #[template_child]
    pub entity_column: TemplateChild<gtk::TreeViewColumn>,
    #[template_child]
    pub entity_id_column: TemplateChild<gtk::TreeViewColumn>,
    #[template_child]
    pub inspector_tree: TemplateChild<gtk::TreeView>,
    #[template_child]
    pub parameter_column: TemplateChild<gtk::TreeViewColumn>,
    #[template_child]
    pub value_column: TemplateChild<gtk::TreeViewColumn>,
    #[template_child]
    pub parameter_name_renderer: TemplateChild<gtk::CellRendererText>,
    #[template_child]
    pub parameter_value_renderer: TemplateChild<gtk::CellRendererText>,

    // toolbar buttons
    #[template_child]
    pub new: TemplateChild<gtk::Button>,
    #[template_child]
    pub open: TemplateChild<gtk::Button>,
    #[template_child]
    pub save: TemplateChild<gtk::Button>,
    #[template_child]
    pub save_as: TemplateChild<gtk::Button>,
    #[template_child]
    pub bake_and_export: TemplateChild<gtk::Button>,

    // inspector buttons
    #[template_child]
    pub add_component: TemplateChild<gtk::Button>,
    #[template_child]
    pub remove_component: TemplateChild<gtk::Button>,

    // sb buttons
    #[template_child]
    pub add_entity: TemplateChild<gtk::Button>,
    #[template_child]
    pub remove_entity: TemplateChild<gtk::Button>,
    #[template_child]
    pub rename_entity: TemplateChild<gtk::Button>,
    #[template_child]
    pub export_entity: TemplateChild<gtk::Button>,

    pub sb_treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
    pub it_treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
    pub worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
    pub window: Arc<Mutex<Option<gtk::ApplicationWindow>>>,
    pub current_entity_id: Arc<Mutex<Option<u64>>>,
    pub current_component_name: Arc<Mutex<Option<String>>>,
    pub current_world_path: Arc<Mutex<Option<String>>>,
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

pub fn regen_inspector_from_component(it_treestore: Arc<Mutex<Option<gtk::TreeStore>>>, component: &mut Component) {
    let mut model = it_treestore.lock().unwrap();
    let model = model.as_ref().unwrap();
    model.clear();
    let root = model.append(None);
    model.set(&root, &[(0, &Value::from(component.get_name()))]);
    for (_, property) in component.get_parameters() {
        let property_node = model.append(Some(&root));
        model.set(&property_node, &[(0, &Value::from(property.name.clone().as_str()))]);
        let parameter_value = property.value.clone();
        match parameter_value {
            ParameterValue::String(value) => {
                model.set(&property_node, &[(1, &Value::from(value.as_str()))]);
            }
            ParameterValue::Float(value) => {
                model.set(&property_node, &[(1, &Value::from(value.to_string().as_str()))]);
            },
            ParameterValue::Int(value) => {
                model.set(&property_node, &[(1, &Value::from(value.to_string().as_str()))]);
            },
            ParameterValue::Bool(value) => {
                model.set(&property_node, &[(1, &Value::from(value.to_string().as_str()))]);
            },
            ParameterValue::Vec3(value) => {
                let x = value.x.to_string();
                let y = value.y.to_string();
                let z = value.z.to_string();
                model.set(&property_node, &[(1, &Value::from(format!("{},{},{}", x, y, z).as_str()))]);
            },
            ParameterValue::Quaternion(value) => {
                let ypr = Quaternion::to_euler_angles_zyx(&value);
                let yaw = ypr.x.to_string();
                let pitch = ypr.y.to_string();
                let roll = ypr.z.to_string();
                model.set(&property_node, &[(1, &Value::from(format!("{},{},{}", yaw, pitch, roll).as_str()))]);
            },
            _ => {
                model.set(&property_node, &[(1, &Value::from("unknown"))]);
                warn!("unknown parameter type");
            }
        }
    }
}

pub fn inspector_blank_slate(it_treestore: Arc<Mutex<Option<gtk::TreeStore>>>) {
    let mut model = it_treestore.lock().unwrap();
    let model = model.as_ref().unwrap();
    model.clear();
    let root = model.append(None);
    model.set(&root, &[(0, &Value::from("No entity selected"))]);
}

pub fn get_component_from_sb_treepath(sb_treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
                                      worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
                                      path: &gtk::TreePath,
                                      entity_id_to_set: Arc<Mutex<Option<u64>>>) -> Option<Component> {
    let model = sb_treestore.clone();
    let model = model.lock().unwrap();
    let model = model.as_ref().unwrap();
    let mut iter = model.iter_from_string(&*path.to_str().unwrap()).unwrap();
    let mut path = path.clone();
    let mut component = None;
    let component_name = model.get_value(&iter, 0).get::<String>().unwrap();
    // the entity containing this component should be the first parent of the component
    let mut entity_id = None;
    let mut saved_entity_id = model.get_value(&iter, 1).get::<String>();
    while let Some(parent) = model.iter_parent(&iter) {
        iter = parent;
        let tmp = model.get_value(&iter, 1).get::<String>();
        // if tmp is ok, then we have found the entity id
        if let Ok(tmp) = tmp {
            entity_id = Some(tmp);
            break;
        }
        path.up();
    }

    if entity_id.is_none() {
        // we didn't find an entity id, so we can't get the component
        // however, we might be able to get the entity id from the actual current selected row
        // set the current entity id
        if let Ok(id) = saved_entity_id {
            let mut current_entity_id = entity_id_to_set.lock().unwrap();
            *current_entity_id = id.parse::<u64>().ok();
        } else {
            let mut current_entity_id = entity_id_to_set.lock().unwrap();
            *current_entity_id = Option::None;
        }
    }
    let entity_id = entity_id?;
    // set the current entity id
    let mut current_entity_id = entity_id_to_set.lock().unwrap();
    *current_entity_id = Some(entity_id.parse::<u64>().unwrap());
    // get the worldmachine
    let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
    let worldmachine = worldmachine.lock().unwrap();
    // get the entity
    let entity = worldmachine.get_entity(str::parse(&entity_id).unwrap()).unwrap();
    // get the component
    entity.lock().unwrap().get_components().iter().for_each(|c| {
        if c.get_name() == component_name {
            component = Some(c.deref().clone());
        }
    });
    component
}

pub fn saveas(predetermined_path: Option<String>, window: Arc<Mutex<Option<gtk::ApplicationWindow>>>, worldmachine: Arc<Mutex<WorldMachine>>) {
    let window = window.lock().unwrap().as_ref().unwrap().clone();
    let dialog = gtk::FileChooserDialog::new(Some("Save World"), Some(&window), gtk::FileChooserAction::Save, &[("Cancel", gtk::ResponseType::Cancel), ("Save", gtk::ResponseType::Accept)]);
    if let Some(predetermined_path) = predetermined_path {
        dialog.set_file(&gio::File::for_path(&predetermined_path));
    }
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            let mut worldmachine = worldmachine.lock().unwrap();
            let path = dialog.file().unwrap().path().unwrap();
            worldmachine.save_state_to_file(&path.to_str().unwrap());
        }
        dialog.close();
    });
    dialog.show();
}

pub fn open(predetermined_path: Option<String>, window: Arc<Mutex<Option<gtk::ApplicationWindow>>>, worldmachine: Arc<Mutex<WorldMachine>>) {
    let window = window.lock().unwrap().as_ref().unwrap().clone();
    let dialog = gtk::FileChooserDialog::new(Some("Open World"), Some(&window), gtk::FileChooserAction::Open, &[("Cancel", gtk::ResponseType::Cancel), ("Open", gtk::ResponseType::Accept)]);
    if let Some(predetermined_path) = predetermined_path {
        dialog.set_file(&gio::File::for_path(&predetermined_path));
    }
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            let mut worldmachine = worldmachine.lock().unwrap();
            let path = dialog.file().unwrap().path().unwrap();
            worldmachine.load_state_from_file(&path.to_str().unwrap());
        }
        dialog.close();
    });
    dialog.show();
}

impl Editor {
    pub fn setup(&self, obj: &<Editor as ObjectSubclass>::Type) {
        // create a treemodel for the scene browser
        let mut model = self.sb_treestore.lock().unwrap();
        *model = Some(gtk::TreeStore::new(&[Type::STRING, Type::STRING]));
        let model = model.as_ref().unwrap();
        let root = model.append(None);
        model.set(&root, &[(0, &Value::from("worldmachine"))]);
        self.scene_browser.set_model(Some(model));
        // setup a cell renderer
        let cell = gtk::CellRendererText::new();
        self.entity_column.pack_start(&cell, true);
        self.entity_column.add_attribute(&cell, "text", 0);
        let cell = gtk::CellRendererText::new();
        self.entity_id_column.pack_start(&cell, true);
        self.entity_id_column.add_attribute(&cell, "text", 1);

        self.scene_browser.set_activate_on_single_click(true);

        // setup the clicking callback for the scene browser
        #[derive(Clone)]
        struct ClickedData {
            sb_treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
            it_treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
            worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
            inspector_tree: gtk::TreeView,
            entity_id_to_set: Arc<Mutex<Option<u64>>>,
            component_name_to_set: Arc<Mutex<Option<String>>>,
        }
        let clicked_data = ClickedData {
            sb_treestore: self.sb_treestore.clone(),
            it_treestore: self.it_treestore.clone(),
            worldmachine: self.worldmachine.clone(),
            inspector_tree: self.inspector_tree.clone(),
            entity_id_to_set: self.current_entity_id.clone(),
            component_name_to_set: self.current_component_name.clone(),
        };
        self.scene_browser.connect_row_activated(clone!(@strong clicked_data as cd => move |_, path, _| {
            let sb_treestore = cd.sb_treestore.clone();
            let it_treestore = cd.it_treestore.clone();
            let worldmachine = cd.worldmachine.clone();
            let component = get_component_from_sb_treepath(sb_treestore, worldmachine, path, cd.entity_id_to_set.clone());
            if let Some(component) = component {
                let component_name = component.name.clone();
                regen_inspector_from_component(it_treestore, &mut Box::new(component));
                cd.component_name_to_set.lock().unwrap().replace(component_name.to_string());
            } else {
                inspector_blank_slate(it_treestore);
            }
            cd.inspector_tree.expand_all();
        }));


        // do the same for the inspector
        let mut model = self.it_treestore.lock().unwrap();
        *model = Some(gtk::TreeStore::new(&[Type::STRING, Type::STRING]));
        let model = model.as_ref().unwrap();
        let root = model.append(None);
        model.set(&root, &[(0, &Value::from("parameter")), (1, &Value::from("value"))]);
        self.inspector_tree.set_model(Some(model));
        self.inspector_tree.set_enable_search(false);
        // setup a cell renderer
        self.parameter_column.add_attribute(&self.parameter_name_renderer.get(), "text", 0);
        self.value_column.add_attribute(&self.parameter_value_renderer.get(), "text", 1);
        self.parameter_value_renderer.set_foreground(Some("red"));
        self.value_column.set_clickable(true);

        // setup the edit callback for the value column
        let it_treestore = self.it_treestore.clone();
        let worldmachine = self.worldmachine.clone();
        let entity_id_to_set = self.current_entity_id.clone();
        self.parameter_value_renderer.connect_edited(move |_, path, new_text| {
            {
                let mut model = it_treestore.lock().unwrap();
                let model = model.as_ref().unwrap();
                let iter = model.iter_from_string(&*path.to_str().unwrap()).unwrap();
                model.set(&iter, &[(1, &Value::from(new_text))]);
            }
            // now we need to set the value on the component
            let mut path = path.clone();
            let mut entity_id = entity_id_to_set.lock().unwrap();
            if let Some(entity_id) = *entity_id {
                let model = it_treestore.clone();
                let model = model.lock().unwrap();
                let model = model.as_ref().unwrap();
                let mut iter = model.iter_from_string(&*path.to_str().unwrap()).unwrap();
                let mut path = path.clone();
                // we need to:
                // find the entity
                // figure out what type to cast the value to
                // set the value on the component
                let mut component_name = None;
                let mut property_name = None;
                // set the property name
                property_name = Some(model.get_value(&iter, 0).get::<String>().unwrap());
                while let Some(parent) = model.iter_parent(&iter) {
                    iter = parent;
                    let tmp = model.get_value(&iter, 0).get::<String>();
                    // if tmp is ok, then we have found the component name
                    if let Ok(tmp) = tmp {
                        component_name = Some(tmp);
                        break;
                    }
                    path.up();
                }
                if let Some(component_name) = component_name {
                    // get the worldmachine
                    let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
                    let mut worldmachine = worldmachine.lock().unwrap();
                    worldmachine.attempt_to_set_component_property(entity_id, component_name, property_name.unwrap(), new_text.to_string());
                }
            }
        });

        // setup the callback for clicking the save button
        let worldmachine = self.worldmachine.clone();
        let window = self.window.clone();
        let current_world_path = self.current_world_path.clone();
        self.save.connect_clicked(move |_| {
            let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
            let window = window.clone();
            let current_world_path = current_world_path.lock().unwrap();
            if let Some(current_world_path) = current_world_path.as_ref() {
                let mut worldmachine = worldmachine.lock().unwrap();
                worldmachine.save_state_to_file(current_world_path);
            } else {
                saveas(None, window, worldmachine);
            }
        });
        // setup the callback for clicking the save as button
        let worldmachine = self.worldmachine.clone();
        let window = self.window.clone();
        let current_world_path = self.current_world_path.clone();
        self.save_as.connect_clicked(move |_| {
            let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
            let window = window.clone();
            let current_world_path = current_world_path.lock().unwrap();
            if let Some(current_world_path) = current_world_path.as_ref() {
                saveas(Some(current_world_path.clone()), window, worldmachine);
            } else {
                saveas(None, window, worldmachine);
            }
        });

        // setup the callback for clicking the open button
        let worldmachine = self.worldmachine.clone();
        let window = self.window.clone();
        let current_world_path = self.current_world_path.clone();
        self.open.connect_clicked(move |_| {
            let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
            let window = window.clone();
            let current_world_path = current_world_path.clone();
            let current_world_path = current_world_path.lock().unwrap();
            if let Some(current_world_path) = current_world_path.as_ref() {
                open(Some(current_world_path.clone()), window, worldmachine);
            } else {
                open(None, window, worldmachine);
            }
        });

        // setup the callback for clicking the new button
        let worldmachine = self.worldmachine.clone();
        let window = self.window.clone();
        let current_world_path = self.current_world_path.clone();
        self.new.connect_clicked(move |_| {
            let worldmachine = worldmachine.clone();
            let window = window.clone();
            let windowinner = window.clone();
            let windowinner = windowinner.lock().unwrap();
            let windowinner = windowinner.as_ref().unwrap();
            let current_world_path = current_world_path.clone();
            // prompt the user to save the current world
            let dialog = MessageDialog::new(Some(windowinner), DialogFlags::MODAL, MessageType::Question, ButtonsType::YesNo, "Would you like to save the current world?");
            dialog.set_title(Some("Save Current World?"));
            dialog.connect_response(move |dialog, response| {
                match response {
                    ResponseType::Yes => {
                        let worldmachine_clone = worldmachine.clone();
                        let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
                        let current_world_path = current_world_path.lock().unwrap();
                        if let Some(current_world_path) = current_world_path.as_ref() {
                            let worldmachine = worldmachine.clone();
                            let mut worldmachine = worldmachine.lock().unwrap();
                            worldmachine.save_state_to_file(current_world_path);
                        } else {
                            let window = window.clone();
                            let windowinner = window.lock().unwrap();
                            let windowinner = windowinner.as_ref().unwrap();
                            saveas(None, Arc::new(Mutex::new(Some(windowinner.clone()))), worldmachine);
                        }
                        let worldmachine = worldmachine_clone;
                        let worldmachine = worldmachine.lock().unwrap();
                        let worldmachine = worldmachine.as_ref().unwrap();
                        let mut worldmachine = worldmachine.lock().unwrap();
                        worldmachine.blank_slate();
                        dialog.destroy();
                    },
                    ResponseType::No => {
                        let worldmachine = worldmachine.lock().unwrap();
                        let worldmachine = worldmachine.as_ref().unwrap();
                        let mut worldmachine = worldmachine.lock().unwrap();
                        worldmachine.blank_slate();
                        dialog.destroy();
                    },
                    _ => {}
                }
            });
            dialog.show();
        });

        // setup the callback for clicking the add entity button
        let worldmachine = self.worldmachine.clone();
        let renderer = self.renderer.clone();
        let window = self.window.clone();
        self.add_entity.connect_clicked(move |_| {
            let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
            let window = window.clone();
            let window_clone = window.clone();
            let window = window.lock().unwrap();
            let window = window.as_ref().unwrap();
            let renderer = renderer.clone();
            let renderer = renderer.lock().unwrap();

            let mut entity_picker = EntityPicker::new();
            // set worldmachine
            *entity_picker.imp().worldmachine.clone().lock().unwrap() = Some(worldmachine);
            *entity_picker.imp().renderer.clone().lock().unwrap() = renderer.clone();
            entity_picker.imp().populate_listbox();
            entity_picker.show();
        });

        // setup the callback for clicking the remove entity button
        let worldmachine = self.worldmachine.clone();
        let current_entity_id = self.current_entity_id.clone();
        let it_treestore = self.it_treestore.clone();
        let window = self.window.clone();
        self.remove_entity.connect_clicked(move |_| {
            let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
            let window = window.clone();
            let window = window.lock().unwrap();
            let window = window.as_ref().unwrap();
            let it_treestore = it_treestore.clone();
            let current_entity_id = current_entity_id.clone();
            let dialog = MessageDialog::new(Some(window), DialogFlags::MODAL, MessageType::Question, ButtonsType::YesNo, "Are you sure you want to delete this entity?");
            dialog.set_title(Some("Delete Entity?"));
            dialog.connect_response(move |dialog, response| {
                match response {
                    ResponseType::Yes => {
                        let current_entity_id = current_entity_id.lock().unwrap();
                        if let Some(current_entity_id) = current_entity_id.as_ref() {
                            let mut worldmachine = worldmachine.lock().unwrap();
                            let index = worldmachine.get_entity_index(*current_entity_id);
                            if let Some(index) = index {
                                worldmachine.remove_entity_at_index(index);
                                inspector_blank_slate(it_treestore.clone());
                            }
                        }
                        dialog.destroy();
                    }
                    ResponseType::No => {
                        dialog.destroy();
                    }
                    _ => {}
                }
            });
            dialog.show();
        });

        // setup the callback for clicking the rename entity button
        let worldmachine = self.worldmachine.clone();
        let current_entity_id = self.current_entity_id.clone();
        let it_treestore = self.it_treestore.clone();
        let window = self.window.clone();
        self.rename_entity.connect_clicked(move |_| {
            let entity_namer = EntityNamer::new();
            entity_namer.imp().cancel_button.connect_clicked(clone!(@weak entity_namer => move |_| {
                entity_namer.destroy();
            }));
            #[derive(Clone)]
            struct EntityNamerData {
                pub worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
                pub current_entity_id: Arc<Mutex<Option<u64>>>,
                pub entity_namer: EntityNamer,
            }
            let data = EntityNamerData {
                worldmachine: worldmachine.clone(),
                current_entity_id: current_entity_id.clone(),
                entity_namer: entity_namer.clone(),
            };
            entity_namer.imp().ok_button.connect_clicked(move |_| {
                let data = data.clone();
                let worldmachine = data.worldmachine.lock().unwrap().as_ref().unwrap().clone();
                let mut worldmachine = worldmachine.lock().unwrap();
                let current_entity_id = data.current_entity_id.lock().unwrap().unwrap();
                let name = data.entity_namer.imp().name_entry.text().to_string();
                worldmachine.rename_entity(current_entity_id, name.as_str());
                data.entity_namer.destroy();
            });
            entity_namer.show();
        });

        // setup the callback for clicking the export entity button
        let worldmachine = self.worldmachine.clone();
        let current_entity_id = self.current_entity_id.clone();
        let window = self.window.clone();
        self.export_entity.connect_clicked(move |_| {
            let worldmachine = worldmachine.lock().unwrap();
            let worldmachine = worldmachine.as_ref().unwrap();
            let mut worldmachine = worldmachine.lock().unwrap();
            let current_entity_id = current_entity_id.lock().unwrap();
            let current_entity_id = current_entity_id.as_ref().unwrap();
            worldmachine.save_entity_def(*current_entity_id);
        });

        // setup the callback for clicking the add component button
        let worldmachine = self.worldmachine.clone();
        let current_entity_id = self.current_entity_id.clone();
        let it_treestore = self.it_treestore.clone();
        let window = self.window.clone();
        self.add_component.connect_clicked(move |_| {
            if let Some(id) = current_entity_id.lock().unwrap().clone() {
                let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
                let window = window.clone();
                let window = window.lock().unwrap();
                let window = window.as_ref().unwrap();
                let it_treestore = it_treestore.clone();
                let mut component_picker = ComponentPicker::new();
                // set worldmachine
                *component_picker.imp().worldmachine.clone().lock().unwrap() = Some(worldmachine);
                *component_picker.imp().selected_entity_id.clone().lock().unwrap() = id;
                component_picker.imp().populate_listbox();
                component_picker.show();
            }
        });

        // setup the callback for clicking the remove component button
        let worldmachine = self.worldmachine.clone();
        let current_entity_id = self.current_entity_id.clone();
        let current_component_name = self.current_component_name.clone();
        let it_treestore = self.it_treestore.clone();
        let window = self.window.clone();
        self.remove_component.connect_clicked(move |_| {
            if let Some(id) = current_entity_id.lock().unwrap().clone() {
                if let Some(cname) = current_component_name.lock().unwrap().clone() {
                    let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
                    let window = window.clone();
                    let window = window.lock().unwrap();
                    let window = window.as_ref().unwrap();
                    let it_treestore = it_treestore.clone();
                    let current_entity_id = current_entity_id.clone();
                    let dialog = MessageDialog::new(Some(window), DialogFlags::MODAL, MessageType::Question, ButtonsType::YesNo, "Are you sure you want to delete this component?");
                    dialog.set_title(Some("Delete Component?"));
                    dialog.connect_response(move |dialog, response| {
                        match response {
                            ResponseType::Yes => {
                                let mut worldmachine = worldmachine.lock().unwrap();
                                let cname = cname.clone();
                                let component_type = COMPONENT_TYPES.lock().unwrap().get(&cname).unwrap().clone();
                                worldmachine.remove_component_from_entity(id, component_type);
                                inspector_blank_slate(it_treestore.clone());
                                dialog.destroy();
                            }
                            ResponseType::No => {
                                dialog.destroy();
                            }
                            _ => {}
                        }
                    });
                    dialog.show();
                }
            }
        });

        // bake & export button
        let worldmachine = self.worldmachine.clone();
        let current_world_path = self.current_world_path.clone();
        let window = self.window.clone();
        self.bake_and_export.connect_clicked(move |_| {
            let worldmachine = worldmachine.lock().unwrap().as_ref().unwrap().clone();
            let current_world_path = current_world_path.lock().unwrap().clone();
            let current_world_path = current_world_path.as_ref();
            if current_world_path.is_none() {
                warn!("No world path set, cannot bake and export");
                saveas(None, window.clone(), worldmachine.clone());
                let warning_dialog = MessageDialog::new(Some(window.lock().unwrap().as_ref().unwrap()), DialogFlags::MODAL, MessageType::Warning, ButtonsType::Ok, "no world path set, please save and try again");
                warning_dialog.set_title(Some("No World Path Set"));
                warning_dialog.connect_response(|dialog, _| {
                    dialog.destroy();
                });
                warning_dialog.show();
                return;
            }
            let current_world_path = current_world_path.unwrap();
            // get the last part of the path and remove extension if it exists
            let world_name = Path::new(current_world_path).file_stem();
            if let Some(world_name) = world_name {
                let world_name = world_name.to_str().unwrap();
                let world_name = world_name.to_string();
                let mut worldmachine = worldmachine.lock().unwrap();
                worldmachine.compile_map(world_name.as_str());
            } else {
                error!("could not get world name from path: {}", current_world_path);
            }
        });
    }

    pub fn regen_model_from_world(&self, wm: &mut World) {
        let mut model = self.sb_treestore.lock().unwrap();
        let model = model.as_ref().unwrap();
        model.clear();
        let root = model.append(None);
        model.set(&root, &[(0, &Value::from("worldmachine"))]);
        for entity in wm.clone().entities {
            let entity_node = model.append(Some(&root));
            model.set(&entity_node, &[(0, &Value::from(entity.get_name())), (1, &Value::from(entity.get_id().to_string().as_str()))]);
            for component in entity.get_components() {
                let component_node = model.append(Some(&entity_node));
                model.set(&component_node, &[(0, &Value::from(component.get_name()))]);
            }
        }

        self.scene_browser.expand_all();
    }
}

impl WidgetImpl for Editor {}
impl WindowImpl for Editor {}
impl BoxImpl for Editor {}