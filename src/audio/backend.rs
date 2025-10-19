use bevy_ecs::prelude::*;
use rodio::{OutputStream, Sink, Source, SpatialSink};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub enum AudioSinkType {
    Regular(Sink),
    Spatial(SpatialSink),
}

impl AudioSinkType {
    pub fn set_volume(&self, volume: f32) {
        match self {
            AudioSinkType::Regular(sink) => sink.set_volume(volume),
            AudioSinkType::Spatial(sink) => sink.set_volume(volume),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            AudioSinkType::Regular(sink) => sink.empty(),
            AudioSinkType::Spatial(sink) => sink.empty(),
        }
    }

    pub fn pause(&self) {
        match self {
            AudioSinkType::Regular(sink) => sink.pause(),
            AudioSinkType::Spatial(sink) => sink.pause(),
        }
    }

    pub fn play(&self) {
        match self {
            AudioSinkType::Regular(sink) => sink.play(),
            AudioSinkType::Spatial(sink) => sink.play(),
        }
    }

    pub fn stop(&self) {
        match self {
            AudioSinkType::Regular(sink) => sink.stop(),
            AudioSinkType::Spatial(sink) => sink.stop(),
        }
    }

    pub fn append<S>(&self, source: S)
    where
        S: Source<Item = f32> + Send + 'static,
    {
        match self {
            AudioSinkType::Regular(sink) => sink.append(source),
            AudioSinkType::Spatial(sink) => sink.append(source),
        }
    }
}

struct SendOutputStream(Arc<OutputStream>);

unsafe impl Send for SendOutputStream {}
unsafe impl Sync for SendOutputStream {}

#[derive(Resource)]
pub struct AudioBackend {
    stream: SendOutputStream,

    sinks: Arc<Mutex<HashMap<Entity, AudioSinkType>>>,
}

impl AudioBackend {
    pub fn new() -> Result<Self, String> {
        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .map_err(|e| format!("Failed to initialize audio output: {}", e))?;

        Ok(Self {
            stream: SendOutputStream(Arc::new(stream)),
            sinks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn create_sink(&self, entity: Entity) -> Result<(), String> {
        let sink = rodio::Sink::connect_new(self.stream.0.mixer());
        let mut sinks = self.sinks.lock().unwrap();
        sinks.insert(entity, AudioSinkType::Regular(sink));
        Ok(())
    }

    pub fn create_spatial_sink(&self, entity: Entity, emitter_pos: [f32; 3]) -> Result<(), String> {
        let left_ear = [-1.0, 0.0, 0.0];
        let right_ear = [1.0, 0.0, 0.0];

        let sink = rodio::SpatialSink::connect_new(
            self.stream.0.mixer(),
            emitter_pos,
            left_ear,
            right_ear,
        );

        let mut sinks = self.sinks.lock().unwrap();
        sinks.insert(entity, AudioSinkType::Spatial(sink));
        Ok(())
    }

    pub fn set_emitter_position(&self, entity: Entity, position: [f32; 3]) {
        let sinks = self.sinks.lock().unwrap();
        if let Some(AudioSinkType::Spatial(sink)) = sinks.get(&entity) {
            sink.set_emitter_position(position);
        }
    }

    pub fn has_sink(&self, entity: Entity) -> bool {
        let sinks = self.sinks.lock().unwrap();
        sinks.contains_key(&entity)
    }

    pub fn remove_sink(&self, entity: Entity) {
        let mut sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.remove(&entity) {
            sink.stop();
        }
    }

    pub fn play_audio<S>(&self, entity: Entity, source: S, volume: f32) -> Result<(), String>
    where
        S: Source<Item = f32> + Send + 'static,
    {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&entity) {
            sink.set_volume(volume);
            sink.append(source);
            Ok(())
        } else {
            Err(format!("No sink found for entity {:?}", entity))
        }
    }

    pub fn set_volume(&self, entity: Entity, volume: f32) {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&entity) {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    pub fn is_playing(&self, entity: Entity) -> bool {
        let sinks = self.sinks.lock().unwrap();
        sinks
            .get(&entity)
            .map(|sink| !sink.is_empty())
            .unwrap_or(false)
    }

    pub fn pause(&self, entity: Entity) {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&entity) {
            sink.pause();
        }
    }

    pub fn resume(&self, entity: Entity) {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&entity) {
            sink.play();
        }
    }

    pub fn stop(&self, entity: Entity) {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&entity) {
            sink.stop();
        }
    }

    pub fn active_count(&self) -> usize {
        let sinks = self.sinks.lock().unwrap();
        sinks.len()
    }

    pub fn cleanup_finished(&self) {
        let mut sinks = self.sinks.lock().unwrap();
        sinks.retain(|_, sink| !sink.is_empty());
    }
}

impl Default for AudioBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create default audio backend")
    }
}

pub struct MemorySource {
    data: Arc<Vec<f32>>,
    position: usize,
    sample_rate: u32,
    channels: u16,
}

impl MemorySource {
    pub fn new(data: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            data: Arc::new(data),
            position: 0,
            sample_rate,
            channels,
        }
    }
}

impl Iterator for MemorySource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.data.len() {
            let sample = self.data[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for MemorySource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        let total_samples = self.data.len() as u64;
        let frames = total_samples / self.channels as u64;
        let duration_secs = frames as f64 / self.sample_rate as f64;
        Some(std::time::Duration::from_secs_f64(duration_secs))
    }
}
