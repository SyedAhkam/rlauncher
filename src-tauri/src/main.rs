#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::{PathBuf, Path};

use tauri::{
    CustomMenuItem, GlobalShortcutManager, Manager, PhysicalSize, Size, SystemTray,
    SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};
use xdg::BaseDirectories;
use log::*;
use lazy_static::lazy_static;

const CACHE_DIR: &str = "rlauncher-cache";
const APPLICATIONS_DIR: &str = "/usr/share/applications";

lazy_static! {
    static ref XDG_DIRS: BaseDirectories = BaseDirectories::with_prefix(CACHE_DIR).unwrap();
}

#[derive(Debug)]
struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    fn new() -> Self {
        let cache_dir = (*XDG_DIRS).get_cache_home();

        // Make sure the cache directory exists
        if !std::path::Path::new(&cache_dir).exists() {
            std::fs::create_dir_all(&cache_dir).unwrap();
            warn!("Created cache directory at {}", cache_dir.display());
        }

        Self {
            cache_dir
        }
    }

    fn add_if_not_exists(&self, desktop_entry: freedesktop_entry_parser::Entry) {
        // Get the desktop entry section
        let desktop_entry_section = desktop_entry.section("Desktop Entry");

        // Get the name of application
        let name = desktop_entry_section.attr("Name").unwrap();
        

        println!("Desktop file: {:#?}", name);
    }

    fn rebuild_cache(&self) {
        println!("Cache dir: {}", self.cache_dir.display());

        for entry in std::fs::read_dir(APPLICATIONS_DIR).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            
            if !path.is_file() { continue };
            if !path.extension().unwrap().eq("desktop") { continue };

            let desktop_file = freedesktop_entry_parser::parse_entry(&path).expect("Failed to parse desktop file");

            self.add_if_not_exists(desktop_file);
        }
    }
}

#[tauri::command]
fn rebuild_cache(cache_manager: tauri::State<CacheManager>) {
    cache_manager.rebuild_cache();
}

fn main() {
    let sys_tray = SystemTray::new().with_menu(
        SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("focus", "Focus"))
            .add_item(CustomMenuItem::new("rebuild-cache", "Rebuild Cache"))
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new("exit", "Exit"))
    );

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();

            // Set window size
            window
                .set_size(Size::Physical(PhysicalSize {
                    width: 800,
                    height: 400,
                }))
                .unwrap();

            // Center window
            window.center().unwrap();

            // Register global keyboard shortcuts
            app.global_shortcut_manager()
                .register("alt+d", move || {
                    // to toggle visibility
                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                    }
                })
                .unwrap();

            // todo: handle ESC while window is focused

            Ok(())
        })
        .system_tray(sys_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { tray_id: _, id, .. } => match id.as_str() {
                "focus" => {
                    app.get_window("main").unwrap().show().unwrap();
                }
                "exit" => {
                    app.exit(0);
                },
                "rebuild-cache" => {
                    app.state::<CacheManager>().rebuild_cache();
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            // WindowEvent::Focused(false) => {
            //     // Hide the window when it loses focus
            //     event.window().hide().unwrap();
            // },
            WindowEvent::Focused(true) => {
                // Make sure the window is centered after regaining focus
                event.window().center().unwrap();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![rebuild_cache])
        .manage(CacheManager::new())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
