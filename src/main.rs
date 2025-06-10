use eframe::egui::{self};
use interference_generator::app::App;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 520.0]),
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    #[allow(clippy::empty_loop)]
    std::thread::spawn(move || rt.block_on(async { loop {} }));

    eframe::run_native(
        "Interference generator",
        native_options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}
