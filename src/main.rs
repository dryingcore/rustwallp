mod add;
mod apply;
mod barcap;
mod i3;
mod library;
mod media;
mod paths;
mod selection;
mod test_session;
mod thumbs;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Wallpapers")
            .with_app_id("rustwallp")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("rustwallp", options, Box::new(|cc| Ok(Box::new(ui::App::new(cc)))))
}
