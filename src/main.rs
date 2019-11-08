use rodio::Sink;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Write;
use std::{thread, time};
use termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    // prepare terminal
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = termion::async_stdin().keys();

    write!(stdout, "scli ☁️  🦀 ☁️ \r\n").unwrap();

    // read file name
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    // initialize audio device and load file
    let device = rodio::default_output_device().unwrap();
    write!(stdout, "using device: {}\r\n", device.name()).unwrap();
    write!(stdout, "file: {}\r\n", filename).unwrap();

    stdout.lock().flush().unwrap();

    let file = File::open(filename).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    // event loop
    let player = thread::spawn(move || {
        // start the audio
        let sink = Sink::new(&device);
        sink.append(source);

        // listen for user input
        loop {
            let input = stdin.next();
            if let Some(Ok(key)) = input {
                match key {
                    Key::Char('q') => break,
                    Key::Char(' ') => {
                        if sink.is_paused() {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                    },
                    _ => { continue }
                }
            }

            thread::sleep(time::Duration::from_millis(50));
        }
    });

    player.join().unwrap();
}
