use libsex::bindings::*;

pub struct H2eckRenderer {
    pub state: H2eckState,
}

pub enum H2eckState {
    Welcome,
}

impl Default for H2eckRenderer {
    fn default() -> Self {
        Self {
            state: H2eckState::Welcome,
        }
    }
}

impl H2eckRenderer {
    // should be called upon the render action of our GtkGLArea
    pub fn render(&self) {
        unsafe {
            print!("rendering");
            // set the clear color to black
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }
    }
}