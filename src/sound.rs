use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source, source::Buffered};
use std::borrow::Cow;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use thiserror::Error;
use tracing::error;

const DEFAULT_SOUND: &[u8] = include_bytes!("../assets/confirmation-004.mp3");

#[derive(Debug, Error)]
pub enum SoundError {
    #[error("Sound output stream error: {0}")]
    OutputStream(String),
    #[error("Sound file error: {0}")]
    File(String),
    #[error("Sound decoder error: {0}")]
    Decoder(String),
}

pub fn validate_sound_file(path: &PathBuf) -> Result<&PathBuf, SoundError> {
    // validate path
    if !path.exists() {
        let err = SoundError::File(format!("File not found: {:?}", path));
        return Err(err);
    };

    // Validate file extension
    path.extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| ["mp3", "wav"].contains(&ext.to_lowercase().as_str()))
        .ok_or_else(|| {
            SoundError::File(
                "Unsupported file extension. Only .mp3 and .wav are supported".to_owned(),
            )
        })?;

    Ok(path)
}

pub struct Sound {
    buffer: Arc<Buffered<Decoder<Cursor<Cow<'static, [u8]>>>>>,
}

impl Sound {
    /// Uses `DEFAULT_SOUND` if no `path` is given
    pub fn new(path: Option<PathBuf>) -> Result<Self, SoundError> {
        let data = match path {
            Some(path) => Cow::Owned(fs::read(path).map_err(|e| SoundError::File(e.to_string()))?),
            None => Cow::Borrowed(DEFAULT_SOUND),
        };
        let decoder =
            Decoder::try_from(Cursor::new(data)).map_err(|e| SoundError::Decoder(e.to_string()))?;
        let buffer = Arc::new(decoder.buffered());

        Ok(Self { buffer })
    }

    /// Opens the output device on demand and releases it after playing.
    /// A long-lived stream breaks after suspend/resume and floods stderr with errors.
    pub fn play(&self) {
        let source = (*self.buffer).clone();
        thread::spawn(move || match open_sink() {
            Ok(sink) => {
                let player = Player::connect_new(sink.mixer());
                player.append(source);
                player.sleep_until_end();
            }
            Err(err) => error!("{err}"),
        });
    }
}

fn open_sink() -> Result<MixerDeviceSink, SoundError> {
    let mut sink = DeviceSinkBuilder::from_default_device()
        .map_err(|e| SoundError::OutputStream(e.to_string()))?
        .with_error_callback(|_| {})
        .open_sink_or_fallback()
        .map_err(|e| SoundError::OutputStream(e.to_string()))?;
    sink.log_on_drop(false);
    Ok(sink)
}
