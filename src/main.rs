#[macro_use]
extern crate serde;

use rodio::Sink;
use std::env;
use std::io;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use termion;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Gauge, Widget};
use tui::Terminal;
mod event;
mod soundcloud;

const SC_ORANGE: Color = Color::Rgb(237, 97, 43);

struct App {
    progress: u16,
}

impl App {
    fn new() -> App {
        App { progress: 0 }
    }

    fn update(&mut self) {
        self.progress += 5;
        if self.progress > 100 {
            self.progress = 0;
        }
    }
}

enum UserInput {
    PlayPause,
    Quit,
}

fn main() -> Result<(), failure::Error> {
    // terminal init
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // read file name
    let args: Vec<String> = env::args().collect();
    let url = &args[1];

    // resolve the track
    let client = soundcloud::Client::new();
    let track = client.track(url.to_string()).unwrap();

    // build a channel for user -> player
    let (tx, rx) = mpsc::channel();

    // start player thread and listen for incoming events
    let player = thread::spawn(move || player(track, rx));
    let events = event::Events::new();
    let mut app = App::new();

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(2), Constraint::Percentage(98)].as_ref())
                .split(f.size());

            Gauge::default()
                .style(Style::default().fg(SC_ORANGE))
                .percent(app.progress)
                .render(&mut f, chunks[0]);
        })?;

        match events.next()? {
            event::Event::Input(input) => match input {
                Key::Char('q') => {
                    tx.send(UserInput::Quit).unwrap();
                    break;
                }
                Key::Char(' ') => {
                    tx.send(UserInput::PlayPause).unwrap();
                }
                _ => {}
            },
            event::Event::Tick => {
                app.update();
            }
        }
    }

    player.join().unwrap();
    Ok(())
}

fn player(track: soundcloud::Track, rx: mpsc::Receiver<UserInput>) {
    // load default output device
    let device = rodio::default_output_device().unwrap();

    // resolve and decode stream
    let client = soundcloud::Client::new();
    let stream = client.stream(track.stream_url).unwrap();
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
