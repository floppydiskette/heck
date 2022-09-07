use std::ffi::CString;
use std::sync::{Arc, Mutex};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, PopoverMenuBar, GLArea, Inhibit};
use gtk::ffi::*;
use gtk::gdk::ffi::GdkFrameClock;
use gtk::gio::Menu;
use gtk::glib::translate::ToGlibPtr;
use gtk::glib::Type;
use crate::gio::glib::clone;
use crate::gio::SimpleAction;
use crate::h2eck_window::editor::Editor;
use crate::renderer::H2eckRenderer;
use crate::worldmachine::WorldMachine;


#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/realmicrosoft/h2eck/window.ui")]
pub struct h2eckWindow {
    pub renderer: Arc<Mutex<H2eckRenderer>>,
    pub worldmachine: Arc<Mutex<WorldMachine>>,
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
}

#[glib::object_subclass]
impl ObjectSubclass for h2eckWindow {
    const NAME: &'static str = "h2eckWindow";
    type Type = super::h2eckWindow;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for h2eckWindow {
    fn constructed(&self, obj: &Self::Type) {
        // call "constructed" on parent
        self.parent_constructed(obj);
        self.setup(obj);
    }
}

impl h2eckWindow {
    pub fn setup(&self, obj: &<h2eckWindow as ObjectSubclass>::Type) {
        let editor_obj = Editor::new();

        let renderer = obj.clone().imp().renderer.clone();
        let worldmachine = obj.clone().imp().worldmachine.clone();


        // connect render signal
        editor_obj.imp().main_view.connect_render(move |a, b| {
            let mut inner_renderer = renderer.lock().unwrap();
            let mut inner_worldmachine = worldmachine.lock().unwrap();
            inner_renderer.render(&mut inner_worldmachine);
            Inhibit(false)
        });

        // tell glib to redraw the editor every frame
        editor_obj.imp().main_view.connect_realize(move |a| {
            let frame_clock = a.frame_clock().unwrap();
            frame_clock.connect_update(clone!(@weak a => move |_| {
                a.queue_draw();
            }));
            frame_clock.begin_updating();
        });

        // create a gesture for controlling the camera
        let gesture = gtk::GestureDrag::new();
        gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
        let renderer = obj.clone().imp().renderer.clone();
        gesture.connect_drag_update(move |a, mouse_x, mouse_y| {
            let mut inner_renderer = renderer.lock().unwrap();
            inner_renderer.rotate_camera(mouse_x as f32, mouse_y as f32);
        });
        let renderer = obj.clone().imp().renderer.clone();
        gesture.connect_drag_begin(move |a, mouse_x, mouse_y| {
            let mut inner_renderer = renderer.lock().unwrap();
            inner_renderer.start_rotate_camera(mouse_x as f32, mouse_y as f32);
        });
        let renderer = obj.clone().imp().renderer.clone();
        gesture.connect_drag_end(move |a, mouse_x, mouse_y| {
            let mut inner_renderer = renderer.lock().unwrap();
            inner_renderer.end_rotate_camera();
        });
        gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
        editor_obj.imp().main_view.add_controller(&gesture);

        // create an eventcontroller for keyboard input
        let event_controller = gtk::EventControllerKey::new();
        let renderer = obj.clone().imp().renderer.clone();
        event_controller.connect_key_pressed(move |a, keyval, keycode, state| {
            let mut inner_renderer = renderer.lock().unwrap();
            inner_renderer.process_key(keyval, true);
            Inhibit(false)
        });
        let renderer = obj.clone().imp().renderer.clone();
        event_controller.connect_key_released(move |a, keyval, keycode, state| {
            let mut inner_renderer = renderer.lock().unwrap();
            inner_renderer.process_key(keyval, false);
        });
        obj.add_controller(&event_controller);

        editor_obj.show();

        self.stack.add_child(&editor_obj);
        self.stack.set_visible_child(&editor_obj);
    }
}

impl WidgetImpl for h2eckWindow {}
impl WindowImpl for h2eckWindow {}
impl ApplicationWindowImpl for h2eckWindow {}