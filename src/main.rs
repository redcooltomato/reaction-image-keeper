/* #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release */
/* #![expect(rustdoc::missing_crate_level_docs)] // it's an example */

use eframe::egui;

const APP_NAME: &'static str = "reaction image keeper";

fn main() -> eframe::Result {
   /*  env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`). */
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    search_rq: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            search_rq: "".to_string(),
        }
    }
}

fn upd_filter() {
    // todo
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("search").show(ui, |ui| {
            ui.horizontal(|ui| {
                let search_input = egui::TextEdit::singleline(&mut self.search_rq)
                    .hint_text("searc by name")
                    .return_key(egui::KeyboardShortcut {
                        logical_key: egui::Key::Enter,
                        modifiers: egui::Modifiers::NONE,
                    })
                    .show(ui);
                    
                if search_input.response.changed() {
                    upd_filter();
                }
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                // todo
            });
        });
    }
}