use rodio::{Decoder, OutputStream, Sink};
use std::fs;

pub struct Sound {
    sound_data: Vec<u8>,
}

impl Sound {
    pub fn new(path: &str) -> Result<Self, String> {
        let sound_data = fs::read(path).map_err(|e| format!("Failed to read sound file: {}", e))?;
        Ok(Self { sound_data })
    }

    pub fn play(&self) -> Result<(), String> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to get audio output: {}", e))?;

        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Failed to create audio sink: {}", e))?;

        let cursor = std::io::Cursor::new(self.sound_data.clone());
        let source = Decoder::new(cursor).map_err(|e| format!("Failed to decode audio: {}", e))?;

        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }
}
