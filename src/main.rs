#![windows_subsystem = "windows"]

mod audio;
mod backend;
mod db;
mod tray;
mod types;
mod ui;
mod widgets;

use crate::backend::Backend;
use crate::db::Db;
use crate::tray::init_tray;
use crate::ui::GramApp;

use env_logger::{Builder, Target};
use log::{LevelFilter, info};

use std::fs::File;
use tokio::sync::mpsc;

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

    let database = rt.block_on(async {
        let database = Db::connect().await;
        database.init().await;
        database
    });

    let chats = rt.block_on(database.load_chats());

    info!("Chats loaded: {}", chats.len());

    // Backend -> UI.
    let (tx_bot, rx_ui) = mpsc::channel(32);

    // UI -> Backend.
    let (tx_ui, rx_bot) = mpsc::channel(32);

    let backend = Backend::new(tx_bot, rx_bot, database);

    let _backend_task = rt.spawn(async move {
        backend.tick().await;
    });

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

    let result = eframe::run_native(
        "Nalei100Gram",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let app = GramApp::new(tx_ui, rx_ui, chats);

            Ok(Box::new(MainApp {
                inner: app,
                tray: None,
                hide_on_start: true,
            }))
        }),
    );

    if let Err(error) = result {
        log::error!("GUI error: {error}");
    }

    info!("App stopped");
}

struct MainApp {
    inner: GramApp,
    tray: Option<tray_icon::TrayIcon>,
    hide_on_start: bool,
}

impl eframe::App for MainApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if self.tray.is_none() {
            self.tray = Some(init_tray(ctx.clone()));
            info!("Tray initialized");
        }

        if self.hide_on_start {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Visible(false));

            self.hide_on_start = false;
        }

        eframe::App::ui(&mut self.inner, ui, frame);
    }
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
