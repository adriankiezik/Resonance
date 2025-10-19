use super::{Engine, Plugin};

#[derive(Default)]
pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(&self, engine: &mut Engine) {
        let mut eng = std::mem::take(engine)
            .add_plugin(crate::app::CorePlugin::default())
            .add_plugin(crate::transform::TransformPlugin::default())
            .add_plugin(crate::assets::AssetsPlugin::default());

        #[cfg(feature = "window")]
        {
            eng = eng.add_plugin(crate::window::WindowPlugin::default());
        }

        #[cfg(feature = "renderer")]
        {
            eng = eng.add_plugin(crate::renderer::RenderPlugin::default());
        }

        #[cfg(feature = "audio")]
        {
            eng = eng.add_plugin(crate::audio::AudioPlugin::default());
        }

        *engine = eng;
    }
}
