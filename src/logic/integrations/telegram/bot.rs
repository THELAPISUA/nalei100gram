use teloxide::Bot;

use crate::logic::integrations::traits::AccauntIntegration;

struct TelegramBot {
    bot: Bot,
}
impl TelegramBot {
    fn new(token: &str) -> Self {
        Self {
            bot: Bot::new(token),
        }
    }
}
impl AccauntIntegration for TelegramBot {}
