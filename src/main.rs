#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Music Player", 
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder { maximized: Some(true), ..Default::default() },
            ..Default::default()
        }, 
        Box::new(|cc| {
            Box::new(App {
                text: String::new(),
                num: 0,
            })
        })
    )
}

struct App {
    text: String,
    num: u8,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Heading");
            ui.horizontal(|ui| {
                let label = ui.label("Label: ");
                ui.text_edit_singleline(&mut self.text).labelled_by(label.id);
            });
            ui.add(egui::Slider::new(&mut self.num, 0..=100).text("text"));
            if ui.button("button").clicked() {
                self.num += 1;
            }
            ui.label(format!("text '{}', num {}", self.text, self.num));
        });
    }
}