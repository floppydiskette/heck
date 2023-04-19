use serde::Serialize;
use crate::renderer::BASE_DIR;
use crate::worldmachine::ecs::ENTITY_ID_MANAGER;
use crate::worldmachine::{WorldDef, WorldMachine};

pub fn load_state_from_file(wm: &mut WorldMachine, file_path: &str) -> bool {
    let file_path = if file_path.starts_with("/") {
        file_path.to_string()
    } else {
        format!("{}/work/maps/{}", BASE_DIR, file_path)
    };

    let contents = std::fs::read_to_string(file_path);
    if contents.is_err() {
        warn!("failed to load state from file: {}", contents.err().unwrap());
        return false;
    }
    let contents = contents.unwrap();
    let world = serde_yaml::from_str(&contents);
    if world.is_err() {
        warn!("failed to load state from file: {}", world.err().unwrap());
        return false;
    }
    let world = world.unwrap();
    wm.world = world;

    {
        let mut eid_man = ENTITY_ID_MANAGER.lock().unwrap();
        eid_man.id = wm.world.eid_manager;
    }
    wm.entities_wanting_to_load_things.clear();
    for i in 0..wm.world.entities.len() {
        wm.entities_wanting_to_load_things.push(i);
    }

    let mut insp_state = crate::ui_defs::entity_inspector::STATE.lock().unwrap();
    insp_state.edit_buffer.clear();
    insp_state.components.clear();

    let mut list_state = crate::ui_defs::entity_list::STATE.lock().unwrap();
    list_state.entities.clear();

    wm.selected_entity = None;

    true
}

pub fn save_state_to_file(wm: &mut WorldMachine, file_path: &str) -> bool {
    let file_path = if file_path.starts_with("/") {
        file_path.to_string()
    } else {
        format!("{}/work/maps/{}", BASE_DIR, file_path)
    };
    {
        let eid_man = ENTITY_ID_MANAGER.lock().unwrap();
        wm.world.eid_manager = eid_man.id;
    }
    let serialised = serde_yaml::to_string(&wm.world);
    if serialised.is_err() {
        warn!("failed to serialise world: {}", serialised.err().unwrap());
        return false;
    }
    let serialised = serialised.unwrap();
    if std::fs::write(file_path, serialised).is_err() {
        warn!("failed to write world to file");
        return false;
    }
    true

}

pub fn compile(wm: &mut WorldMachine, name: &str) -> bool {
    // create a directory for the map (if it doesn't exist)
    let map_dir = format!("{}/maps/{}", BASE_DIR, name);
    let res = std::fs::create_dir_all(map_dir.clone());
    if res.is_err() {
        error!("failed to create map directory: {}", res.err().unwrap());
        return false;
    }

    let worlddef = WorldDef {
        name: String::from(name),
        world: wm.world.clone(),
    };

    let mut serialized = Vec::new();
    let res = worlddef.serialize(&mut rmp_serde::Serializer::new(&mut serialized));
    if res.is_err() {
        error!("failed to serialize worlddef: {}", res.err().unwrap());
        return false;
    }
    // write the worlddef to a file
    let res = std::fs::write(format!("{}/worlddef", map_dir), serialized);
    if res.is_err() {
        error!("failed to write worlddef: {}", res.err().unwrap());
        return false;
    }
    info!("wrote worlddef to file");
    true
}