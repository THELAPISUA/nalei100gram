use crate::logic::integrations::traits::AccauntIntegration;

pub struct AccountManager {
    accounts: Vec<Box<dyn AccauntIntegration>>,
}
impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
        }
    }
    pub fn add_account(&mut self, account: Box<dyn AccauntIntegration>) {
        self.accounts.push(account);
    }
}
