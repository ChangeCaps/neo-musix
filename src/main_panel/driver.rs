use crate::app::*;
use crate::audio_engine::*;
use eframe::egui::*;
use std::collections::HashMap;

pub fn driver(app: &mut App, ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.label(format!("Running: {}", app.audio_handle.running()));

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    device(
                        "Input Device",
                        &mut app.audio_engine_descriptor.input_device,
                        &app.audio_handle.audio_devices.input_devices,
                        ui,
                    );
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    device(
                        "Output Device",
                        &mut app.audio_engine_descriptor.output_device,
                        &app.audio_handle.audio_devices.output_devices,
                        ui,
                    );
                });
            });
        });

        ui.horizontal(|ui| {
            ui.label("Latency");
            ui.add(DragValue::new(&mut app.audio_engine_descriptor.latency));
        });

        ui.horizontal(|ui| {
            if ui.button("Restart engine").clicked() {
                app.audio_handle
                    .start_engine(app.audio_engine_descriptor.clone());
            }

            if ui.button("Update devices").clicked() {
                app.audio_handle.update_audio_devices();
            }
        });
    });
}

pub fn device(
    label: &str,
    selected: &mut Option<String>,
    devices: &HashMap<String, DeviceInfo>,
    ui: &mut Ui,
) {
    ComboBox::from_label(label)
        .selected_text(match selected {
            Some(selected) => selected.to_string(),
            None => format!("Default").to_string(),
        })
        .show_ui(ui, |ui| {
            ui.set_max_width(500.0);

            for name in devices.keys() {
                ui.selectable_value(selected, Some(name.clone()), name);
            }
        });

    if let Some(name) = selected {
        if let Some(device) = devices.get(name) {
            ui.label(format!("Sample rate: {}", device.sample_rate));
            ui.label(format!("Channels: {}", device.channels));
            ui.label(format!("Sample format: {:?}", device.sample_format));
        } else {
            *selected = None;
        }
    }
}
