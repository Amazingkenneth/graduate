use lofty::AudioFile;
use rand::seq::SliceRandom;
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use std::mem::ManuallyDrop;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

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
    stream: Arc<std::sync::Mutex<ManuallyDrop<AudioStream>>>,
    mut paths: Vec<String>,
    running_status: Arc<AtomicBool>,
) {
    let mut is_first_audio = true;
    loop {
        for audio_dir in &paths {
            let audio_buf = std::fs::File::open(&audio_dir).unwrap();
            let cur_duration = {
                let mut another_buf = std::fs::File::open(&audio_dir).unwrap();
                // println!("audio_dir: {}", audio_dir);
                let tagged_file =
                    lofty::TaggedFile::read_from(&mut another_buf, lofty::ParseOptions::new())
                        .unwrap();
                tagged_file.properties().duration()
            };
            let file = std::io::BufReader::new(audio_buf);
            let source = rodio::Decoder::new(file).unwrap();
            if is_first_audio {
                is_first_audio = false;
                let sink = &stream.lock().unwrap().sink;
                sink.append(source.fade_in(Duration::from_secs(8)));
            } else {
                let sink = &stream.lock().unwrap().sink;
                sink.append(source);
            }
            sleep(cur_duration).await;
            while stream.lock().unwrap().sink.len() > 1 && !stream.lock().unwrap().sink.is_paused()
            {
                sleep(cur_duration).await;
            }
            if stream.lock().unwrap().sink.is_paused() {
                drop(stream.lock().unwrap());
                running_status.store(false, Ordering::Relaxed);
                return;
            }
        }
        let mut rng = rand::thread_rng();
        paths.shuffle(&mut rng);
    }
}
