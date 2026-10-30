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
use crate::utils::types::{AccountPlatform, Settings, User};
use egui_material_icons::icons::{ICON_VISIBILITY, ICON_VISIBILITY_OFF};
use log::info;
use tokio::sync::mpsc;

pub struct StartWindow {
    pub inner: Option<GramApp>,
    pub runtime: tokio::runtime::Handle,
    pub tray: Option<tray_icon::TrayIcon>,
    pub started: bool,
    pub theme: AppTheme,
    pub show_token: bool,
    pub error: Option<String>,

    pub(crate) settings: Option<Settings>,

    pub bot_token: String,
    pub accounts: Vec<User>,
    pub selected_platform: AccountPlatform,
}

impl StartWindow {
    fn add_account(&mut self) {
        let token = self.bot_token.trim().to_owned();

        if token.is_empty() {
            self.error = Some("Введіть токен акаунта.".to_owned());
            return;
        }

        if self
            .accounts
            .iter()
            .any(|account| account.token == token && account.platform == self.selected_platform)
        {
            self.error = Some("Цей акаунт вже доданий.".to_owned());
            return;
        }

        self.accounts.push(User {
            token,
            platform: self.selected_platform.clone(),
        });

        self.bot_token.clear();
        self.error = None;
        self.show_token = false;
    }

    fn remove_account(&mut self, index: usize) {
        if index < self.accounts.len() {
            self.accounts.remove(index);
        }
    }

    pub fn start_backend(&mut self) {
        if self.started || self.accounts.is_empty() {
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

        let (rx, mut account_manager) = AccountManager::new();

        let mut app_accounts = Vec::new();

        for (id, user) in self.accounts.iter().enumerate() {
            let id = id as i64;

            match &user.platform {
                AccountPlatform::Telegram => {
                    account_manager
                        .add_account(Account::new(id, Box::new(TelegramBot::new(&user.token))));

                    app_accounts.push((id, format!("Telegram {}", id + 1)));
                }

                AccountPlatform::Discord => {
                    // TODO: Discord integration
                    app_accounts.push((id, format!("Discord {}", id + 1)));

                    info!("Discord account {} saved, integration unavailable", id);
                }
            }
        }

        let settings = Settings {
            users: self.accounts.clone(),
            selected_theme: self
                .settings
                .as_ref()
                .map(|s| s.selected_theme)
                .unwrap_or(0),
        };

        save_settings(settings.clone());
        self.settings = Some(settings.clone());

        let backend = Backend::new(tx_bot, rx_bot, database, Arc::new(account_manager), rx);

        self.runtime.spawn(async move {
            backend.tick().await;
        });

        self.inner = Some(GramApp::new(tx_ui, rx_ui, app_accounts, chats, settings));

        self.started = true;

        info!("Backend started");
    }
}
impl eframe::App for StartWindow {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame) {
        use eframe::egui::{self, Align, Color32, Layout, RichText, TextEdit};

        let ctx = ui.ctx().clone();
        setup_fonts(&ctx);

        if self.tray.is_none() {
            self.tray = Some(init_tray(ctx.clone()));
            info!("Tray initialized");
        }

        if !self.started {
            let mut add_clicked = false;
            let mut continue_clicked = false;
            let mut remove_index = None;

            egui::CentralPanel::default().show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space(40.0);

                    // Logo
                    ui.label(
                        RichText::new("N100G")
                            .size(16.0)
                            .strong()
                            .color(self.theme.accent),
                    );

                    ui.add_space(12.0);

                    // Header
                    ui.heading(
                        RichText::new("Додайте акаунт")
                            .size(28.0)
                            .color(self.theme.text),
                    );

                    ui.add_space(8.0);

                    ui.label(
                        RichText::new("Підключіть свої месенджери до Nalei100Gram")
                            .size(13.0)
                            .color(self.theme.muted),
                    );

                    ui.add_space(25.0);

                    egui::Frame::default()
                        .fill(self.theme.surface)
                        .inner_margin(egui::Margin::same(22))
                        .show(ui, |ui| {
                            ui.set_width(360.0);

                            // Platform selection
                            ui.label(RichText::new("Платформа").strong().color(self.theme.text));

                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.selected_platform,
                                    AccountPlatform::Telegram,
                                    RichText::new("Telegram").color(self.theme.text),
                                );

                                ui.add_space(20.0);

                                ui.radio_value(
                                    &mut self.selected_platform,
                                    AccountPlatform::Discord,
                                    RichText::new("Discord").color(self.theme.text),
                                );
                            });

