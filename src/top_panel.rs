use crate::app::*;
use crate::main_panel::MainPanel;
use eframe::egui::*;

pub fn update(app: &mut App, ctx: &CtxRef) {
    TopPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            file(app, ui);
            main_panel(app, ui);
        });
    });
}

pub fn file(app: &mut App, ui: &mut Ui) {
    let popup_id = ui.make_persistent_id("file_popup");
    let response = ui.button("File");

    if response.clicked() {
        ui.memory().toggle_popup(popup_id);
    }

    containers::popup_below_widget(ui, popup_id, &response, |ui| {
        ui.set_max_width(100.0);

        if ui.button("Open").clicked() {}

        if ui.button("Save").clicked() {}

        if ui.button("Export").clicked() {}
    });
}

pub fn main_panel(app: &mut App, ui: &mut Ui) {
    ComboBox::from_id_source("main_panel_combo")
        .selected_text(format!("{:?}", app.main_panel))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut app.main_panel, MainPanel::Arrangement, "Arrangement");
            ui.selectable_value(&mut app.main_panel, MainPanel::Driver, "Driver");
        });
}
