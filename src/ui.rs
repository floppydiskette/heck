use std::collections::VecDeque;
use std::ops::Mul;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use egui_glfw_gl::egui;
use egui_glfw_gl::egui::{Align, CentralPanel, Frame, Rgba, Separator, SidePanel, TopBottomPanel, Ui};
use gfx_maths::Vec3;
use crate::renderer::ht_renderer;
use crate::ui_defs;
use crate::ui_defs::entity_inspector::InspectorWant;
use crate::ui_defs::entity_list::EntityListWant;
use crate::worldmachine::{savestates, WorldMachine};

lazy_static!{
    pub static ref STATE: Arc<Mutex<State>> = Arc::new(Mutex::new(State::default()));

    pub static ref SHOW_UI: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref SHOW_DEBUG_LOCATION: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref SHOW_FPS: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref FPS: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));
    pub static ref DEBUG_LOCATION: Arc<Mutex<Vec3>> = Arc::new(Mutex::new(Vec3::new(0.0, 0.0, 0.0)));
    pub static ref WANT_QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    pub static ref WANT_SAVE: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub struct State {
    pub ask_quit: bool,
    pub add_entity: bool,
    pub open_map: bool,
    pub save_map: bool,
    pub compile_map: bool,
    pub map_name: String,
    pub world_name: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ask_quit: false,
            add_entity: false,
            open_map: false,
            save_map: false,
            compile_map: false,
            map_name: "".to_string(),
            world_name: "".to_string(),
        }
    }
}

