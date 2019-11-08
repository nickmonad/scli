use std::{thread, time};
use std::fs::File;
use std::io::BufReader;
use rodio::Source;

fn main() {
    println!("-- scli --");

    let device = rodio::default_output_device().unwrap();
    println!("using device: {}", device.name());

    let file = File::open("sunsets-in-space.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    rodio::play_raw(&device, source.convert_samples());

    thread::sleep(time::Duration::from_secs(10));
}
