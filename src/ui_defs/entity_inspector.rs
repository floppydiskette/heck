use std::collections::{BTreeMap, HashMap, HashSet};
use std::collections::hash_map::Entry;
use egui_glfw_gl::egui;
use egui_glfw_gl::egui::{AboveOrBelow, ScrollArea, Ui};
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use gfx_maths::Quaternion;
use crate::ui::WANT_SAVE;
use crate::ui_defs::entity_list;
use crate::worldmachine::{EntityId, WorldMachine};
use crate::worldmachine::components::{BoxCollider, COMPONENT_TYPE_BOX_COLLIDER, COMPONENT_TYPE_JUKEBOX, COMPONENT_TYPE_LIGHT, COMPONENT_TYPE_MESH_RENDERER, COMPONENT_TYPE_TERRAIN, COMPONENT_TYPE_TRANSFORM, COMPONENT_TYPE_TRIGGER, Jukebox, Light, MeshRenderer, Transform, Trigger};
use crate::worldmachine::ecs::{ComponentType, Entity, Parameter, ParameterValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorWant {
    None,
}

lazy_static!{
    pub static ref STATE: Arc<Mutex<InspectorState>> = Arc::new(Mutex::new(InspectorState::default()));
}

pub struct InspectorState {
    pub components: HashMap<ComponentType, (bool, BTreeMap<String, Parameter>)>,
    pub edit_buffer: HashMap<EntityId, HashMap<ComponentType, HashMap<String, Parameter>>>,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            components: HashMap::new(),
            edit_buffer: Default::default(),
        }
    }
}

pub fn inspector(ui: &mut Ui, wm: &mut WorldMachine, want: &mut InspectorWant) {
    let mut eid = None;
    let mut multiple_selected = false;
    let mut list_state = entity_list::STATE.lock().unwrap();
    for (id, (selected, _)) in &list_state.entities {
        if *selected {
            if eid.is_some() {
                multiple_selected = true;
                break;
            } else {
                eid = Some(id);
            }
        }
    }

    let entity_list = &mut wm.world.entities;

    if let Some(eid) = eid {
        wm.selected_entity = Some(*eid);
        let entity = entity_list.iter_mut().find(|e| e.uid == *eid);
        if let Some(entity) = entity {
            ui.horizontal(|ui| {
                ui.label(format!("entity: {}", entity.name));

                let response = ui.button("add component");
                let popup = ui.make_persistent_id("add_component_p");
                if response.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup));
                }

                egui::popup_above_or_below_widget(ui, popup, &response, AboveOrBelow::Above, |ui| {
                    if ui.button("transform").clicked() {
                        entity.add_component(Transform::default());
                        WANT_SAVE.store(true, Ordering::Relaxed);
                    }

                    if ui.button("mesh renderer").clicked() {
                        entity.add_component(MeshRenderer::default());
                        WANT_SAVE.store(true, Ordering::Relaxed);
                    }

                    if ui.button("light").clicked() {
                        entity.add_component(Light::default());
                        WANT_SAVE.store(true, Ordering::Relaxed);
                    }

                    if ui.button("box_collider").clicked() {
                        entity.add_component(BoxCollider::default());
                        WANT_SAVE.store(true, Ordering::Relaxed);
                    }

                    if ui.button("jukebox").clicked() {
                        entity.add_component(Jukebox::default());
                        WANT_SAVE.store(true, Ordering::Relaxed);
                    }

                    if ui.button("trigger").clicked() {
                        entity.add_component(Trigger::default());
                        WANT_SAVE.store(true, Ordering::Relaxed);
                    }
                });

                if ui.button("rm component").clicked() {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                    let mut to_remove = HashSet::new();
                    for (i, c) in entity.components.iter().enumerate() {
                        if let Some((selected, _)) = STATE.lock().unwrap().components.get(&c.component_type) {
                            if *selected {
                                to_remove.insert(i);
                            }
                        }
                    }

                    let mut i = 0;
                    for r in to_remove {
                        entity.components.remove(r - i);
                        i += 1;
                    }
                }

            });
            for i in 0..entity.components.len() {
                render_component(ui, entity, i, &mut STATE.lock().unwrap());
            }
        }
    } else if multiple_selected {
        wm.selected_entity = None;
        ui.label("multiple entities selected");
    } else {
        wm.selected_entity = None;
        ui.label("no entity selected");
    }
}

