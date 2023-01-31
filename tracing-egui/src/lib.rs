mod archive;
mod layer;
mod panel;

#[cfg(feature = "smartstring")]
type SmartString = smartstring::SmartString<smartstring::LazyCompact>;
#[cfg(not(feature = "smartstring"))]
type SmartString = String;

pub use crate::{layer::EguiLayer, panel::LogPanel};

pub fn layer() -> EguiLayer {
    EguiLayer::new()
}

pub fn show(ctx: &egui::Context, open: &mut bool) -> Option<egui::InnerResponse<Option<()>>> {
    let window = egui::Window::new("Log")
        .resizable(true)
        .collapsible(true)
        .hscroll(true)
        .open(open);
    show_in(ctx, window)
}

pub fn show_in(ctx: &egui::Context, window: egui::Window<'_>) -> Option<egui::InnerResponse<Option<()>>> {
    window.show(ctx, |ui| {
        ui.add(LogPanel);
    })
}

