use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Widget;

pub struct Wave {
    pub width: u16,
    pub height: u16,
    pub progress: f32,
    pub samples: Vec<u16>,
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
    pub fn width(&mut self, width: u16) -> &mut Wave {
        self.width = width;
        self
    }

    pub fn height(&mut self, height: u16) -> &mut Wave {
        self.height = height;
        self
    }

    pub fn progress(&mut self, progress: f32) -> &mut Wave {
        self.progress = progress;
        self
    }

    pub fn samples(&mut self, samples: Vec<u16>) -> &mut Wave {
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