pub fn render_component(ui: &mut Ui, entity: &mut Entity, component: usize, state: &mut InspectorState) {
    if let Some(component) = entity.components.get_mut(component) {
        let ct = component.component_type.clone();
        let cvs = component.parameters.clone();
        if let Entry::Vacant(e) = state.components.entry(ct.clone()) {
            e.insert((false, cvs));
        }

        ui.checkbox(&mut state.components.get_mut(&ct).unwrap().0, &component.component_type.name);

        let edit_buffer = state.edit_buffer.get_mut(&entity.uid);
        let edit_buffer = match edit_buffer {
            Some(e) => e,
            None => {
                state.edit_buffer.insert(entity.uid, HashMap::new());
                state.edit_buffer.get_mut(&entity.uid).unwrap()
            }
        };

        match component.component_type.clone() {
            x if x == COMPONENT_TYPE_TRANSFORM.clone() => {
                let position = component.parameters.get_mut("position").unwrap();
                let position = match &mut position.value {
                    ParameterValue::Vec3(v) => v,
                    _ => panic!("invalid parameter type"),
                };
                let old_position = position.clone();

                ui.horizontal(|ui| {
                    ui.label("position");
                    ui.add(egui::DragValue::new(&mut position.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut position.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut position.z).speed(0.1));
                });
                if position != &old_position {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                }

                let rotation = component.parameters.get_mut("rotation").unwrap();
                let rotation = match &mut rotation.value {
                    ParameterValue::Quaternion(v) => v,
                    _ => panic!("invalid parameter type"),
                };

                let mut rotation_euler = rotation.to_euler_angles_zyx();
                let old_rotation_euler = rotation_euler.clone();
                ui.horizontal(|ui| {
                    ui.label("rotation");
                    ui.add(egui::DragValue::new(&mut rotation_euler.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut rotation_euler.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut rotation_euler.z).speed(0.1));
                });

                if rotation_euler != old_rotation_euler {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                }

                *rotation = Quaternion::from_euler_angles_zyx(&rotation_euler);

                let scale = component.parameters.get_mut("scale").unwrap();
                let scale = match &mut scale.value {
                    ParameterValue::Vec3(v) => v,
                    _ => panic!("invalid parameter type"),
                };
                let old_scale = scale.clone();
                ui.horizontal(|ui| {
                    ui.label("scale");
                    ui.add(egui::DragValue::new(&mut scale.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut scale.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut scale.z).speed(0.1));
                });
                if scale != &old_scale {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                }
            }
            x if x == COMPONENT_TYPE_MESH_RENDERER.clone() => {

                if let Entry::Vacant(e) = edit_buffer.entry(ct.clone()) {
                    e.insert(HashMap::new());
                }
                let mesh = component.parameters.get_mut("mesh").unwrap();

                let edit_buffer = edit_buffer.get_mut(&ct).unwrap();
                if let Entry::Vacant(e) = edit_buffer.entry("mesh".to_string()) {
                    e.insert(mesh.clone());
                }
                let mesh = match &mut edit_buffer.get_mut("mesh").unwrap().value {
                    ParameterValue::String(v) => v,
                    _ => panic!("invalid parameter type"),
                };
                let old_mesh = mesh.clone();

                ui.horizontal(|ui| {
                    ui.label("mesh");
                    if ui.text_edit_singleline(mesh).lost_focus() {
                        let mut actual_mesh = component.parameters.get_mut("mesh").unwrap();
                        actual_mesh.value = ParameterValue::String(mesh.clone());
                    }
                });
                if mesh != &old_mesh {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                }

                let texture = component.parameters.get_mut("texture").unwrap();
                if let Entry::Vacant(e) = edit_buffer.entry("texture".to_string()) {
                    e.insert(texture.clone());
                }
                let texture = match &mut edit_buffer.get_mut("texture").unwrap().value {
                    ParameterValue::String(v) => v,
                    _ => panic!("invalid parameter type"),
                };
                let old_texture = texture.clone();

                ui.horizontal(|ui| {
                    ui.label("texture");
                    if ui.text_edit_singleline(texture).lost_focus() {
                        let mut actual_texture = component.parameters.get_mut("texture").unwrap();
                        actual_texture.value = ParameterValue::String(texture.clone());
                    }
                });
                if texture != &old_texture {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                }
            }
            x if x == COMPONENT_TYPE_LIGHT.clone() => {
                let position = component.parameters.get_mut("position").unwrap();
                let position = match &mut position.value {
                    ParameterValue::Vec3(v) => v,
                    _ => panic!("invalid parameter type"),
                };
                let old_positon = position.clone();
                ui.horizontal(|ui| {
                    ui.label("position");
                    ui.add(egui::DragValue::new(&mut position.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut position.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut position.z).speed(0.1));
                });
                if position != &old_positon {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                }

                let colour = component.parameters.get_mut("colour").unwrap();
                let colour = match &mut colour.value {
                    ParameterValue::Vec3(v) => v,
                    _ => panic!("invalid parameter type"),
                };
                let old_colour = colour.clone();

                ui.horizontal(|ui| {
                    ui.label("colour");
                    ui.add(egui::DragValue::new(&mut colour.x).speed(0.1));
                    ui.add(egui::DragValue::new(&mut colour.y).speed(0.1));
                    ui.add(egui::DragValue::new(&mut colour.z).speed(0.1));
                });
                if colour != &old_colour {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                }

                let intensity = component.parameters.get_mut("intensity").unwrap();
                let intensity = match &mut intensity.value {
                    ParameterValue::Float(v) => v,
                    _ => panic!("invalid parameter type"),
                };
                let old_intensity = intensity.clone();

                ui.horizontal(|ui| {
                    ui.label("scale");
                    ui.add(egui::DragValue::new(intensity).speed(0.1));
                });
                if intensity != &old_intensity {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                }
            }

            _ => {
                ui.label("error: component data not programmed");
            }
        }
    } else {
        ui.label("error: component not found");
    }
}