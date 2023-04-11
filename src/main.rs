use bh_diver::app::BHDiver;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Black Hole Diver",
        options,
        Box::new(|cc| Box::new(BHDiver::new(cc))),
    )
    .unwrap();
}
