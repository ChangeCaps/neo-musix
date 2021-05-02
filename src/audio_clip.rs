use crate::audio_engine::*;
use crate::audio_source::*;
use eframe::egui::*;

pub struct Frame {
    pub samples: Vec<f32>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    pub fn push(&mut self, sample: f32) {
        self.samples.push(sample);
    }

    pub fn sample(&self, channel: u16) -> f32 {
        *self.samples.get(channel as usize).unwrap_or(&0.0)
    }
}

pub struct AudioClip {
    pub frames: Vec<Frame>,
    pub format: AudioFormat,
}

impl AudioClip {
    pub fn new(format: AudioFormat) -> Self {
        Self {
            frames: Vec::new(),
            format,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn push_sample(&mut self, sample: f32) {
        if self.frames.is_empty() {
            self.frames.push(Frame {
                samples: vec![sample],
            });
        } else {
            let frame = self.frames.last_mut().unwrap();

            if frame.samples.len() as u16 == self.format.channels {
                self.frames.push(Frame {
                    samples: vec![sample],
                });
            } else {
                frame.push(sample);
            }
        }
    }
}

impl AudioSource for AudioClip {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.button("Audio Clip")
    }

    fn sample(&self, time: f32, channel: u16) -> f32 {
        let frame = (self.format.frame_rate as f32 * time) as usize;

        if let Some(frame) = self.frames.get(frame) {
            frame.sample(channel)
        } else {
            0.0
        }
    }
}
