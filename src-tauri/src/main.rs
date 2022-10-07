#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::CustomMenuItem;
use tauri::GlobalShortcutManager;
use tauri::Manager;
use tauri::PhysicalSize;
use tauri::Size;
use tauri::SystemTray;
use tauri::SystemTrayEvent;
use tauri::SystemTrayMenu;
use tauri::SystemTrayMenuItem;
use tauri::WindowEvent;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    let sys_tray = SystemTray::new()
        .with_menu(
            SystemTrayMenu::new()
                .add_item(CustomMenuItem::new("focus", "Focus"))                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(CustomMenuItem::new("exit", "Exit"))
        );

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();

            // Set window size
            window.set_size(Size::Physical(PhysicalSize {
                width: 800,
                height: 400,
            })).unwrap();

            // Center window
            window.center().unwrap();

            // Register a global keyboard shortcut
            app.global_shortcut_manager().register("alt+d", move || {
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                }
            }).unwrap();

            Ok(())
        })
        .system_tray(sys_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { tray_id: _, id, .. } => {
                match id.as_str() {
                    "focus" => {
                        app.get_window("main").unwrap().show().unwrap();
                    },
                    "exit" => {
                        app.exit(0);
                    },
                    _ => {}
                }
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::Focused(false) => {
                // Hide the window when it loses focus
                event.window().hide().unwrap();
            },
            WindowEvent::Focused(true) => {
                // Make sure the window is centered after regaining focus
                event.window().center().unwrap();
            },
            _ => {}

        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
