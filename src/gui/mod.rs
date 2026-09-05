use eframe::egui;

/// Launches the egui/eframe front end, blocking until the window closes.
pub fn run() -> eframe::Result<()> {
    eframe::run_native(
        "the-bug",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App))),
    )
}

struct App;

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |_ui| {});
    }
}
