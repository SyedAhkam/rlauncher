#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{path::{PathBuf, Path}, sync::Arc};

use sqlx::Row;
use tauri::{
    CustomMenuItem, GlobalShortcutManager, Manager, PhysicalSize, Size, SystemTray,
    SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};
use xdg::BaseDirectories;
use log::*;
use lazy_static::lazy_static;
use gtk::prelude::GtkWindowExt;
use anyhow::Result;

const CACHE_DIR: &str = "rlauncher-cache";
const APPLICATIONS_DIR: &str = "/usr/share/applications";

lazy_static! {
    static ref XDG_DIRS: BaseDirectories = BaseDirectories::with_prefix(CACHE_DIR).unwrap();
}

struct Application {
    name: String,
    exec: String,
    comment: Option<String>,
    categories: Option<String>,
    keywords: Option<String>,
    use_terminal: bool
}

impl Application {
    fn from_desktop(path: &Path) -> Result<Self> {
        let desktop_file = freedesktop_entry_parser::parse_entry(path).unwrap();

        let desktop_entry_section = desktop_file.section("Desktop Entry");
        
        let exec = desktop_entry_section.attr("Exec");
        if !exec.is_some() {
            warn!("No Exec field in {}", path.display());
            
            return Err(anyhow::anyhow!("No Exec field in {}", path.display()));
        }

        Ok(Self {
            name: desktop_entry_section.attr("Name").unwrap().to_string(),
            exec: exec.unwrap().to_string(),
            comment: desktop_entry_section.attr("Comment").map(|s| s.to_string()),
            categories: desktop_entry_section.attr("Categories").map(|s| s.to_string()),
            keywords: desktop_entry_section.attr("Keywords").map(|s| s.to_string()),
            use_terminal: desktop_entry_section.attr("Terminal").unwrap_or("false") == "true"
        })
    }
}

#[derive(Debug)]
struct CacheManager {
    cache_dir: PathBuf,
    application_db: sqlx::SqlitePool,
}

impl CacheManager {
    async fn new() -> Self {
        let cache_dir = (*XDG_DIRS).get_cache_home();

        // Make sure the cache directory exists
        if !std::path::Path::new(&cache_dir).exists() {
            std::fs::create_dir_all(&cache_dir).unwrap();
            info!("Created cache directory at {}", cache_dir.display());
        }

        // Make sure the applications database exists
        let application_db_path = cache_dir.join("applications.db");
        if !application_db_path.exists() {
            std::fs::File::create(&application_db_path).unwrap();
            info!("Created applications database at {}", application_db_path.display());
        }

        // Connect to the applications database
        let application_db = sqlx::SqlitePool::connect(&format!(
            "sqlite://{}",
            application_db_path.display()
        )).await.unwrap();

        Self {
            cache_dir,
            application_db,
        }
    }

    // TODO
    async fn does_exist_by_name(&self, db: &sqlx::SqlitePool, name: &str) -> bool {
        // let rows = sqlx::query("SELECT COUNT(*) FROM applications WHERE name = ?")
        //     .bind(name)
        //     .fetch_one(db)
        //     .await
        //     .unwrap();
        
        // rows.get::<usize, i64>(0) > 0

        false
    }

    async fn add_application(&self, application: Application) {
        match sqlx::query(
            "INSERT INTO applications (name, exec, comment, categories, keywords, use_terminal) VALUES (?, ?, ?, ?, ?, ?)"
        )
            .bind(&application.name)
            .bind(&application.exec)
            .bind(&application.comment)
            .bind(&application.categories)
            .bind(&application.keywords)
            .bind(application.use_terminal)
            .execute(&self.application_db)
            .await {
                Ok(_) => info!("Added application {} to database", application.name),
                Err(e) => error!("Failed to add application {} to database: {}", application.name, e)
            }
    }

    async fn add_if_not_exists(&self, application: Application) {
        if self.does_exist_by_name(&self.application_db, &application.name).await { return }

        println!("Adding application {}", application.name);
        self.add_application(application).await;
    }

    async fn ensure_tables_exist(&self) {
        sqlx::query("CREATE TABLE IF NOT EXISTS applications (
            name TEXT NOT NULL PRIMARY KEY,
            exec TEXT NOT NULL,
            comment TEXT,
            categories TEXT,
            keywords TEXT,
            use_terminal BOOLEAN NOT NULL,
            UNIQUE (name)
        )").execute(&self.application_db).await.unwrap();
    }

    async fn rebuild_cache(&self) -> Result<()> {
        println!("Cache dir: {}", self.cache_dir.display());

        // Make sure all the necessary database tables exist
        self.ensure_tables_exist().await;

        // Get all the desktop files in the applications directory
        for entry in std::fs::read_dir(APPLICATIONS_DIR).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            
            if !path.is_file() { continue };
            if !path.extension().unwrap().eq("desktop") { continue };

            
            if let Ok(application) = Application::from_desktop(&path) {
                self.add_if_not_exists(application).await;
            };
        }

        Ok(())
    }
}

#[tauri::command]
async fn rebuild_cache(cache_manager: tauri::State<'_, CacheManager>) -> Result<(), ()> {
    cache_manager.rebuild_cache().await.unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let sys_tray = SystemTray::new().with_menu(
        SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("focus", "Focus"))
            .add_item(CustomMenuItem::new("rebuild-cache", "Rebuild Cache"))
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new("exit", "Exit"))
    );

    tauri::Builder::default()
        .setup(|app| {
            // Setup logging
            simple_logger::SimpleLogger::new()
                .init()
                .unwrap();

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

            // Stick the window in all workspaces
            window.gtk_window()?.stick();

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
                    let app_cloned = app.clone();
                    tauri::async_runtime::spawn(async move {
                        app_cloned.state::<CacheManager>().rebuild_cache().await.unwrap();
                    });
                }
                _ => {}
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
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![rebuild_cache])
        .manage(CacheManager::new().await)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

        Ok(())
}
