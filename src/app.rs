use crate::audio_engine::*;
use crate::audio_source::*;
use crate::main_panel::*;
use eframe::{egui::*, epi};
use std::collections::HashMap;

pub struct App {
    pub main_panel: MainPanel,
    pub audio_handle: AudioEngineHandle,
    pub audio_engine_descriptor: AudioEngineDescriptor,
    pub sources: HashMap<AudioSourceId, Box<dyn AudioSource>>,
}

impl App {
    pub fn new() -> Self {
        let descriptor = AudioEngineDescriptor::default();

        let handle = crate::audio_engine::init(descriptor).unwrap();

        log::debug!("Audio devices: {:#?}", handle.audio_devices);

        Self {
            main_panel: MainPanel::Arrangement,
            audio_handle: handle,
            audio_engine_descriptor: Default::default(),
            sources: HashMap::new(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Neo Musix"
    }

    fn update(&mut self, ctx: &CtxRef, frame: &mut epi::Frame<'_>) {
        crate::top_panel::update(self, ctx);
        crate::main_panel::main_panel(self, ctx);
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        storage.set_string(
            "engine_descriptor",
            ron::to_string(&self.audio_engine_descriptor).unwrap(),
        );
    }

    fn load(&mut self, storage: &dyn epi::Storage) {
        if let Some(val) = storage.get_string("engine_descriptor") {
            if let Ok(descriptor) = ron::from_str(&val) {
                self.audio_engine_descriptor = descriptor;
                self.audio_handle
                    .start_engine(self.audio_engine_descriptor.clone());
            }
        }
    }
}
