#[allow(dead_code)]
mod event;

use std::io;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Widget;
use tui::Terminal;

use event::{Event, Events};

struct Wave {
    width: u32,
    height: u32,
    progress: f32,
    samples: Vec<u16>,
}

impl Default for Wave {
    fn default() -> Wave {
        Wave {
            width: 0,
            height: 0,
            progress: 0.0,
            samples: Vec::new(),
        }
    }
}

impl Wave {
    fn width(&mut self, width: u32) -> &mut Wave {
        self.width = width;
        self
    }

    fn height(&mut self, height: u32) -> &mut Wave {
        self.height = height;
        self
    }

    fn progress(&mut self, progress: f32) -> &mut Wave {
        self.progress = progress;
        self
    }

    fn samples(&mut self, samples: Vec<u16>) -> &mut Wave {
        self.samples = samples;
        self
    }
}

impl Widget for Wave {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let width = area.right() - area.left();
        let steps = self.samples.len() as u16 / width;

        for x in 0..width {
            let sample = self.samples[(x * steps) as usize];
            let ratio = ((sample as f32 / self.height as f32) * 10.0) as u8;

            let mut line = vec![" "; (10 - ratio) as usize];
            line.resize_with(10, || "|");

            for y in 0..10 {
                buf.set_string(x, y, line[y as usize], Style::default());
            }
        }
    }
}

fn main() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            Wave::default()
                .width(1000)
                .height(100)
                .progress(0.0)
                .samples(vec![43; 1321])
                .render(&mut f, size);
        })?;

        match events.next()? {
            Event::Input(key) => {
                if key == Key::Char('q') {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
