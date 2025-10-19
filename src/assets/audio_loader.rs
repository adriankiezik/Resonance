
use crate::assets::loader::{AssetLoader, LoadError};
use std::io::Cursor;
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Clone, Debug)]
pub struct AudioData {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub duration: f32,
}

impl AudioData {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
            samples: Vec::new(),
            duration: 0.0,
        }
    }

    pub fn fallback_silence() -> Self {
        let sample_rate = 44100;
        let channels = 2;
        let duration = 1.0;
        let sample_count = (sample_rate as f32 * duration * channels as f32) as usize;

        Self {
            sample_rate,
            channels,
            samples: vec![0.0; sample_count],
            duration,
        }
    }

    pub fn test_tone(frequency: f32, duration: f32) -> Self {
        let sample_rate = 44100;
        let channels = 2;
        let sample_count = (sample_rate as f32 * duration) as usize;
        let mut samples = Vec::with_capacity(sample_count * channels as usize);

        for i in 0..sample_count {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;

            samples.push(sample);
            samples.push(sample);
        }

        Self {
            sample_rate,
            channels,
            samples,
            duration,
        }
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels as usize
    }
}

pub struct AudioLoader;

impl AssetLoader for AudioLoader {
    type Asset = AudioData;

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {

        let file = std::fs::File::open(path)
            .map_err(|e| LoadError::LoadFailed(format!("Failed to open audio file: {}", e)))?;

        load_audio_from_reader(file, path)
    }

    fn extensions(&self) -> &[&str] {
        &["wav", "mp3", "ogg", "flac", "m4a", "aac"]
    }
}

fn load_audio_from_reader(
    reader: impl std::io::Read + std::io::Seek + Send + Sync + 'static,
    path: &Path,
) -> Result<AudioData, LoadError> {

    let mss = MediaSourceStream::new(Box::new(ReadOnlySource::new(reader)), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension() {
        hint.with_extension(&ext.to_string_lossy());
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| LoadError::LoadFailed(format!("Failed to probe audio format: {}", e)))?;

    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or_else(|| LoadError::LoadFailed("No audio tracks found".into()))?;

    let track_id = track.id;
    let codec_params = &track.codec_params;

    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| LoadError::LoadFailed("Unknown sample rate".into()))?;

    let channels = codec_params
        .channels
        .ok_or_else(|| LoadError::LoadFailed("Unknown channel count".into()))?
        .count() as u16;

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default())
        .map_err(|e| LoadError::LoadFailed(format!("Failed to create decoder: {}", e)))?;

    let mut samples = Vec::new();

    loop {

        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => {
                return Err(LoadError::LoadFailed(format!(
                    "Failed to read packet: {}",
                    e
                )))
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = decoder
            .decode(&packet)
            .map_err(|e| LoadError::LoadFailed(format!("Failed to decode packet: {}", e)))?;

        convert_audio_buffer(&decoded, &mut samples);
    }

    let duration = samples.len() as f32 / (sample_rate * channels as u32) as f32;

    log::info!(
        "Loaded audio: {}Hz, {} channels, {:.2}s ({} samples)",
        sample_rate,
        channels,
        duration,
        samples.len()
    );

    Ok(AudioData {
        sample_rate,
        channels,
        samples,
        duration,
    })
}

fn convert_audio_buffer(buffer: &AudioBufferRef, output: &mut Vec<f32>) {
    let num_channels = match buffer {
        AudioBufferRef::F32(buf) => buf.spec().channels.count(),
        AudioBufferRef::S16(buf) => buf.spec().channels.count(),
        AudioBufferRef::S32(buf) => buf.spec().channels.count(),
        AudioBufferRef::U8(buf) => buf.spec().channels.count(),
        AudioBufferRef::U16(buf) => buf.spec().channels.count(),
        AudioBufferRef::U32(buf) => buf.spec().channels.count(),
        _ => {
            log::warn!("Unsupported audio buffer format");
            return;
        }
    };

    let num_frames = match buffer {
        AudioBufferRef::F32(buf) => buf.frames(),
        AudioBufferRef::S16(buf) => buf.frames(),
        AudioBufferRef::S32(buf) => buf.frames(),
        AudioBufferRef::U8(buf) => buf.frames(),
        AudioBufferRef::U16(buf) => buf.frames(),
        AudioBufferRef::U32(buf) => buf.frames(),
        _ => return,
    };

    match buffer {
        AudioBufferRef::F32(buf) => {
            for frame in 0..num_frames {
                for channel in 0..num_channels {
                    output.push(buf.chan(channel)[frame]);
                }
            }
        }
        AudioBufferRef::S16(buf) => {
            for frame in 0..num_frames {
                for channel in 0..num_channels {
                    let sample = buf.chan(channel)[frame];
                    output.push(sample as f32 / i16::MAX as f32);
                }
            }
        }
        AudioBufferRef::S32(buf) => {
            for frame in 0..num_frames {
                for channel in 0..num_channels {
                    let sample = buf.chan(channel)[frame];
                    output.push(sample as f32 / i32::MAX as f32);
                }
            }
        }
        AudioBufferRef::U8(buf) => {
            for frame in 0..num_frames {
                for channel in 0..num_channels {
                    let sample = buf.chan(channel)[frame];
                    output.push((sample as f32 - 128.0) / 128.0);
                }
            }
        }
        AudioBufferRef::U16(buf) => {
            for frame in 0..num_frames {
                for channel in 0..num_channels {
                    let sample = buf.chan(channel)[frame];
                    output.push((sample as f32 - 32768.0) / 32768.0);
                }
            }
        }
        AudioBufferRef::U32(buf) => {
            for frame in 0..num_frames {
                for channel in 0..num_channels {
                    let sample = buf.chan(channel)[frame];
                    output.push((sample as f32 - 2147483648.0) / 2147483648.0);
                }
            }
        }
        _ => {}
    }
}

pub fn load_audio_from_bytes(bytes: Vec<u8>) -> Result<AudioData, LoadError> {
    let cursor = Cursor::new(bytes);
    load_audio_from_reader(cursor, Path::new("memory"))
}
