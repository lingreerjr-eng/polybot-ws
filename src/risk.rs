use std::collections::HashMap;

#[derive(Default)]
pub struct RiskState {
    pub realized_pnl: f64,
    pub inventory: HashMap<String, f64>,
}

pub struct RiskLimits {
    pub max_daily_loss: f64,
    pub max_inventory_per_token: f64,
}

impl RiskState {
    pub fn can_trade(&self, limits: &RiskLimits, token_a: &str, token_b: &str) -> bool {
        if self.realized_pnl <= -limits.max_daily_loss {
            return false;
        }

        let ia = self.inventory.get(token_a).copied().unwrap_or(0.0);
        let ib = self.inventory.get(token_b).copied().unwrap_or(0.0);

        ia.abs() <= limits.max_inventory_per_token
            && ib.abs() <= limits.max_inventory_per_token
    }

    pub fn apply_fill(&mut self, token: &str, delta: f64) {
        *self.inventory.entry(token.into()).or_insert(0.0) += delta;
    }

    pub fn apply_pnl(&mut self, pnl: f64) {
        self.realized_pnl += pnl;
    }
}
