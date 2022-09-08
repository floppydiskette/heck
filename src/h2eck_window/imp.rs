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
use gtk::glib::{Type, Value};
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
    pub editor: Arc<Mutex<Option<Editor>>>,

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
        let mut editor_obj = self.editor.lock().unwrap();
        *editor_obj = Some(Editor::new());
        let editor_obj = editor_obj.as_ref().unwrap();

        *editor_obj.imp().worldmachine.clone().lock().unwrap() = Some(self.worldmachine.clone());

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
        let worldmachine = obj.clone().imp().worldmachine.clone();
        let editor = self.editor.clone();
        editor_obj.imp().main_view.connect_realize( move |a| {
            let mut inner_worldmachine = worldmachine.lock().unwrap();
            inner_worldmachine.initialise(editor.clone());
            debug!("initialised worldmachine");
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
        let editor = self.editor.clone();
        gesture.connect_drag_update(move |a, mouse_x, mouse_y| {
            let mut inner_renderer = renderer.lock().unwrap();
            let mut inner_editor = editor.lock().unwrap();
            let inner_editor = inner_editor.as_ref().unwrap();
            inner_editor.imp().main_view.grab_focus();
            inner_renderer.rotate_camera(mouse_x as f32, mouse_y as f32);
        });
        let renderer = obj.clone().imp().renderer.clone();
        let editor = self.editor.clone();
        gesture.connect_drag_begin(move |a, mouse_x, mouse_y| {
            let mut inner_renderer = renderer.lock().unwrap();
            let mut inner_editor = editor.lock().unwrap();
            let inner_editor = inner_editor.as_ref().unwrap();
            inner_editor.imp().main_view.grab_focus();
            inner_renderer.start_rotate_camera(mouse_x as f32, mouse_y as f32);
        });
        let renderer = obj.clone().imp().renderer.clone();
        gesture.connect_drag_end(move |a, mouse_x, mouse_y| {
            let mut inner_renderer = renderer.lock().unwrap();
            inner_renderer.end_rotate_camera(mouse_x as f32, mouse_y as f32);
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

        // create a gesture to deselect everything when clicking on the glarea
        let gesture = gtk::GestureClick::new();
        gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
        gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);
        let editor = self.editor.clone();
        gesture.connect_released(move |a, mouse_x, mouse_y, state| {
            let mut inner_editor = editor.lock().unwrap();
            let inner_editor = inner_editor.as_ref().unwrap();
            inner_editor.imp().main_view.grab_focus();
        });
        editor_obj.imp().main_view.add_controller(&gesture);

        editor_obj.show();

        self.stack.add_child(editor_obj);
        self.stack.set_visible_child(editor_obj);
    }
}

impl WidgetImpl for h2eckWindow {}
impl WindowImpl for h2eckWindow {}
impl ApplicationWindowImpl for h2eckWindow {}