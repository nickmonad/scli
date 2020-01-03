use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::Widget;

pub struct Clock {
    pub elapsed_ms: u32,
    pub total_ms: u32,
}

impl Default for Clock {
    fn default() -> Clock {
        Clock {
            elapsed_ms: 0,
            total_ms: 0,
        }
    }
}

impl Clock {
    pub fn elapsed(&mut self, elapsed_ms: u32) -> &mut Clock {
        self.elapsed_ms = elapsed_ms;
        self
    }

    pub fn total(&mut self, total_ms: u32) -> &mut Clock {
        self.total_ms = total_ms;
        self
    }

    fn format(value_ms: u32) -> String {
        let hours = value_ms / (3600 * 1000); // hours in ms
        let minutes = value_ms / (60 * 1000); // minutes in ms
        let seconds = value_ms % (60 * 1000) / 1000; // seconds in ms

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

impl Widget for Clock {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        // show elapsed time
        let elapsed = Clock::format(self.elapsed_ms);
        buf.set_string(area.left(), area.top(), elapsed, Style::default());

        // show total time
        let total = Clock::format(self.total_ms);
        buf.set_string(
            area.right() - total.len() as u16,
            area.top(),
            total,
            Style::default(),
        );
    }
}
