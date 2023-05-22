use rand::seq::SliceRandom;
use rodio::source::Source;
use rodio::OutputStream;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Mutex;
use tokio::time::Duration;

pub static AUDIO_PLAYER: Mutex<Option<AudioStream>> = Mutex::new(None);
static CURRENT_STREAM: AtomicUsize = AtomicUsize::new(0);
static IS_FIRST_AUDIO: AtomicBool = AtomicBool::new(true);

pub struct AudioStream {
    pub sink: crate::sink::Sink,
    pub stream: OutputStream,
    pub audio_paths: Vec<String>,
}
unsafe impl Send for AudioStream {}
unsafe impl Sync for AudioStream {}

impl std::fmt::Debug for AudioStream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Display trait is not implemented yet.")
    }
}

pub fn play_music() {
    let mut paths = AUDIO_PLAYER
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .audio_paths
        .clone();
    loop {
        for audio_dir in &paths {
            let audio_buf = std::fs::File::open(audio_dir).unwrap();
            let file = std::io::BufReader::new(audio_buf);
            let source = rodio::Decoder::new(file).unwrap();
            if IS_FIRST_AUDIO.load(Ordering::Relaxed) {
                IS_FIRST_AUDIO.store(false, Ordering::Relaxed);
                AUDIO_PLAYER
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .sink
                    .append(source.fade_in(Duration::from_secs(8)));
            } else {
                AUDIO_PLAYER
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .sink
                    .append(source);
            }
            let sleep_mutex = crate::sink::SLEEP_UNTIL_END.lock().unwrap();
            if let Some(sleep_until_end) = sleep_mutex.as_ref() {
                let _ = sleep_until_end.recv();
            }
        }
        let mut rng = rand::thread_rng();
        paths.shuffle(&mut rng);
    }
}
