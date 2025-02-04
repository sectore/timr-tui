use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SoundError {
    #[error("Sound output stream error: {0}")]
    OutputStream(String),
    #[error("Sound file error: {0}")]
    File(String),
    #[error("Sound sink error: {0}")]
    Sink(String),
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

// #[derive(Clone)]
pub struct Sound {
    path: PathBuf,
}

impl Sound {
    pub fn new(path: PathBuf) -> Result<Self, SoundError> {
        Ok(Self { path })
    }

    pub fn play(&self) -> Result<(), SoundError> {
        // validate file again
        validate_sound_file(&self.path)?;
        // before playing the sound
        let path = self.path.clone();

        std::thread::spawn(move || -> Result<(), SoundError> {
            // Important note: Never (ever) use a single `_` as a placeholder here. `_stream` or something is fine!
            // The value will dropped and the sound will fail without any errors
            // see https://github.com/RustAudio/rodio/issues/330
            let (_stream, handle) =
                OutputStream::try_default().map_err(|e| SoundError::OutputStream(e.to_string()))?;
            let file = File::open(&path).map_err(|e| SoundError::File(e.to_string()))?;
            let sink = Sink::try_new(&handle).map_err(|e| SoundError::Sink(e.to_string()))?;
            let decoder = Decoder::new(BufReader::new(file))
                .map_err(|e| SoundError::Decoder(e.to_string()))?;
            sink.append(decoder);
            sink.sleep_until_end();

            Ok(())
        });

        Ok(())
    }
}
