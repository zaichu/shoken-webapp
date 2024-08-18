use super::{
    dividend_list::lib::DividendListManager, profit_and_loss::lib::ProfitAndLossManager,
    templete::TemplateManager,
};

pub fn create_factory(id: &str) -> Box<dyn TemplateManager> {
    match id {
        "dividend" => Box::new(DividendListManager::new()),
        "profit-loss" => Box::new(ProfitAndLossManager::new()),
        _ => panic!("Unknown id"),
    }
}
