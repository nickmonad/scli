use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
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
    fn format(value_ms: u32) -> String {
        let hours = value_ms / (3600 * 1000); // hours in ms
        let minutes = value_ms / (60 * 1000) % 60; // minutes in ms
        let seconds = value_ms % (60 * 1000) / 1000; // seconds in ms

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

pub struct Status {
    pub is_playing: bool,
    pub clock: Clock,
}

impl Default for Status {
    fn default() -> Status {
        Status {
            is_playing: false,
            clock: Clock::default(),
        }
    }
}

impl Status {
    pub fn is_playing(&mut self, is_playing: bool) -> &mut Status {
        self.is_playing = is_playing;
        self
    }

    pub fn clock(&mut self, clock: Clock) -> &mut Status {
        self.clock = clock;
        self
    }
}

impl Widget for Status {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        // show elapsed time
        let elapsed = Clock::format(self.clock.elapsed_ms);
        buf.set_string(area.left(), area.top(), &elapsed, Style::default());

        // show state
        let state = if self.is_playing { "Playing" } else { "Paused" };
        buf.set_string(
            area.left() + elapsed.len() as u16 + 2,
            area.top(),
            state,
            Style::default(),
        );

        // show total time
        let total = Clock::format(self.clock.total_ms);
        buf.set_string(
            area.right() - total.len() as u16,
            area.top(),
            total,
            Style::default(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_format() {
        assert_eq!(Clock::format(0), "00:00");

        assert_eq!(Clock::format(1000), "00:01");
        assert_eq!(Clock::format(1000 * 60), "01:00");
        assert_eq!(Clock::format(1000 * 60 * 60), "01:00:00");

        assert_eq!(Clock::format(271019), "04:31");
        assert_eq!(Clock::format(4112738), "01:08:32");
        assert_eq!(Clock::format(8688931), "02:24:48");
    }
}