                            ui.add_space(18.0);

                            // Token input
                            ui.label(
                                RichText::new("Токен доступу")
                                    .strong()
                                    .color(self.theme.text),
                            );

                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                let token_edit = TextEdit::singleline(&mut self.bot_token)
                                    .password(!self.show_token)
                                    .hint_text("Введіть токен")
                                    .desired_width(285.0);

                                ui.add_sized([285.0, 38.0], token_edit);

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

                            let description = match self.selected_platform {
                                AccountPlatform::Telegram => "Токен Telegram-бота.",
                                AccountPlatform::Discord => {
                                    "Підтримка Discord поки не реалізована."
                                }
                            };

                            ui.label(
                                RichText::new(description)
                                    .size(11.0)
                                    .color(self.theme.muted),
                            );

                            if let Some(error) = &self.error {
                                ui.add_space(10.0);

                                ui.label(RichText::new(error));
                            }

                            ui.add_space(16.0);

                            let add_button =
                                egui::Button::new(RichText::new("Додати акаунт").strong())
                                    .fill(self.theme.accent)
                                    .min_size(egui::vec2(360.0, 42.0));

                            if ui.add(add_button).clicked() {
                                add_clicked = true;
                            }
                        });

                    // Added accounts
                    if !self.accounts.is_empty() {
                        ui.add_space(20.0);

                        ui.set_width(405.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Додані акаунти")
                                    .strong()
                                    .color(self.theme.text),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("{}", self.accounts.len()))
                                        .color(self.theme.muted),
                                );
                            });
                        });

                        ui.add_space(8.0);

                        egui::Frame::default()
                            .fill(self.theme.surface)
                            .inner_margin(egui::Margin::same(12))
                            .show(ui, |ui| {
                                for (index, account) in self.accounts.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        let platform = match account.platform {
                                            AccountPlatform::Telegram => "Telegram",
                                            AccountPlatform::Discord => "Discord",
                                        };

                                        ui.label(RichText::new("●").color(self.theme.accent));

                                        ui.label(
                                            RichText::new(platform).strong().color(self.theme.text),
                                        );

                                        ui.label(
                                            RichText::new("Додано")
                                                .size(11.0)
                                                .color(self.theme.muted),
                                        );

                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if ui.small_button("✕").clicked() {
                                                    remove_index = Some(index);
                                                }
                                            },
                                        );
                                    });

                                    if index + 1 < self.accounts.len() {
                                        ui.separator();
                                    }
                                }
                            });

                        ui.add_space(15.0);

                        ui.horizontal(|ui| {
                            if ui.button("＋ Додати ще").clicked() {
                                self.bot_token.clear();
                                self.error = None;
                                self.show_token = false;
                            }

                            ui.add_space(10.0);

                            let continue_button =
                                egui::Button::new(RichText::new("Продовжити").strong())
                                    .fill(self.theme.accent)
                                    .min_size(egui::vec2(220.0, 38.0));

                            if ui.add(continue_button).clicked() {
                                continue_clicked = true;
                            }
                        });
                    }

                    ui.add_space(20.0);

                    ui.label(
                        RichText::new("Дані акаунтів зберігаються локально")
                            .size(11.0)
                            .color(self.theme.muted),
                    );
                });
            });

            // Process actions outside UI closures
            if let Some(index) = remove_index {
                self.remove_account(index);
            }

            if add_clicked {
                self.add_account();
            }

            if continue_clicked {
                self.start_backend();
            }

            return;
        }

        if let Some(app) = self.inner.as_mut() {
            eframe::App::ui(app, ui, frame);
        }
    }
}
