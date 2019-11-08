use rodio::Sink;
use rodio::Source;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::{thread, time};

fn main() {
    println!("scli ☁️  🦀 ☁️ ");

    // read file name
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    // initialize audio device and load file
    let device = rodio::default_output_device().unwrap();
    println!("using device: {}", device.name());
    println!("file: {}", filename);

    let file = File::open(filename).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    let player = thread::spawn(move || {
        let sink = Sink::new(&device);
        sink.append(source);

        thread::sleep(time::Duration::from_secs(10));
    });

    player.join().unwrap();
}
