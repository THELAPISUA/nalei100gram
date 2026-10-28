use crate::logic::integrations::traits::AccauntIntegration;

pub struct Account {
    pub name: String,
    pub id: i64,
    pub integration: Box<dyn AccauntIntegration>,
}
impl Account {
    pub fn new(id: i64, integration: Box<dyn AccauntIntegration>) -> Self {
        Self {
            name: String::new(),
            id,
            integration,
        }
    }
}
