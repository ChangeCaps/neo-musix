pub mod arrangement;
pub mod driver;

use crate::app::*;
use eframe::egui::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainPanel {
    Driver,
    Arrangement,
}

pub fn main_panel(app: &mut App, ctx: &CtxRef) {
    CentralPanel::default().show(ctx, |ui| match &app.main_panel {
        MainPanel::Driver => driver::driver(app, ui),
        MainPanel::Arrangement => arrangement::arrangement(app, ui),
    });
}
