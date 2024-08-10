use self::app::KapestrApplication;
use adw::prelude::*;
mod app;
mod config;
mod services;
mod ui;
mod widgets;
mod win;

pub fn main() {
    ui::init_resources();
    let application = KapestrApplication::new();
    application.run();
}
