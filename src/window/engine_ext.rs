
use crate::app::Engine;

pub trait EngineExt {
    fn run(self);
}

impl EngineExt for Engine {
    fn run(self) {
        crate::window::runner::run(self);
    }
}
