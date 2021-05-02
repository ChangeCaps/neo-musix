use crate::audio_clip::*;
use crate::audio_source::*;
use cpal::{traits::*, *};
use crossbeam::channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

type Streams = (Stream, Stream);

fn lauch_engine(
    descriptor: AudioEngineDescriptor,
    sender: Sender<Response>,
    receiver: Receiver<Command>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| {
        let mut engine = AudioEngine {
            descriptor,
            host: default_host(),
            input_devices: HashMap::new(),
            output_devices: HashMap::new(),
            sender,
            receiver,
            frame: 0,
            channel: 0,
            recording: None,
        };

        engine.update_devices().unwrap();
        let _streams = engine.run().unwrap();

        std::thread::park();
    })
}

pub fn init(descriptor: AudioEngineDescriptor) -> anyhow::Result<AudioEngineHandle> {
    let (handle_sender, receiver) = unbounded();
    let (sender, handle_receiver) = unbounded();

    let engine_thread = lauch_engine(descriptor, handle_sender, handle_receiver);

    let mut handle = AudioEngineHandle {
        engine_thread: Some(engine_thread),
        audio_devices: Default::default(),
        sender,
        receiver,
    };

    handle.update_audio_devices();

    Ok(handle)
}

enum Command {
    UpdateAudioDevices,
    StartRecording,
}

