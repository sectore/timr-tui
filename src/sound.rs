use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, error};

#[derive(Debug, Error)]
pub enum SoundError {
    #[error("Output stream error: {0}")]
    OutputStream(String),
    #[error("File error: {0}")]
    File(std::io::Error),
    #[error("Sink error: {0}")]
    Sink(String),
    #[error("Decoder error: {0}")]
    Decoder(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

// #[derive(Clone)]
pub struct Sound {
    path: String,
}

impl Sound {
    pub fn new(path: &str) -> Result<Self, SoundError> {
        // Validate file extension
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .filter(|ext| ["mp3", "wav"].contains(&ext.to_lowercase().as_str()))
            .ok_or_else(|| {
                let err = SoundError::UnsupportedFormat(
                    "Unsupported file format. Only MP3 and WAV are supported".to_owned(),
                );
                error!(%err);
                err
            })?;

        Ok(Self {
            path: path.to_string(),
        })
    }

    pub fn play(&self) -> Result<(), SoundError> {
        let path = self.path.clone();

        debug!("Sound::play before thread");
        std::thread::spawn(move || -> Result<(), SoundError> {
            debug!("Sound::play thread {:?} ", &path);
            // Important note: Never (ever) use a single `_` as a placeholder here. `_stream` or something is fine!
            // The value will dropped and the sound will fail without any errors
            // see https://github.com/RustAudio/rodio/issues/330
            let (_stream, handle) = OutputStream::try_default().map_err(|e| {
                let err = SoundError::OutputStream(e.to_string());
                error!(%err);
                err
            })?;
            let file = File::open(&path).map_err(SoundError::File)?;

            let sink = Sink::try_new(&handle).map_err(|e| {
                let err = SoundError::Sink(e.to_string());
                error!(%err);
                err
            })?;
            let decoder = Decoder::new(BufReader::new(file)).map_err(|e| {
                let err = SoundError::Decoder(e.to_string());
                error!(%err);
                err
            })?;
            sink.append(decoder);
            sink.sleep_until_end();

            Ok(())
        });

        Ok(())
    }
}
