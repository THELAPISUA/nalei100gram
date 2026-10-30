use crate::logic::integrations::traits::AccauntIntegration;

pub struct Account {
    pub id: i64,
    pub integration: Box<dyn AccauntIntegration>,
}
impl Account {
    pub fn new(id: i64, integration: Box<dyn AccauntIntegration>) -> Self {
        Self { id, integration }
    }
}
