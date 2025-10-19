use crate::app::Resonance;

pub trait ResonanceExt {
    fn run(self);
}

impl ResonanceExt for Resonance {
    fn run(self) {
        crate::window::runner::run(self);
    }
}
