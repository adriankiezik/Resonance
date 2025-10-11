
use resonance_app::Engine;

pub trait EngineExt {

    fn run(self);
}

impl EngineExt for Engine {
    fn run(self) {
        crate::runner::run(self);
    }
}