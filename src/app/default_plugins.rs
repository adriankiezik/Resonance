use super::{Plugin, Resonance};

#[derive(Default)]
pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(&self, engine: &mut Resonance) {
        let engine_with_defaults = std::mem::take(engine)
            .add_plugin(crate::app::CorePlugin::default())
            .add_plugin(crate::transform::TransformPlugin::default())
            .add_plugin(crate::assets::AssetsPlugin::default())
            .add_plugin(crate::window::WindowPlugin::default())
            .add_plugin(crate::renderer::RenderPlugin::default())
            .add_plugin(crate::input::InputPlugin::default())
            .add_plugin(crate::audio::AudioPlugin::default())
            .add_plugin(crate::core::PerformancePlugin::default());

        *engine = engine_with_defaults;
    }
}
