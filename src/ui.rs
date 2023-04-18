use std::collections::VecDeque;
use std::ops::Mul;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use egui_glfw_gl::egui;
use egui_glfw_gl::egui::{CentralPanel, Frame, Rgba, SidePanel, TopBottomPanel, Ui};
use gfx_maths::Vec3;
use crate::renderer::ht_renderer;
use crate::ui_defs;
use crate::ui_defs::entity_list::EntityListWant;
use crate::worldmachine::WorldMachine;

lazy_static!{
    pub static ref STATE: Arc<Mutex<State>> = Arc::new(Mutex::new(State::default()));

    pub static ref SHOW_UI: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref SHOW_DEBUG_LOCATION: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref SHOW_FPS: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref FPS: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));
    pub static ref DEBUG_LOCATION: Arc<Mutex<Vec3>> = Arc::new(Mutex::new(Vec3::new(0.0, 0.0, 0.0)));
    pub static ref WANT_QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub struct State {
    pub add_entity: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            add_entity: false,
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

    TopBottomPanel::top("menubar").show(&renderer.backend.egui_context.lock().unwrap(), |ui| {
        ui.menu_button("File", |ui| {
            {
                let _ = ui.button("test 1");
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