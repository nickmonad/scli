#[macro_use]
extern crate serde;

use rodio::Sink;
use rodio::Source;
use std::env;
use std::io;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use termion;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Gauge, Widget};
use tui::Terminal;
mod event;
mod soundcloud;
mod wave;

const SC_ORANGE: Color = Color::Rgb(237, 97, 43);

struct Player<'a> {
    track: &'a soundcloud::Track,
    audio_sink: rodio::Sink,
    timer: Arc<Mutex<Duration>>,
    state: PlayerState,
    progress: u8,
}

#[derive(Clone, Eq, PartialEq)]
enum PlayerState {
    Playing,
    Paused,
    Stopped,
}

enum PlayerEvent {
    Tick,
    PlayPause,
}

impl Player<'_> {
    fn new(track: &soundcloud::Track) -> Player {
        // load default output device
        let device = rodio::default_output_device().unwrap();

        // resolve and decode stream
        let client = soundcloud::Client::new();
        let stream = client.stream(&track.stream_url).unwrap();
        let source = rodio::Decoder::new(BufReader::new(stream)).unwrap();

        let timer = Arc::new(Mutex::new(Duration::from_secs(0)));
        let with_elapsed = source.buffered().elapsed(Arc::clone(&timer));

        // start audio on registered device
        let sink = Sink::new(&device);
        sink.append(with_elapsed);

        Player {
            track: track,
            audio_sink: sink,
            timer: timer,
            state: PlayerState::Playing,
            progress: 0,
        }
    }

    fn update(&mut self, msg: PlayerEvent) {
        match msg {
            PlayerEvent::Tick => {
                if self.audio_sink.empty() {
                    self.state = PlayerState::Stopped;
                } else {
                    if self.state == PlayerState::Stopped {
                        self.progress = 0;
                    } else {
                        let val = *self.timer.lock().unwrap();
                        self.progress =
                            ((val.as_millis() as f32 / self.track.duration as f32) * 100.0) as u8;
                    }
                }
            }
            PlayerEvent::PlayPause => {
                if self.audio_sink.is_paused() {
                    self.audio_sink.play();
                    self.state = PlayerState::Playing;
                } else {
                    self.audio_sink.pause();
                    self.state = PlayerState::Paused;
                }
            }
        }
    }

    fn state(&self) -> PlayerState {
        self.state.clone()
    }

    fn progress(&self) -> u8 {
        self.progress
    }
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

    // resolve the track and waveform
    let client = soundcloud::Client::new();
    let track = client.track(url.to_string()).unwrap();
    let wave = client.wave(&track).unwrap();

    // start player thread and listen for incoming from it
    let mut app = Player::new(&track);
    let events = event::Events::new();

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            wave::Wave::default()
                .width(wave.width)
                .height(wave.height)
                .samples(wave.samples.clone())
                .render(&mut f, chunks[0])
        })?;

        match events.next()? {
            event::Event::Tick => {
                app.update(PlayerEvent::Tick);
                if app.state() == PlayerState::Stopped {
                    break;
                }
            }
            event::Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Char(' ') => {
                    app.update(PlayerEvent::PlayPause);
                }
                _ => {}
            },
        }
    }

    Ok(())
}
