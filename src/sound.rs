use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Source, source::Buffered};
use std::borrow::Cow;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

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
    stream: MixerDeviceSink,
}

impl Sound {
    /// Uses `DEFAULT_SOUND` if no `path` is given
    pub fn new(path: Option<PathBuf>) -> Result<Self, SoundError> {
        let stream = DeviceSinkBuilder::open_default_sink()
            .map_err(|e: rodio::DeviceSinkError| SoundError::OutputStream(e.to_string()))?;

        let data = match path {
            Some(path) => Cow::Owned(fs::read(path).map_err(|e| SoundError::File(e.to_string()))?),
            None => Cow::Borrowed(DEFAULT_SOUND),
        };
        let decoder =
            Decoder::try_from(Cursor::new(data)).map_err(|e| SoundError::Decoder(e.to_string()))?;
        let buffer = Arc::new(decoder.buffered());

        Ok(Self { buffer, stream })
    }

    pub fn play(&self) -> Result<(), SoundError> {
        self.stream.mixer().add((*self.buffer).clone());
        Ok(())
    }
}
