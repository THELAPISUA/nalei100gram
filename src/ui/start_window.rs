use std::sync::Arc;

use crate::logic::backend::Backend;
use crate::logic::db::Db;
use crate::logic::integrations::manager::account::Account;
use crate::logic::integrations::manager::manager::AccountManager;
use crate::logic::integrations::telegram::bot::TelegramBot;
use crate::ui::app::GramApp;
use crate::ui::theme::{AppTheme, setup_fonts};
use crate::ui::tray::init_tray;
use crate::utils::settings::save_settings;
use crate::utils::types::{Settings, User};
use egui_material_icons::icons::{ICON_VISIBILITY, ICON_VISIBILITY_OFF};
use log::info;
use tokio::sync::mpsc;

pub struct StartWindow {
    pub inner: Option<GramApp>,
    pub runtime: tokio::runtime::Handle,
    pub tray: Option<tray_icon::TrayIcon>,
    pub started: bool,
    pub theme: AppTheme,
    pub bot_token: String,
    pub show_token: bool,
    pub error: Option<String>,
    pub(crate) settings: Settings,
}

impl StartWindow {
    pub fn start_backend(&mut self, token: String) {
        if self.started {
            return;
        }

        let database = self.runtime.block_on(async {
            let database = Db::connect().await;
            database.init().await;
            database
        });

        let chats = self.runtime.block_on(database.load_chats());

        info!("Chats loaded: {}", chats.len());

        let (tx_bot, rx_ui) = mpsc::channel(32);
        let (tx_ui, rx_bot) = mpsc::channel(32);

        let mut account_manager = AccountManager::new();
        let accounts = vec![(0, String::new())];
        account_manager.add_account(Account::new(
            accounts[0].0,
            Box::new(TelegramBot::new(&token)),
        ));

        // Додай token до конструктора Backend.
        let backend = Backend::new(tx_bot, rx_bot, database, Arc::new(account_manager));

        self.runtime.spawn(async move {
            backend.tick().await;
        });

        self.inner = Some(GramApp::new(
            tx_ui,
            rx_ui,
            accounts,
            chats,
            self.settings.clone(),
        ));
        self.started = true;

        info!("Backend started");
    }
}

impl eframe::App for StartWindow {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame) {
        use eframe::egui::{self, Align, Layout, RichText, TextEdit};

        let ctx = ui.ctx().clone();
        setup_fonts(&ctx);

        if self.tray.is_none() {
            self.tray = Some(init_tray(ctx.clone()));
            info!("Tray initialized");
        }

        if !self.started {
            let mut start_clicked = false;

            egui::CentralPanel::default().show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space(65.0);

                    ui.label(
                        RichText::new("N100G")
                            .size(16.0)
                            .strong()
                            .color(self.theme.accent),
                    );

                    ui.add_space(12.0);

                    ui.heading(
                        RichText::new("Ласкаво просимо")
                            .size(30.0)
                            .color(self.theme.text),
                    );

                    ui.add_space(8.0);

                    ui.label(
                        RichText::new("Підключи свого Telegram-бота")
                            .size(14.0)
                            .color(self.theme.muted),
                    );

                    ui.add_space(30.0);

                    egui::Frame::default()
                        .fill(self.theme.surface)
                        .inner_margin(egui::Margin::same(24))
                        .show(ui, |ui| {
                            ui.set_width(340.0);

                            ui.label(RichText::new("Токен бота").strong().color(self.theme.text));

                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                let token_edit = TextEdit::singleline(&mut self.bot_token)
                                    .password(!self.show_token)
                                    .hint_text("Токен можна отримати в адміна")
                                    .desired_width(260.0);

                                ui.add_sized([260.0, 38.0], token_edit);

                                if ui
                                    .button(if self.show_token {
                                        ICON_VISIBILITY
                                    } else {
                                        ICON_VISIBILITY_OFF
                                    })
                                    .clicked()
                                {
                                    self.show_token = !self.show_token;
                                }
                            });

                            ui.add_space(8.0);

                            ui.label(
                                RichText::new("Токен потрібен для підключення бота.")
                                    .size(12.0)
                                    .color(self.theme.muted),
                            );

                            if let Some(error) = &self.error {
                                ui.add_space(8.0);

                                ui.label(
                                    RichText::new(error)
                                        .color(egui::Color32::from_rgb(255, 110, 120)),
                                );
                            }

                            ui.add_space(18.0);

                            let button = egui::Button::new(
                                RichText::new("Підключити та запустити")
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(self.theme.accent)
                            .min_size(egui::vec2(340.0, 44.0));

                            if ui.add(button).clicked() {
                                start_clicked = true;
                            }
                        });

                    ui.add_space(20.0);

                    ui.label(
                        RichText::new("Токен не показується на екрані за замовчуванням")
                            .size(11.0)
                            .color(self.theme.muted),
                    );
                });
            });

            if start_clicked {
                let token = self.bot_token.trim().to_owned();

                if token.is_empty() {
                    self.error = Some("Спочатку введи токен бота.".to_owned());
                } else {
                    self.error = None;
                    save_settings(Settings {
                        user: User {
                            token: token.clone(),
                        },
                        theme: None,
                        selected_theme: 0,
                    });
                    self.start_backend(token);
                }
            }

            return;
        }

        if let Some(app) = self.inner.as_mut() {
            eframe::App::ui(app, ui, frame);
        }
    }
}
