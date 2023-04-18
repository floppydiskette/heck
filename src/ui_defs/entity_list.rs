use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use egui_glfw_gl::egui;
use egui_glfw_gl::egui::{AboveOrBelow, ScrollArea, Ui};
use std::sync::{Arc, Mutex};
use crate::worldmachine::{EntityId, WorldMachine};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityListWant {
    None,
    Close,
    AddEntity,
    AddPrefab,
    DeleteEntity,
}

lazy_static!{
    static ref STATE: Arc<Mutex<EntityListState>> = Arc::new(Mutex::new(EntityListState::default()));
}

struct EntityListState {
    pub entities: HashMap<EntityId, (bool, String)>,
    pub eids: Vec<EntityId>,
}

impl Default for EntityListState {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            eids: vec![],
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
    });

    ui.separator();

    ScrollArea::both()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let mut state = STATE.lock().unwrap();
                let mut found = Vec::new();
                let mut to_delete = Vec::new();
                let entity_list = &mut wm.world.entities;
                for entity in entity_list {
                    found.push(entity.uid);
                    if let Entry::Vacant(e) = state.entities.entry(entity.uid) {
                        e.insert((false, entity.name.clone()));
                        state.eids.push(entity.uid);
                    }
                    let name = state.entities.get(&entity.uid).unwrap().1.clone();
                    let check = ui.checkbox(&mut state.entities.get_mut(&entity.uid).unwrap().0, name);

                    if *want == EntityListWant::DeleteEntity && state.entities.get(&entity.uid).unwrap().0 {
                        to_delete.push(entity.uid);
                    }
                }
                for eid in state.eids.clone().iter() {
                    if !found.contains(eid) {
                        state.entities.remove(eid);
                        state.eids.retain(|x| x != eid);
                    }
                }
                for eid in to_delete {
                    wm.world.entities.retain(|x| x.uid != eid);
                    state.entities.remove(&eid);
                    state.eids.retain(|x| x != &eid);
                }
            });
        });
}

pub fn add_entity(ui: &mut Ui, wm: &mut WorldMachine, want: &mut EntityListWant) {
    let entity_list = &mut wm.world.entities;

    if ui.button("ok").clicked() {
        *want = EntityListWant::Close;
    }
}