use log::info;

use tray_icon::{
    MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};

#[derive(Debug)]
pub enum TrayCommand {
    Show,
    Hide,
    Exit,
}

fn load_tray_icon() -> tray_icon::Icon {
    let bytes = include_bytes!("../assets/nalei100gramICO.png");

    let image = image::load_from_memory(bytes)
        .expect("Не вдалося завантажити icon.png")
        .into_rgba8();

    let (width, height) = image.dimensions();

    tray_icon::Icon::from_rgba(image.into_raw(), width, height)
        .expect("Не вдалося створити tray icon")
}

pub fn init_tray(ctx: eframe::egui::Context) -> TrayIcon {
    let menu = Menu::new();

    let open = MenuItem::new("Open", true, None);
    let hide = MenuItem::new("Hide", true, None);
    let exit = MenuItem::new("Exit", true, None);

    let open_id = open.id().as_ref().to_owned();
    let hide_id = hide.id().as_ref().to_owned();
    let exit_id = exit.id().as_ref().to_owned();

    menu.append(&open).unwrap();
    menu.append(&hide).unwrap();
    menu.append(&exit).unwrap();

    let menu_ctx = ctx.clone();

    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        info!("MENU EVENT: {:?}", event.id);

        let command = if event.id.as_ref() == open_id.as_str() {
            Some(TrayCommand::Show)
        } else if event.id.as_ref() == hide_id.as_str() {
            Some(TrayCommand::Hide)
        } else if event.id.as_ref() == exit_id.as_str() {
            Some(TrayCommand::Exit)
        } else {
            None
        };

        if let Some(command) = command {
            handle_command(&menu_ctx, command);
        }
    }));

    TrayIconEvent::set_event_handler(Some(move |event| {
        let should_show = matches!(
            event,
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            }
        );

        if should_show {
            handle_command(&ctx, TrayCommand::Show);
        }
    }));

    let tray = TrayIconBuilder::new()
        .with_icon(load_tray_icon())
        .with_menu(Box::new(menu))
        .with_tooltip("Nalei100Gram")
        .build()
        .expect("Не удалось создать tray icon");

    info!("Tray icon created");

    tray
}

fn handle_command(ctx: &eframe::egui::Context, command: TrayCommand) {
    use eframe::egui::ViewportCommand;

    match command {
        TrayCommand::Show => {
            info!("Showing window");

            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }

        TrayCommand::Hide => {
            info!("Hiding window");

            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        }

        TrayCommand::Exit => {
            info!("Exiting application");

            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        }
    }

    ctx.request_repaint();
}
