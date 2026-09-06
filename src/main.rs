/* #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release */
/* #![expect(rustdoc::missing_crate_level_docs)] // it's an example */

use std::{println, eprintln, fs};
use anyhow::Result;
use egui::Sense;
use eframe::egui;

const APP_NAME: &'static str = "reaction image keeper";

// boilerplate
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

// reaction image/video/meme/gif, whatever we re storing
#[derive(Debug, PartialEq, Clone)]
struct Entry {
    name: String,
    file_path: String,
    tags: Vec<String>,
}

// doohickey that keeps entries and their metadata
#[derive(Debug)]
struct EntryStorage {
    entries: Vec<Entry>,
    // todo add metadata storage n stuff
}

impl EntryStorage {
    fn new() -> Self {
        EntryStorage {
            entries: vec![],
        }
    }

    fn upd_storage(&mut self) -> Result<()> {
        let mut new_entries: Vec<Entry> = vec![];

        for file in fs::read_dir("./entries")? {
            let file = file?;
            new_entries.push(Entry {
                name: file.file_name().to_string_lossy().to_string(),
                file_path: file.path().to_string_lossy().to_string(),
                tags: vec![], // todo add propper tag search if entry exists in db
            });
        }

        self.entries = new_entries;

        Ok(())
    }

    fn upd_entries_with_filter(&self, search_rq: &String, selected_entries: &mut Vec<Entry>) {
        selected_entries.clear();

        for entry in &self.entries {
            if search_rq == "" || entry.name.contains(search_rq) /* && !selected_entries.contains(entry) */ {
                selected_entries.push(entry.clone());
            }
        }
    }
}

// window itself
#[derive(Debug)]
struct MyApp {
    search_rq: String,
    selected_entries: Vec<Entry>,
    entry_storage: EntryStorage,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            search_rq: "".to_string(),
            selected_entries: vec![],
            entry_storage: EntryStorage::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        /* println!("{:?}\n\n", self.selected_entries); */

        let screen_size = ui.ctx().input(|i| i.viewport().monitor_size)
            .unwrap(); // todo handle none

        // top bar
        egui::Panel::top("search").show(ui, |ui| {
            ui.horizontal(|ui| {
                let search_input = egui::TextEdit::singleline(&mut self.search_rq)
                    .hint_text("searc by name")
                    .return_key(egui::KeyboardShortcut {
                        logical_key: egui::Key::Enter,
                        modifiers: egui::Modifiers::NONE,
                    })
                    .show(ui)
                    .response;
                    
                if search_input.changed() {
                    self.entry_storage.upd_entries_with_filter(&self.search_rq, &mut self.selected_entries);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    let reload_list_button = ui.button("  ⟳  ");

                    if reload_list_button.clicked() {
                        let resp = self.entry_storage.upd_storage(); // todo handle failure
                        if resp.is_err() {
                            eprintln!("{:?}", resp.err());
                        }
                    }

                    let add_entry_button = ui.button("  +  ");
    
                    if add_entry_button.clicked() {
                        // todo add doohickey to select image
                    }
                });
            });
        });

        // album
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for entry in &self.selected_entries {
                    let (rect, resp) 
                        = ui.allocate_exact_size(screen_size * 0.33, Sense::empty());

                    let uri = format!("file:///{}",
                        std::env::current_dir().unwrap()
                        .join(&entry.file_path)
                        .to_string_lossy().replace('\\', "/"))
                        .replace("./", "");

                    /* println!("{uri}"); */

                    let image_box = ui.put(
                            rect,
                            egui::Image::from_uri(uri).fit_to_exact_size(rect.size())
                        );
                }
            });
        });
    }
}