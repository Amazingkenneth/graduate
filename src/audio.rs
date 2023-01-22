use rand::seq::SliceRandom;
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct Audios {
    pub sink: Arc<std::sync::Mutex<ManuallyDrop<AudioStream>>>,
    pub volume: f32,
    pub on_play: bool,
}

pub struct AudioStream {
    pub sink: rodio::Sink,
    pub stream: OutputStream,
}
unsafe impl Send for AudioStream {}

impl std::fmt::Debug for AudioStream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Display trait is not implemented yet.")
    }
}

pub async fn play_music(
    sink: Arc<std::sync::Mutex<ManuallyDrop<AudioStream>>>,
    mut paths: Vec<String>,
) {
    let mut is_first_audio = true;
    loop {
        let sink = &sink.lock().unwrap().sink;
        for audio_dir in &paths {
            let audio_buf = std::fs::File::open(&audio_dir).unwrap();
            let file = std::io::BufReader::new(audio_buf);
            let source = rodio::Decoder::new(file).unwrap();
            if is_first_audio {
                sink.append(source.fade_in(Duration::from_secs(8)));
                is_first_audio = false;
            } else {
                sink.append(source);
            }
        }
        sink.sleep_until_end();
        let mut rng = rand::thread_rng();
        paths.shuffle(&mut rng);
    }
}
