use rand::seq::SliceRandom;
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use tokio::time::*;

#[derive(Clone)]
pub struct Audios {
    pub sink: Arc<std::sync::Mutex<ManuallyDrop<AudioStream>>>,
    pub volume: f32,
}

pub struct AudioStream {
    pub sink: rodio::Sink,
    pub stream: OutputStream,
}
unsafe impl Send for AudioStream {}

impl std::fmt::Debug for Audios {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Display trait is not implemented yet.")
    }
}

pub async fn play_music(
    sink: Arc<std::sync::Mutex<ManuallyDrop<AudioStream>>>,
    mut paths: Vec<String>,
    wait: u64,
) {
    let mut is_first_audio = true;
    loop {
        for audio_dir in &paths {
            let audio_buf = std::fs::File::open(&audio_dir).unwrap();
            let file = std::io::BufReader::new(audio_buf);
            let source = rodio::Decoder::new(file).unwrap();
            let sink = &sink.lock().unwrap().sink;
            if is_first_audio {
                sink.append(source.fade_in(Duration::from_secs(8)));
                is_first_audio = false;
            } else {
                sink.append(source);
            }
        }
        sleep(Duration::from_secs(wait)).await;
        let mut rng = rand::thread_rng();
        paths.shuffle(&mut rng);
    }
}