enum Response {
    UpdateAudioDevices(AudioDevices),
    StopRecording(AudioSourceId),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AudioEngineDescriptor {
    pub latency: f32,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
}

impl Default for AudioEngineDescriptor {
    fn default() -> Self {
        Self {
            latency: 5.0,
            input_device: None,
            output_device: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AudioFormat {
    pub frame_rate: u32,
    pub channels: u16,
}

impl AudioFormat {
    pub fn sample_format(&self) -> u32 {
        self.frame_rate * self.channels as u32
    }
}

#[derive(Debug, Default)]
pub struct AudioDevices {
    pub input_devices: HashMap<String, DeviceInfo>,
    pub output_devices: HashMap<String, DeviceInfo>,
}

#[derive(Debug)]
pub struct DeviceInfo {
    pub sample_rate: u32,
    pub channels: u32,
    pub sample_format: SampleFormat,
}

pub struct AudioEngineHandle {
    engine_thread: Option<std::thread::JoinHandle<()>>,
    pub audio_devices: AudioDevices,
    sender: Sender<Command>,
    receiver: Receiver<Response>,
}

impl AudioEngineHandle {
    pub fn running(&self) -> bool {
        self.engine_thread.is_some()
    }

    pub fn stop_engine(&mut self) {
        if let Some(thread) = self.engine_thread.take() {
            thread.thread().unpark();
            thread.join().unwrap();
        }
    }

    pub fn start_engine(&mut self, descriptor: AudioEngineDescriptor) {
        self.stop_engine();

        let (handle_sender, receiver) = unbounded();
        let (sender, handle_receiver) = unbounded();

        let engine_thread = lauch_engine(descriptor, handle_sender, handle_receiver);

        self.engine_thread = Some(engine_thread);

        self.sender = sender;
        self.receiver = receiver;
    }

    pub fn update_audio_devices(&mut self) {
        self.sender.send(Command::UpdateAudioDevices).unwrap();

        match self.receiver.recv().unwrap() {
            Response::UpdateAudioDevices(audio_devices) => self.audio_devices = audio_devices,
            _ => panic!("wrong response"),
        }
    }
}

pub struct AudioEngine {
    descriptor: AudioEngineDescriptor,
    host: Host,
    input_devices: HashMap<String, Arc<Device>>,
    output_devices: HashMap<String, Arc<Device>>,
    sender: Sender<Response>,
    receiver: Receiver<Command>,
    recording: Option<AudioClip>,
    frame: u32,
    channel: u16,
}

impl AudioEngine {
    pub fn handle_commands(&mut self, format: &AudioFormat) {
        for command in self.receiver.try_iter().collect::<Vec<_>>() {
            match command {
                Command::UpdateAudioDevices => {
                    let audio_devices = self.update_devices().unwrap();

                    self.sender
                        .send(Response::UpdateAudioDevices(audio_devices))
                        .unwrap();
                }
                Command::StartRecording => {
                    self.recording = Some(AudioClip::new(format.clone()));
                }
            }
        }
    }

    pub fn run(mut self) -> anyhow::Result<Streams> {
        let input_device = match &self.descriptor.input_device {
            Some(device) => self.input_devices.get(device).unwrap().clone(),
            None => Arc::new(self.host.default_input_device().unwrap()),
        };

        let output_device = match &self.descriptor.output_device {
            Some(device) => self.output_devices.get(device).unwrap().clone(),
            None => Arc::new(self.host.default_output_device().unwrap()),
        };

        let input_config = input_device
            .supported_input_configs()?
            .next()
            .unwrap()
            .with_max_sample_rate();
        let output_config = output_device
            .supported_output_configs()?
            .next()
            .unwrap()
            .with_max_sample_rate();

        let channels = output_config.channels();
        let sample_rate = output_config.sample_rate().0;

        let latency_frames = (self.descriptor.latency / 1_000.0) * sample_rate as f32;
        let latency_samples = latency_frames as usize * channels as usize;

        let buf = ringbuf::RingBuffer::new(latency_samples * 2);
        let (mut prod, mut cons) = buf.split();

        for _ in 0..latency_samples {
            prod.push(0.0).unwrap();
        }

        let input_stream = match input_config.sample_format() {
            SampleFormat::F32 => input_device.build_input_stream(
                &input_config.into(),
                move |data, callback| input::<f32>(&mut prod, data, callback),
                error,
            ),
            SampleFormat::I16 => input_device.build_input_stream(
                &input_config.into(),
                move |data, callback| input::<i16>(&mut prod, data, callback),
                error,
            ),
            SampleFormat::U16 => input_device.build_input_stream(
                &input_config.into(),
                move |data, callback| input::<u16>(&mut prod, data, callback),
                error,
            ),
        }?;

        let format = AudioFormat {
            frame_rate: sample_rate / channels as u32,
            channels,
        };

        let output_stream = match output_config.sample_format() {
            SampleFormat::F32 => output_device.build_output_stream(
                &output_config.into(),
                move |data, callback| {
                    self.handle_commands(&format);
                    output::<f32>(&mut self, &mut cons, &format, data, callback)
                },
                error,
            ),
            SampleFormat::I16 => output_device.build_output_stream(
                &output_config.into(),
                move |data, callback| {
                    self.handle_commands(&format);
                    output::<i16>(&mut self, &mut cons, &format, data, callback)
                },
                error,
            ),
            SampleFormat::U16 => output_device.build_output_stream(
                &output_config.into(),
                move |data, callback| {
                    self.handle_commands(&format);
                    output::<u16>(&mut self, &mut cons, &format, data, callback)
                },
                error,
            ),
        }?;

        input_stream.play().unwrap();
        output_stream.play().unwrap();

        Ok((input_stream, output_stream))
    }

    pub fn update_devices(&mut self) -> anyhow::Result<AudioDevices> {
        let input_devices = self
            .host
            .input_devices()?
            .map(|device| (device.name().unwrap(), Arc::new(device)))
            .collect::<HashMap<_, _>>();

        let output_devices = self
            .host
            .output_devices()?
            .map(|device| (device.name().unwrap(), Arc::new(device)))
            .collect::<HashMap<_, _>>();

        let audio_devices = AudioDevices {
            input_devices: input_devices
                .iter()
                .map(|(name, device)| {
                    let config = device.supported_input_configs().unwrap().next().unwrap();
                    let sample_config = config.with_max_sample_rate();

                    (
                        name.clone(),
                        DeviceInfo {
                            sample_rate: sample_config.sample_rate().0,
                            channels: sample_config.channels() as u32,
                            sample_format: sample_config.sample_format(),
                        },
                    )
                })
                .collect(),
            output_devices: output_devices
                .iter()
                .map(|(name, device)| {
                    let config = device.supported_output_configs().unwrap().next().unwrap();
                    let sample_config = config.with_max_sample_rate();

                    (
                        name.clone(),
                        DeviceInfo {
                            sample_rate: sample_config.sample_rate().0,
                            channels: sample_config.channels() as u32,
                            sample_format: sample_config.sample_format(),
                        },
                    )
                })
                .collect(),
        };

        self.input_devices = input_devices;
        self.output_devices = output_devices;

        Ok(audio_devices)
    }
}

fn error(err: StreamError) {
    log::error!("engine error: {}", err);
}

fn input<T: Sample>(prod: &mut ringbuf::Producer<f32>, data: &[T], _: &InputCallbackInfo) {
    for sample in data {
        let _ = prod.push(sample.to_f32());
    }
}

fn output<T: Sample>(
    engine: &mut AudioEngine,
    cons: &mut ringbuf::Consumer<f32>,
    format: &AudioFormat,
    data: &mut [T],
    _: &OutputCallbackInfo,
) {
    for sample in data {
        engine.channel += 1;
        engine.channel %= format.channels;

        if engine.channel == 0 {
            engine.frame += 1;
        }

        let feedback = cons.pop().unwrap_or(0.0);

        if let Some(clip) = &mut engine.recording {
            if !clip.is_empty() || engine.channel == 0 {
                clip.push_sample(feedback);
            }
        }

        *sample = T::from(&feedback);
    }
}
