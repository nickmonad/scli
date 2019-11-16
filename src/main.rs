#[macro_use]
extern crate serde;

use reqwest;
use reqwest::header;
use rodio::Sink;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Write;
use std::sync::mpsc;
use std::{thread, time};
use termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

enum UserInput {
    PlayPause,
    Quit,
}

#[derive(Deserialize, Debug)]
struct ResolvedResource {
    status: String,
    location: String,
}

fn main() {
    // prepare terminal
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let stdin = termion::async_stdin().keys();

    // read file name
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    // build a channel for user -> player
    let (tx, rx) = mpsc::channel();

    // TESTING, get resolved track
    let location = resolve("https://soundcloud.com/nickmonad/sunsets-in-space".to_string())
        .expect("http error");

    write!(stdout, "track: {}\r\n", location).unwrap();
    write!(stdout, "scli ☁️  🦀 ☁️ \r\n").unwrap();
    write!(stdout, "playing: {}\r\n", filename).unwrap();
    stdout.lock().flush().unwrap();

    // event loop
    let player = thread::spawn(move || player(&args[1], rx));
    let user = thread::spawn(move || user(stdin, tx));

    user.join().unwrap();
    player.join().unwrap();
}

fn resolve(url: String) -> Result<String, reqwest::Error> {
    let oauth = env::var("SC_TOKEN").expect("no oauth token set");
    let client = reqwest::Client::builder()
        .redirect(reqwest::RedirectPolicy::none())
        .build()?;

    let mut resp = client
        .get("https://api.soundcloud.com/resolve")
        .header(header::USER_AGENT, "scli")
        .query(&[("oauth_token", oauth)])
        .query(&[("url", url)])
        .send()?;

    let resolved: ResolvedResource = resp.json()?;
    Ok(resolved.location)
}

fn user(mut stdin: termion::input::Keys<termion::AsyncReader>, tx: mpsc::Sender<UserInput>) {
    // listen for user input
    loop {
        let input = stdin.next();
        if let Some(Ok(key)) = input {
            match key {
                Key::Char('q') => {
                    tx.send(UserInput::Quit).unwrap();
                    break;
                }
                Key::Char(' ') => {
                    tx.send(UserInput::PlayPause).unwrap();
                }
                _ => continue,
            }
        }

        thread::sleep(time::Duration::from_millis(50));
    }
}

fn player(filename: &String, rx: mpsc::Receiver<UserInput>) {
    // load device and decode audio
    let device = rodio::default_output_device().unwrap();
    let file = File::open(filename).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    // start audio on registered device
    let sink = Sink::new(&device);
    sink.append(source);

    // listen for user events
    loop {
        let event = rx.recv().unwrap();
        match event {
            UserInput::PlayPause => {
                if sink.is_paused() {
                    sink.play()
                } else {
                    sink.pause();
                }
            }
            UserInput::Quit => break,
        }
    }
}
