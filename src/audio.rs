use rand::seq::SliceRandom;
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};

pub fn play_music(mut paths: Vec<String>, wait: u64) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut rng = rand::thread_rng();
    loop {
        for audio_dir in &paths {
            println!("audiodir = {}", audio_dir);
            let audio_buf = std::fs::File::open(&audio_dir).unwrap();
            let file = std::io::BufReader::new(audio_buf);
            let source = rodio::Decoder::new(file).unwrap();
            sink.append(source);
        }
        paths.shuffle(&mut rng);
        std::thread::sleep(std::time::Duration::from_secs(wait));
    }
}
