use resonance::prelude::*;

fn main() {
    Engine::new()
        .add_plugin(CorePlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(WindowPlugin::default())
        .run();
}