pub fn render(renderer: &mut ht_renderer, worldmachine: &mut WorldMachine) {
    if !SHOW_UI.load(Ordering::Relaxed) {
        return;
    }

    let window_size = renderer.window_size;

    egui::Window::new("debug")
        .default_size(egui::Vec2::new(200.0, 200.0))
        .default_pos(egui::Pos2::new(window_size.x, 50.0))
        .show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
            // right align
            if SHOW_DEBUG_LOCATION.load(Ordering::Relaxed) {
                render_debug_location(ui);
            }
            if SHOW_FPS.load(Ordering::Relaxed) {
                render_fps(ui);
            }
        });

    let mut want = EntityListWant::None;

    egui::Window::new("entity list")
        .default_size(egui::Vec2::new(200.0, 700.0))
        .default_pos(egui::Pos2::new(0.0, 50.0))
        .resizable(true)
        .show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
            ui_defs::entity_list::entity_list(ui, worldmachine, &mut want);
        });

    match want {
        EntityListWant::None => {}
        EntityListWant::Close => {}
        EntityListWant::AddEntity => {
            let mut state = STATE.lock().unwrap();
            state.add_entity = !state.add_entity;
        }
        EntityListWant::DeleteEntity => {}
        EntityListWant::AddPrefab => {}
    }

    if STATE.lock().unwrap().add_entity {
        let mut want = EntityListWant::None;

        egui::Window::new("add entity")
            .resizable(true)
            .show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
                WANT_SAVE.store(true, Ordering::Relaxed);
                ui_defs::entity_list::add_entity(ui, worldmachine, &mut want);
            });

        match want {
            EntityListWant::None => {}
            EntityListWant::Close => {
                let mut state = STATE.lock().unwrap();
                state.add_entity = false;
            }
            EntityListWant::AddEntity => {}
            EntityListWant::DeleteEntity => {}
            EntityListWant::AddPrefab => {}
        }
    }

    if STATE.lock().unwrap().open_map {
        egui::Window::new("open map")
            .resizable(true)
            .show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
                ui.label("please enter path to map: (either absolute or relative to `base/work/maps/`)");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut STATE.lock().unwrap().map_name);
                    if ui.button("open").clicked() {
                        let mut state = STATE.lock().unwrap();
                        state.open_map = false;
                        savestates::load_state_from_file(worldmachine, &state.map_name);
                    }
                    if ui.button("cancel").clicked() {
                        let mut state = STATE.lock().unwrap();
                        state.open_map = false;
                    }
                });
            });
    }

    if STATE.lock().unwrap().save_map {
        egui::Window::new("save map")
            .resizable(true)
            .show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
                ui.label("please enter path to map: (either absolute or relative to `base/work/maps/`)");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut STATE.lock().unwrap().map_name);
                    if ui.button("save").clicked() {
                        WANT_SAVE.store(false, Ordering::Relaxed);
                        let mut state = STATE.lock().unwrap();
                        state.save_map = false;
                        savestates::save_state_to_file(worldmachine, &state.map_name);
                    }
                    if ui.button("cancel").clicked() {
                        let mut state = STATE.lock().unwrap();
                        state.save_map = false;
                    }
                });
            });
    }

    if STATE.lock().unwrap().compile_map {
        egui::Window::new("compile map")
            .resizable(true)
            .show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
                ui.label("please enter name of map: (no path, just the name)");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut STATE.lock().unwrap().world_name);
                    if ui.button("compile").clicked() {
                        let mut state = STATE.lock().unwrap();
                        state.compile_map = false;
                        savestates::compile(worldmachine, &state.world_name);
                    }
                    if ui.button("cancel").clicked() {
                        let mut state = STATE.lock().unwrap();
                        state.compile_map = false;
                    }
                });
            });
    }

    let mut want = InspectorWant::None;

    egui::Window::new("entity inspector")
        .default_size(egui::Vec2::new(200.0, 700.0))
        .default_pos(egui::Pos2::new(window_size.x, window_size.y))
        .resizable(true)
        .show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
            ui_defs::entity_inspector::inspector(ui, worldmachine, &mut want);
        });

    match want {
        InspectorWant::None => {}
    }

    TopBottomPanel::top("menubar").show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
        let mut status = String::new();

        if WANT_SAVE.load(Ordering::Relaxed) {
            status.push_str("modified (awaiting save)");
        } else {
            status.push_str("not modified");
        }

        status.push_str(" | ");

        ui.label(status);

        ui.menu_button("File", |ui| {
            {
                let open = ui.button("open");
                if open.clicked() {
                    let mut state = STATE.lock().unwrap();
                    state.open_map = true;
                }
            }
            ui.separator();
            {
                let open = ui.button("save");
                if open.clicked() {
                    let mut state = STATE.lock().unwrap();
                    state.save_map = true;
                }
            }
            ui.separator();
            {
                let open = ui.button("compile");
                if open.clicked() {
                    let mut state = STATE.lock().unwrap();
                    state.compile_map = true;
                }
            }
            ui.separator();
            {
                let quit = ui.button("quit");
                if quit.clicked() {
                    WANT_QUIT.store(true, Ordering::Relaxed);
                }
            }
        });
    });

    let egui::FullOutput {
        platform_output,
        repaint_after: _,
        textures_delta,
        shapes,
    } = renderer.backend.egui_context.lock().unwrap().end_frame();

    //Handle cut, copy text from egui
    if !platform_output.copied_text.is_empty() {
        egui_glfw_gl::copy_to_clipboard(&mut renderer.backend.input_state.lock().unwrap(), platform_output.copied_text);
    }

    let clipped_shapes = renderer.backend.egui_context.lock().unwrap().tessellate(shapes);
    renderer.backend.painter.lock().unwrap().paint_and_update_textures(1.0, &clipped_shapes, &textures_delta);
}


fn render_debug_location(ui: &mut Ui) {
    let debug_location = DEBUG_LOCATION.lock().unwrap();
    ui.label(format!("x: {}, y: {}, z: {}", debug_location.x, debug_location.y, debug_location.z));
}

fn render_fps(ui: &mut Ui) {
    let fps = FPS.lock().unwrap();
    ui.label(format!("FPS: {}", *fps as u32));
}