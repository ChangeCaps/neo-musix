use eframe::egui::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AudioSourceId(u64);

pub trait AudioSource {
    fn ui(&mut self, ui: &mut Ui) -> Response;
    fn sample(&self, time: f32, channel: u16) -> f32;
}
