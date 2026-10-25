#![windows_subsystem = "windows"]

mod logic;
mod ui;
mod utils;

use env_logger::{Builder, Target};
use log::{LevelFilter, info};

use std::fs::File;

use crate::{
    ui::{start_window::StartWindow, theme::AppTheme},
    utils::{
        settings::{self, load_settings},
        types::{Settings, Theme, User},
    },
};

fn main() {
    let log_file = File::options()
        .create(true)
        .append(true)
        .open("app.log")
        .expect("Не удалось создать файл лога");

    Builder::new()
        .parse_default_env()
        .target(Target::Pipe(Box::new(log_file)))
        .filter_level(LevelFilter::Info)
        .filter_module("teloxide", LevelFilter::Warn)
        .filter_module("egui_wgpu", LevelFilter::Warn)
        .filter_module("egui", LevelFilter::Warn)
        .filter_module("wgpu_core", LevelFilter::Warn)
        .filter_module("wgpu_hal", LevelFilter::Warn)
        .filter_module("zbus", LevelFilter::Warn)
        .filter_module("tracing", LevelFilter::Warn)
        .filter_module("sqlx", LevelFilter::Warn)
        .init();

    std::panic::set_hook(Box::new(|info| {
        log::error!("PANIC: {info}");
    }));

    info!("App started!");

    let rt = tokio::runtime::Runtime::new().expect("Не удалось создать Tokio runtime");
    let runtime = rt.handle().clone();

    let icon = load_icon();

    let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();

    setup.instance_descriptor.backends = eframe::wgpu::Backends::VULKAN;

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_icon(icon)
            .with_drag_and_drop(true),

        renderer: eframe::Renderer::Wgpu,

        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(setup),
            ..Default::default()
        },

        ..Default::default()
    };

    let settings = match load_settings() {
        Some(s) => s,
        None => Settings {
            user: User {
                token: String::new(),
            },
            theme: None,
        },
    };

    let result = eframe::run_native(
        "Nalei100Gram",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let mut app = StartWindow {
                inner: None,
                runtime,
                tray: None,
                started: false,
                theme: AppTheme::default(),
                bot_token: settings.user.token,
                show_token: false,
                error: None,
            };

            // Якщо токен вже збережений — одразу запускаємо бекенд.
            if !app.bot_token.trim().is_empty() {
                let token = app.bot_token.clone();
                app.start_backend(token);
            }

            Ok(Box::new(app))
        }),
    );

    if let Err(error) = result {
        log::error!("GUI error: {error}");
    }

    info!("App stopped");
}

fn load_icon() -> eframe::egui::IconData {
    let bytes = include_bytes!("../assets/nalei100gramICO.png");

    let image = image::load_from_memory(bytes)
        .expect("Не вдалося завантажити icon.png")
        .into_rgba8();

    let (width, height) = image.dimensions();

    eframe::egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
