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
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::widgets::{Paragraph, Text, Widget};
use tui::Terminal;
mod clock;
mod event;
mod soundcloud;
mod wave;

struct Player<'a> {
    track: &'a soundcloud::Track,
    audio: rodio::Sink,
    timer: Arc<Mutex<Duration>>,
    state: PlayerState,
    progress: f32,
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
            audio: sink,
            timer: timer,
            state: PlayerState::Playing,
            progress: 0.0,
        }
    }

    fn update(&mut self, msg: PlayerEvent) {
        match msg {
            PlayerEvent::Tick => {
                if self.audio.empty() {
                    self.state = PlayerState::Stopped;
                } else {
                    if self.state == PlayerState::Stopped {
                        self.progress = 0.0;
                    } else {
                        self.progress =
                            (self.elapsed() as f32 / self.track.duration as f32) * 100.0;
                    }
                }
            }
            PlayerEvent::PlayPause => {
                if self.audio.is_paused() {
                    self.audio.play();
                    self.state = PlayerState::Playing;
                } else {
                    self.audio.pause();
                    self.state = PlayerState::Paused;
                }
            }
        }
    }

    fn state(&self) -> PlayerState {
        self.state.clone()
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn elapsed(&self) -> u32 {
        let val = *self.timer.lock().unwrap();
        val.as_millis() as u32
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
    let sc = soundcloud::Client::new();
    let track = sc.track(url.to_string()).unwrap();
    let wave = sc.wave(&track).unwrap();

    // start player thread and listen for incoming from it
    let mut player = Player::new(&track);
    let events = event::Events::new();

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(10),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let header = [
                Text::styled(&track.user.username, Style::default()),
                Text::raw("\n"),
                Text::styled(&track.title, Style::default().modifier(Modifier::BOLD)),
            ];
            Paragraph::new(header.iter())
                .alignment(Alignment::Left)
                .render(&mut f, chunks[0]);

            wave::Wave::default()
                .width(wave.width)
                .height(wave.height)
                .samples(wave.samples.clone())
                .progress(player.progress())
                .render(&mut f, chunks[1]);

            clock::Clock::default()
                .elapsed(player.elapsed())
                .total(track.duration)
                .render(&mut f, chunks[2]);
        })?;

        match events.next()? {
            event::Event::Tick => {
                player.update(PlayerEvent::Tick);
                if player.state() == PlayerState::Stopped {
                    break;
                }
            }
            event::Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Char(' ') => {
                    player.update(PlayerEvent::PlayPause);
                }
                _ => {}
            },
        }
    }

    Ok(())
}
