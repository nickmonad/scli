use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
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
        // calculate a "resolution" for the waveform
        // based on the width of the buffer area
        let width = area.right() - area.left();
        let steps = self.samples.len() as u16 / width;

        for x in 0..width {
            // given the calculated resolution, grab a sample
            // from the waveform at every step, and calculate
            // its relative display height (0 to 10)
            let sample = self.samples[(x * steps) as usize];
            let height = ((sample as f32 / self.height as f32) * 10.0) as u8;

            // create the line to display, up to height
            let mut line = vec![" "; (10 - height) as usize];
            line.resize_with(10, || "|");

            // stylize line with color based on the progress (as percentage)
            let relative_pos = (x as f32 / width as f32) * 100.0;
            let default_style = Style::default().modifier(Modifier::BOLD);
            let style = if self.progress > relative_pos {
                // progress if fully past x position
                default_style.fg(ORANGE)
            } else if self.progress as u8 == relative_pos as u8 {
                // progress is at or in-between x position
                let p = clamp(self.progress.fract() * 10.0, 0.0, 9.0) as u8;
                default_style.fg(COLORS[p as usize])
            } else {
                // progress is less than x position
                default_style
            };

            // draw line
            for y in 0..10 {
                buf.set_string(x, y, line[y as usize], style);
            }
        }
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        return min;
    }

    if value > max {
        return max;
    }

    value
}

const ORANGE: Color = Color::Rgb(237, 97, 43);
const COLORS: [Color; 10] = [
    Color::Rgb(255, 255, 255),
    Color::Rgb(253, 237, 231),
    Color::Rgb(251, 220, 208),
    Color::Rgb(249, 202, 184),
    Color::Rgb(247, 185, 161),
    Color::Rgb(245, 167, 137),
    Color::Rgb(243, 150, 114),
    Color::Rgb(241, 132, 90),
    Color::Rgb(239, 115, 67),
    Color::Rgb(237, 97, 43),
];
