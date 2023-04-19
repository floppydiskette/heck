use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use egui_glfw_gl::egui;
use egui_glfw_gl::egui::{AboveOrBelow, ScrollArea, Ui};
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use crate::ui::WANT_SAVE;
use crate::worldmachine::{EntityId, WorldMachine};
use crate::worldmachine::ecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityListWant {
    None,
    Close,
    AddEntity,
    AddPrefab,
    DeleteEntity,
    DuplicateEntity,
}

lazy_static!{
    pub static ref STATE: Arc<Mutex<EntityListState>> = Arc::new(Mutex::new(EntityListState::default()));
}

pub struct EntityListState {
    pub entities: HashMap<EntityId, (bool, String)>,
    eids: Vec<EntityId>,
    entity_name_buffer: String,
}

impl Default for EntityListState {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            eids: vec![],
            entity_name_buffer: "".to_string(),
        }
    }
}

pub fn entity_list(ui: &mut Ui, wm: &mut WorldMachine, want: &mut EntityListWant) {
    ui.horizontal(|ui| {
        if ui.button("add entity").clicked() {
            *want = EntityListWant::AddEntity;
        }
        if ui.button("add prefab").clicked() {
            *want = EntityListWant::AddEntity;
        }
        if ui.button("delete entity").clicked() {
            *want = EntityListWant::DeleteEntity;
        }
        if ui.button("duplicate entity").clicked() {
            *want = EntityListWant::DuplicateEntity;
        }
    });

    ui.separator();

    ScrollArea::both()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let mut state = STATE.lock().unwrap();
                let mut found = Vec::new();
                let mut to_delete = Vec::new();
                let mut to_add = Vec::new();
                let entity_list = &mut wm.world.entities;
                for entity in entity_list {
                    found.push(entity.uid);
                    if let Entry::Vacant(e) = state.entities.entry(entity.uid) {
                        e.insert((false, entity.name.clone()));
                        state.eids.push(entity.uid);
                    }
                    let name = state.entities.get(&entity.uid).unwrap().1.clone();
                    let check = ui.checkbox(&mut state.entities.get_mut(&entity.uid).unwrap().0, name.clone());

                    if *want == EntityListWant::DeleteEntity && state.entities.get(&entity.uid).unwrap().0 {
                        to_delete.push(entity.uid);
                    } else if *want == EntityListWant::DuplicateEntity && state.entities.get(&entity.uid).unwrap().0 {
                        let mut new_entity = Entity::new(&name);
                        let components = entity.components.clone();
                        for component in components {
                            new_entity.add_component(component);
                        }
                        to_add.push(new_entity);
                    }
                }
                for eid in state.eids.clone().iter() {
                    if !found.contains(eid) {
                        WANT_SAVE.store(true, Ordering::Relaxed);
                        state.entities.remove(eid);
                        state.eids.retain(|x| x != eid);
                    }
                }
                for eid in to_delete {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                    wm.world.entities.retain(|x| x.uid != eid);
                    state.entities.remove(&eid);
                    state.eids.retain(|x| x != &eid);
                }
                for new_entity in to_add {
                    WANT_SAVE.store(true, Ordering::Relaxed);
                    wm.world.entities.push(new_entity);
                }
            });
        });
}

pub fn add_entity(ui: &mut Ui, wm: &mut WorldMachine, want: &mut EntityListWant) {
    let entity_list = &mut wm.world.entities;
    let mut state = STATE.lock().unwrap();

    ui.label("entity name");
    let done = ui.text_edit_singleline(&mut state.entity_name_buffer).lost_focus();

    ui.horizontal(|ui| {
        if ui.button("ok").clicked() || (done && !state.entity_name_buffer.is_empty()) {
            WANT_SAVE.store(true, Ordering::Relaxed);
            let entity = Entity::new( &state.entity_name_buffer.clone());
            entity_list.push(entity);

            state.entity_name_buffer.clear();
            * want = EntityListWant::Close;
        }
    });
}