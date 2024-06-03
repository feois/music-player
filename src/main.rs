#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Music Player", 
        Default::default(), 
        Box::new(|cc| {
            Box::new(App::default())
        })
    )
}

struct App {}

impl Default for App {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut text = String::new();
            let mut num = 0;
            
            ui.heading("Heading");
            ui.horizontal(|ui| {
                let label = ui.label("Label: ");
                ui.text_edit_singleline(&mut text).labelled_by(label.id);
            });
            ui.add(egui::Slider::new(&mut num, 0..=100).text("text"));
            if ui.button("button").clicked() {
                println!("clicked");
            }
            ui.label(format!("text '{}', num {}", text, num));
        });
    }
}