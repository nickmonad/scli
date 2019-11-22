#[macro_use]
extern crate serde;

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

fn main() {
    // prepare terminal
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let stdin = termion::async_stdin().keys();

    // read file name
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    // build a channel for user -> player
    let (tx, rx) = mpsc::channel();

    // get resolved track location
    let client = sc::Client::new();
    let stream = client.stream("https://soundcloud.com/nickmonad/sunsets-in-space".to_string())
        .expect("http error");

    write!(stdout, "stream: {}\r\n", stream).unwrap();
    write!(stdout, "scli ☁️  🦀 ☁️ \r\n").unwrap();
    write!(stdout, "playing: {}\r\n", filename).unwrap();
    stdout.lock().flush().unwrap();

    // event loop
    let player = thread::spawn(move || player(&args[1], rx));
    let user = thread::spawn(move || user(stdin, tx));

    user.join().unwrap();
    player.join().unwrap();
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

mod sc {
    use std::env;
    use reqwest::header;

    pub struct Client {
        client: reqwest::Client,
        oauth: String,
        url: String,
    }

    #[derive(Deserialize)]
    pub struct Resource {
        pub location: String,
    }

    #[derive(Deserialize)]
    pub struct Track {
        pub stream_url: String,
    }

    impl Client {
        pub fn new() -> Client {
            let oauth = env::var("SC_TOKEN").expect("no oauth token set");
            let rc = reqwest::Client::builder()
                .redirect(reqwest::RedirectPolicy::none())
                .build()
                .unwrap();

            Client { client: rc, oauth: oauth, url: "https://api.soundcloud.com".to_string() }
        }

        pub fn resolve(&self, url: String) -> Result<String, reqwest::Error> {
            let endpoint = format!("{}{}", self.url, "/resolve");
            let mut resp = self.client
                .get(&endpoint)
                .header(header::USER_AGENT, "scli")
                .query(&[("oauth_token", &self.oauth)])
                .query(&[("url", url)])
                .send()?;

            let resolved: self::Resource = resp.json()?;
            Ok(resolved.location)
        }

        pub fn stream(&self, url: String) -> Result<String, reqwest::Error> {
            self.resolve(url).and_then(|location: String| {
                let mut resp = self.client
                    .get(&location)
                    .header(header::USER_AGENT, "scli")
                    .query(&[("oauth_token", &self.oauth)])
                    .send()?;

                let track: self::Track = resp.json()?;
                Ok(track.stream_url)
            })
        }
    }
}
