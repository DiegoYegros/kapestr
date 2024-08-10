use crate::config::APP_ID;
use gio::glib::set_application_name;
use gtk4::gio;
use std::path::PathBuf;
pub fn app_rel_path(dir: &str) -> std::path::PathBuf {
    let root_dir = std::env::current_exe()
        .map(|p| p.parent().unwrap().parent().unwrap().to_path_buf())
        .unwrap();
    root_dir.join(dir)
}
pub fn init_data_dir() {
    let datadir = app_rel_path("share");
    let xdg_data_dirs: Vec<PathBuf> = match std::env::var("XDG_DATA_DIRS") {
        Ok(dirs) => std::env::split_paths(&dirs).collect(),
        Err(_) => vec![],
    };
    if !xdg_data_dirs.iter().any(|d| d == &datadir) {
        let mut xdg_final_dirs = vec![datadir];
        xdg_final_dirs.extend(xdg_data_dirs);
        let xdg_data_dir = std::env::join_paths(&xdg_final_dirs).unwrap();
        std::env::set_var("XDG_DATA_DIRS", xdg_data_dir);
    }
}

pub fn init_glib() {
    set_application_name(APP_ID);
}

pub fn init_gio_resources() {
    let resource_file = app_rel_path("share/kapestr/kapestr.gresource");
    let res = gio::Resource::load(resource_file).expect("Could not load gresource file");
    gio::resources_register(&res);
}

pub fn init_resources() {
    init_data_dir();
    init_gio_resources();
    init_glib();
}
