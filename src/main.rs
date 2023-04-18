#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use std::ops::Deref;
use std::sync::atomic::Ordering;
use std::time::Instant;
use fyrox_sound::context::SoundContext;
use fyrox_sound::engine::SoundEngine;
use gfx_maths::{Quaternion, Vec3};
use crate::keyboard::HTKey;
use crate::mouse::MouseButtonState;
use crate::renderer::ht_renderer;

pub mod renderer;
pub mod worldmachine;
pub mod camera;
pub mod textures;
pub mod meshes;
pub mod light;
pub mod helpers;
pub mod audio;
pub mod keyboard;
pub mod mouse;
pub mod animation;
pub mod animgraph;
pub mod common_anim;
pub mod shaders;
pub mod skeletal_animation;
pub mod optimisations;
pub mod ui;
pub mod viewport;
pub mod ui_defs;


#[tokio::main]
async fn main() {
    // initialise env logger
    env_logger::init();

    let sengine = SoundEngine::new();
    let scontext = SoundContext::new();

    sengine.lock().unwrap().add_context(scontext.clone());

    let mut audio = crate::audio::AudioBackend::new();
    info!("initialised audio subsystem");

    info!("initialising h2eck renderer...");

    let renderer = ht_renderer::init();
    if let Err(e) = renderer {
        error!("failed to initialise renderer: {}", e);
        return;
    }
    let mut renderer = renderer.unwrap();
    renderer.initialise_basic_resources();
    info!("initialised renderer");

    let mut worldmachine = worldmachine::WorldMachine::default();
    worldmachine.initialise();
    pub const DEFAULT_FOV: f32 = 120.0;
    renderer.camera.set_fov(DEFAULT_FOV);

    let mut viewport = viewport::Viewport::init(Vec3::default(), Quaternion::default());

    info!("initialised worldmachine");
    let start_time = Instant::now();

    let mut camera_control = false;

    let mut last_frame_time = std::time::Instant::now();
    loop {
        let delta = (last_frame_time.elapsed().as_millis() as f64 / 1000.0) as f32;
        last_frame_time = Instant::now();

        // calculate fps based on delta
        let fps = 1.0 / delta;
        *crate::ui::FPS.lock().unwrap() = fps;

        renderer.backend.input_state.lock().unwrap().input.time = Some(start_time.elapsed().as_secs_f64());
        renderer.backend.egui_context.lock().unwrap().begin_frame(renderer.backend.input_state.lock().unwrap().input.take());

        // todo: maybe move this somewhere else?

        let mouse_state = mouse::get_mouse_button_state(1);

        if mouse_state == MouseButtonState::Pressed {
            renderer.lock_mouse(true);
            camera_control = true;
        } else {
            renderer.lock_mouse(false);
            camera_control = false;
        }

        if camera_control {
            viewport.has_camera_control = true;
        } else {
            viewport.has_camera_control = false;
        }

        viewport.handle_input(&mut renderer, delta);

        worldmachine.handle_audio(&renderer, &audio, &scontext);
        worldmachine.render(&mut renderer, None);
        renderer.clear_all_shadow_buffers();
        let light_count = renderer.lights.len();
        for i in 0..light_count {
            worldmachine.render(&mut renderer, Some((1, i)));
            worldmachine.render(&mut renderer, Some((2, i)));
            renderer.next_light();
        }

        renderer.swap_buffers(&mut worldmachine);
        renderer.backend.window.lock().unwrap().glfw.poll_events();
        keyboard::reset_keyboard_state();
        mouse::reset_mouse_state();
        for (_, event) in glfw::flush_messages(renderer.backend.events.lock().unwrap().deref()) {
            egui_glfw_gl::handle_event(event.clone(), &mut renderer.backend.input_state.lock().unwrap());
            keyboard::tick_keyboard(event.clone());
            mouse::tick_mouse(event);
        }
        if renderer.manage_window() || crate::ui::WANT_QUIT.load(Ordering::Relaxed) {
            return;
        }
    }
}