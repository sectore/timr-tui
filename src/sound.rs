use rodio::{Decoder, OutputStream, OutputStreamBuilder, Source, source::Buffered};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

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
    buffer: Arc<Buffered<Decoder<BufReader<File>>>>,
    stream: OutputStream,
}

impl Sound {
    pub fn new(path: PathBuf) -> Result<Self, SoundError> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|e| SoundError::OutputStream(e.to_string()))?;

        let file = File::open(&path).map_err(|e| SoundError::File(e.to_string()))?;
        let decoder = Decoder::try_from(file).map_err(|e| SoundError::Decoder(e.to_string()))?;
        let buffer = Arc::new(decoder.buffered());

        Ok(Self { buffer, stream })
    }

    pub fn play(&self) -> Result<(), SoundError> {
        self.stream.mixer().add((*self.buffer).clone());
        Ok(())
    }
}
