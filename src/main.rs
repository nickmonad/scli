#[macro_use]
extern crate serde;

use rodio::Sink;
use std::env;
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

fn main() -> std::io::Result<()> {
    // prepare terminal
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let stdin = termion::async_stdin().keys();

    // read file name
    let args: Vec<String> = env::args().collect();
    let url = &args[1];

    // build a channel for user -> player
    let (tx, rx) = mpsc::channel();

    write!(stdout, "scli ☁️  🦀 ☁️ \r\n").unwrap();
    write!(stdout, "streaming: {}\r\n", url).unwrap();
    stdout.lock().flush().unwrap();

    // event loop
    let player = thread::spawn(move || player(&args[1], rx));
    let user = thread::spawn(move || user(stdin, tx));

    user.join().unwrap();
    player.join().unwrap();

    Ok(())
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

fn player(url: &String, rx: mpsc::Receiver<UserInput>) {
    // load device and decode audio
    let device = rodio::default_output_device().unwrap();

    // resolve stream
    let client = sc::Client::new();
    let stream = client.stream(url.to_string()).unwrap();
    let source = rodio::Decoder::new(BufReader::new(stream)).unwrap();

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
    use reqwest::header;
    use std::env;
    use std::io::Cursor;
    use std::thread;

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

            Client {
                client: rc,
                oauth: oauth,
                url: "https://api.soundcloud.com".to_string(),
            }
        }

        pub fn resolve(&self, url: String) -> Result<String, reqwest::Error> {
            let endpoint = format!("{}{}", self.url, "/resolve");
            let mut resp = self
                .client
                .get(&endpoint)
                .header(header::USER_AGENT, "scli")
                .query(&[("oauth_token", &self.oauth)])
                .query(&[("url", url)])
                .send()?;

            let resolved: self::Resource = resp.json()?;
            Ok(resolved.location)
        }

        pub fn stream(&self, url: String) -> Result<Cursor<Vec<u8>>, reqwest::Error> {
            self.resolve(url)
                .and_then(|location: String| {
                    // fetch track metadata
                    let mut resp = self
                        .client
                        .get(&location)
                        .header(header::USER_AGENT, "scli")
                        .query(&[("oauth_token", &self.oauth)])
                        .send()?;

                    let track: self::Track = resp.json()?;
                    Ok(track.stream_url)
                })
                .and_then(|stream_url: String| {
                    // fetch stream url
                    let mut resp = self
                        .client
                        .get(&stream_url)
                        .header(header::USER_AGENT, "scli")
                        .query(&[("oauth_token", &self.oauth)])
                        .send()?;

                    let resource: self::Resource = resp.json()?;
                    Ok(resource.location)
                })
                .map(|location: String| {
                    // fetch raw audio from resolved stream CDN location
                    let mut resp = self.client.get(&location).send().unwrap();
                    // panic!("status: {}", resp.status().as_str());

                    // create a shared buffer and copy the audio response
                    let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
                    let ret = buffer.clone();
                    thread::spawn(move || {
                        resp.copy_to(&mut buffer).unwrap();
                    });

                    ret
                })
        }
    }
}
