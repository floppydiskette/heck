use std::any::Any;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use gfx_maths::{Quaternion, Vec3};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, Inhibit, GLArea, gdk, pango};
use gtk::gdk::ffi::GdkGLContext;
use gtk::gio::Menu;
use gtk::glib::{Type, Value};
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::renderer::H2eckRenderer;
use crate::worldmachine::{World, WorldMachine};
use crate::worldmachine::ecs::Component;


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

    pub sb_treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
    pub it_treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
    pub worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
    pub current_entity_id: Arc<Mutex<Option<u32>>>,
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

pub fn regen_inspector_from_component(it_treestore: Arc<Mutex<Option<gtk::TreeStore>>>, component: &mut Box<dyn Component>) {
    let mut model = it_treestore.lock().unwrap();
    let model = model.as_ref().unwrap();
    model.clear();
    let root = model.append(None);
    model.set(&root, &[(0, &Value::from(component.get_name().as_str()))]);
    for property in component.get_parameters() {
        let property_node = model.append(Some(&root));
        model.set(&property_node, &[(0, &Value::from(property.name.as_str()))]);
        let parameter_value = property.value;
        let parameter_type = parameter_value.deref().type_id();
        // switch on the type of the parameter
        let string_type = std::any::TypeId::of::<String>();
        let float_type = std::any::TypeId::of::<f32>();
        let int_type = std::any::TypeId::of::<i32>();
        let bool_type = std::any::TypeId::of::<bool>();
        let vec3_type = std::any::TypeId::of::<Vec3>();
        let quaternion_type = std::any::TypeId::of::<Quaternion>();

        match parameter_type {
            x if x == string_type => {
                let string_value = parameter_value.downcast_ref::<String>().unwrap();
                model.set(&property_node, &[(1, &Value::from(string_value.as_str()))]);
            },
            x if x == float_type => {
                let float_value = parameter_value.downcast_ref::<f32>().unwrap().to_string();
                model.set(&property_node, &[(1, &Value::from(float_value.as_str()))]);
            },
            x if x == int_type => {
                let int_value = parameter_value.downcast_ref::<i32>().unwrap().to_string();
                model.set(&property_node, &[(1, &Value::from(int_value.as_str()))]);
            },
            x if x == bool_type => {
                let bool_value = parameter_value.downcast_ref::<bool>().unwrap().to_string();
                model.set(&property_node, &[(1, &Value::from(bool_value.as_str()))]);
            },
            x if x == vec3_type => {
                let vec3_value = parameter_value.downcast_ref::<Vec3>().unwrap();
                let x = vec3_value.x.to_string();
                let y = vec3_value.y.to_string();
                let z = vec3_value.z.to_string();
                model.set(&property_node, &[(1, &Value::from(format!("{},{},{}", x, y, z).as_str()))]);
            },
            x if x == quaternion_type => {
                let quaternion_value = parameter_value.downcast_ref::<Quaternion>().unwrap();
                let ypr = Quaternion::to_euler_angles_zyx(quaternion_value);
                let yaw = ypr.x.to_string();
                let pitch = ypr.y.to_string();
                let roll = ypr.z.to_string();
                model.set(&property_node, &[(1, &Value::from(format!("{},{},{}", yaw, pitch, roll).as_str()))]);
            },
            _ => {
                model.set(&property_node, &[(1, &Value::from("unknown"))]);
            }
        }
    }
}

pub fn get_component_from_sb_treepath(sb_treestore: Arc<Mutex<Option<gtk::TreeStore>>>,
                                      worldmachine: Arc<Mutex<Option<Arc<Mutex<WorldMachine>>>>>,
                                      path: &gtk::TreePath,
                                      entity_id_to_set: Arc<Mutex<Option<u32>>>) -> Option<Box<dyn Component>> {
    let model = sb_treestore.clone();
    let model = model.lock().unwrap();
    let model = model.as_ref().unwrap();
    let mut iter = model.iter_from_string(&*path.to_str().unwrap()).unwrap();
    let mut path = path.clone();
    let mut component = None;
    let component_name = model.get_value(&iter, 0).get::<String>().unwrap();
    // the entity containing this component should be the first parent of the component
    let mut entity_id = None;
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
    let entity_id = entity_id?;
    // set the current entity id
    let mut current_entity_id = entity_id_to_set.lock().unwrap();
    *current_entity_id = Some(entity_id.parse::<u32>().unwrap());
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
            entity_id_to_set: Arc<Mutex<Option<u32>>>,
        }
        let clicked_data = ClickedData {
            sb_treestore: self.sb_treestore.clone(),
            it_treestore: self.it_treestore.clone(),
            worldmachine: self.worldmachine.clone(),
            inspector_tree: self.inspector_tree.clone(),
            entity_id_to_set: self.current_entity_id.clone(),
        };
        self.scene_browser.connect_row_activated(clone!(@strong clicked_data as cd => move |_, path, _| {
            let sb_treestore = cd.sb_treestore.clone();
            let it_treestore = cd.it_treestore.clone();
            let worldmachine = cd.worldmachine.clone();
            let component = get_component_from_sb_treepath(sb_treestore, worldmachine, path, cd.entity_id_to_set.clone());
            if let Some(component) = component {
                regen_inspector_from_component(it_treestore, &mut Box::new(component));
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
    }

    pub fn regen_model_from_world(&self, wm: &mut World) {
        let mut model = self.sb_treestore.lock().unwrap();
        let model = model.as_ref().unwrap();
        model.clear();
        let root = model.append(None);
        model.set(&root, &[(0, &Value::from("worldmachine"))]);
        for entity in wm.clone().entities {
            let entity_node = model.append(Some(&root));
            model.set(&entity_node, &[(0, &Value::from(entity.get_name().as_str())), (1, &Value::from(entity.get_id().to_string().as_str()))]);
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