use std::{env::current_dir, eprintln, fs::{self, create_dir_all}, path::Path, vec};
use anyhow::Result;
use egui::{Margin, Sense, Stroke, Ui, Vec2};
use eframe::egui;
use arboard::{Clipboard, ImageData};
use image::{ImageReader, image_dimensions};

const APP_NAME: &'static str = "reaction image keeper";
const MAGIC_FRACTION: f32 = 0.15;

// boilerplate
fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let screen_size = cc.egui_ctx.input(|i| i.viewport().monitor_size)
                .unwrap();

            cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(
                screen_size * MAGIC_FRACTION
                + Vec2::new(50., 50.) // magic numbers essentially
            ));

            Ok(Box::<App>::default())
        }),
    )
}

// reaction image/video/meme/gif, whatever we re storing
#[derive(Debug, PartialEq, Clone)]
struct Entry {
    name: String, // with file extention
    file_path: String,
    tags: Vec<String>, // wip/unused
    width: u32,
    height: u32,
    img_data_raw: Vec<u8>,
}

// stuff for the app itself
struct App {
    search_rq: String,
    selected_entries: Vec<Entry>,
    entries: Vec<Entry>, // todo add metadata storage n stuff
    started: bool,
    arboard_ctx: Clipboard,
}

// defaults
impl Default for App {
    fn default() -> Self {
        Self {
            search_rq: "".to_string(),
            selected_entries: vec![],
            entries: vec![],
            started: false,
            arboard_ctx: Clipboard::new().unwrap(),
        }
    }
}

impl App {
    fn upd_storage(&mut self) -> Result<()> {
        let mut new_entries: Vec<Entry> = vec![];

        create_dir_all(current_dir().unwrap().join("entries"))?;

        for file in fs::read_dir("./entries")? {
            let file = file?;
            let img  = ImageReader::open(&file.path()).unwrap();
            let (w, h) = image_dimensions(&file.path()).unwrap();

            new_entries.push(Entry {
                name: file.path().with_extension("").file_name().unwrap().to_string_lossy().to_string(),
                file_path: file.path().to_string_lossy().to_string(),
                tags: vec![], // todo add propper tag search if entry exists in db
                width: w,
                height: h,
                img_data_raw: img.decode().unwrap().to_rgba8().into_raw(),
            });
        }

        self.entries = new_entries;

        Ok(())
    }

    fn upd_filtered_entries(&mut self) {
        self.selected_entries.clear();

        for entry in &self.entries {
            if self.search_rq.trim() == "" || entry.name.contains(&self.search_rq) /* && !selected_entries.contains(entry) */ {
                self.selected_entries.push(entry.clone());
            }
        }
    }

    fn copy_image_to_clipboard(clip: &mut Clipboard, entry: &Entry) {
        match clip.set_image(ImageData {
                width: entry.width as usize,
                height: entry.height as usize,
                bytes: std::borrow::Cow::from(entry.img_data_raw.clone()),
            }) {
            Ok(_) => (),
            Err(e) => eprintln!("error occured while copying image to clipboard: {}", e),
        }
    }

    //
    // lots of ui moved here to help with indentation (at some point indentation depth reached 13)
    //

    fn search_bar_ui(&mut self, ui: &mut Ui) {
        let search_input = egui::TextEdit::singleline(&mut self.search_rq)
            .hint_text("searc by name")
            .return_key(egui::KeyboardShortcut {
                logical_key: egui::Key::Enter,
                modifiers: egui::Modifiers::NONE,
            })
            .show(ui)
            .response;
            
        if search_input.changed() {
            self.upd_filtered_entries();
        }
    }

    fn theme_change_ui(&mut self, ui: &mut Ui) {
        let theme_change = ui.button(
            if ui.ctx().theme() == egui::Theme::Light { "☼" }
            else { "☾" }
        );

        if theme_change.clicked() {
            if ui.ctx().theme() == egui::Theme::Light {
                ui.ctx().set_theme(egui::Theme::Dark);
            } else {
                ui.ctx().set_theme(egui::Theme::Light);
            }
        }
    }

