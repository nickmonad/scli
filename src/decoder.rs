// Using a modified version of rodio's mp3 decoder,
// without the Seek trait constraint, so we can stream in memory
// https://github.com/RustAudio/rodio/blob/master/src/decoder/mp3.rs
use rodio::source::Source;
use std::io::Read;
use std::time::Duration;

use minimp3::{Decoder, Frame};

pub struct Mp3Decoder<R>
where
    R: Read,
{
    decoder: Decoder<R>,
    current_frame: Frame,
    current_frame_offset: usize,
}

impl<R> Mp3Decoder<R>
where
    R: Read,
{
    pub fn new(data: R) -> Result<Self, ()> {
        let mut decoder = Decoder::new(data);
        let current_frame = decoder.next_frame().map_err(|_| ())?;

        Ok(Mp3Decoder {
            decoder,
            current_frame,
            current_frame_offset: 0,
        })
    }
}

impl<R> Source for Mp3Decoder<R>
where
    R: Read,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.current_frame.data.len())
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.current_frame.channels as _
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.current_frame.sample_rate as _
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl<R> Iterator for Mp3Decoder<R>
where
    R: Read,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.current_frame_offset == self.current_frame.data.len() {
            self.current_frame_offset = 0;
            match self.decoder.next_frame() {
                Ok(frame) => self.current_frame = frame,
                _ => return None,
            }
        }

        let v = self.current_frame.data[self.current_frame_offset];
        self.current_frame_offset += 1;

        return Some(v);
    }
}
