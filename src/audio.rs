use rand::seq::SliceRandom;
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use std::time::Duration;

pub fn play_music(mut paths: Vec<String>, wait: u64) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut rng = rand::thread_rng();
    let mut first_audio = true;
    loop {
        for audio_dir in &paths {
            let audio_buf = std::fs::File::open(&audio_dir).unwrap();
            let file = std::io::BufReader::new(audio_buf);
            let source = rodio::Decoder::new(file).unwrap();
            if first_audio {
                sink.append(source.fade_in(Duration::from_secs(8)));
                first_audio = false;
            } else {
                sink.append(source);
            }
        }
        paths.shuffle(&mut rng);
        std::thread::sleep(std::time::Duration::from_secs(wait));
    }
}