    fn reload_list_ui(&mut self, ui: &mut Ui) {
        let reload_list_button = ui.button("  ⟳  ");

        if reload_list_button.clicked() {
            let resp = self.upd_storage(); // todo handle failure
            if resp.is_err() {
                eprintln!("{:?}", resp.err());
            }
            self.upd_filtered_entries();
        }
    }

    fn add_entry_ui(&mut self, ui: &mut Ui) {
        let add_entry_button = ui.button("  +  ");
    
        if add_entry_button.clicked() {
            let selected_files = rfd::FileDialog::new().pick_files().unwrap_or(vec![]);

            for file in selected_files {
                let res = fs::copy(
                    &file, 
                    Path::new("./entries").join(&file.file_name().unwrap())
                );
                if res.is_err() {
                    eprintln!("{:?}", res.err());
                }
            }

            self.upd_storage();
            self.upd_filtered_entries();
        }
    }

    fn insert_entry_boxes_ui(&mut self, ui: &mut Ui, entry_deletion_queue: &mut Vec<Entry>) {
        let window_rect = ui.ctx().input(|i| i.viewport().inner_rect).unwrap();
        let screen_size = ui.ctx().input(|i| i.viewport().monitor_size)
            .unwrap(); // todo handle none

        ui.spacing_mut().item_spacing.x = 0.;

        let perfect_size = screen_size * MAGIC_FRACTION;
        let entry_box_size = egui::Vec2::from([
            perfect_size.x 
            + (ui.available_width() % perfect_size.x)
                / (ui.available_width() / (perfect_size.x).max(1.)).max(1.),
            perfect_size.y + (window_rect.max.y * 0.1)
        ]);
        // i think this is a good size?
        
        for entry in &self.selected_entries {
            let (rect, resp) 
                = ui.allocate_exact_size(entry_box_size, Sense::empty());

            let uri = format!("file:///{}",
                std::env::current_dir().unwrap()
                .join(&entry.file_path)
                .to_string_lossy().replace('\\', "/"))
                .replace("./", "");

            egui::Frame::group(ui.style())
            .stroke(Stroke {
                width: 1.,
                color: if ui.ctx().theme() == egui::Theme::Dark
                        { egui::Color32::from_rgb(0xff, 0xff, 0xff) }
                    else
                        { egui::Color32::from_rgb(0x00, 0x00, 0x00) }
            })
            .inner_margin(Margin { left: 0, right: 0, top: 0, bottom: 0})
            .outer_margin(Margin { left: 0, right: 0, top: 5, bottom: 5})
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.put(
                        rect,
                        egui::Image::from_uri(uri)
                    );

                    ui.horizontal(|ui| {
                        let copy_button = ui.button("📋");

                        if copy_button.clicked() {
                            Self::copy_image_to_clipboard(&mut self.arboard_ctx, &entry);
                        }

                        let delete_button = ui.button("🗑");

                        if delete_button.clicked() {
                            entry_deletion_queue.push(entry.clone());
                        }
                    });
                });
            });
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        /* println!("{:?}\n\n", self.selected_entries); */

        let mut entry_deletion_queue: Vec<Entry> = vec![]; // storing ref creates borrow hell

        if !self.started {
            self.started = true;
            self.upd_storage();
            self.upd_filtered_entries();
        }

        // top bar
        egui::Panel::top("search").show(ui, |ui| {
            ui.horizontal(|ui| {
                self.search_bar_ui(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    self.theme_change_ui(ui);

                    self.reload_list_ui(ui);

                    self.add_entry_ui(ui);
                });
            });
        });

        // album
        egui::CentralPanel::default()
            /* .frame(egui::Frame::new().inner_margin(0.)) */
            .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui: &mut egui::Ui| {
                    self.insert_entry_boxes_ui(ui, &mut entry_deletion_queue);
                });
            });
        });

        for entry_to_delete in entry_deletion_queue {
            self.entries.retain(|e| *e != entry_to_delete);
            self.selected_entries.retain(|e| *e != entry_to_delete);
        }
    }
}