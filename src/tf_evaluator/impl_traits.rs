use super::TfEvaluator;

impl std::fmt::Debug for TfEvaluator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("TfEvaluator");
        f.field("date", &self.date)
            .field("future", &self.future)
            .field("bond", &self.bond)
            .field("capital_rate", &self.capital_rate)
            .field("reinvest_rate", &self.reinvest_rate);
        if let Some(accrued_interest) = self.accrued_interest {
            f.field("accrued_interest", &accrued_interest);
        }
        if self.deliver_accrued_interest.is_some() {
            f.field("deliver_accrued_interest", &self.deliver_accrued_interest);
        }
        if let Some(cf) = self.cf {
            f.field("cf", &cf);
        }
        if let Some(dirty_price) = self.dirty_price {
            f.field("dirty_price", &dirty_price);
        }
        if let Some(clean_price) = self.clean_price {
            f.field("clean_price", &clean_price);
        }
        if let Some(future_dirty_price) = self.future_dirty_price {
            f.field("future_dirty_price", &future_dirty_price);
        }
        if let Some(deliver_cost) = self.deliver_cost {
            f.field("deliver_cost", &deliver_cost);
        }
        if let Some(basis_spread) = self.basis_spread {
            f.field("basis_spread", &basis_spread);
        }
        if let Some(f_b_spread) = self.f_b_spread {
            f.field("f_b_spread", &f_b_spread);
        }
        if let Some(carry) = self.carry {
            f.field("carry", &carry);
        }
        if let Some(net_basis_spread) = self.net_basis_spread {
            f.field("net_basis_spread", &net_basis_spread);
        }
        if let Some(duration) = self.duration {
            f.field("duration", &duration);
        }
        if let Some(irr) = self.irr {
            f.field("irr", &irr);
        }
        if let Some(deliver_date) = self.deliver_date {
            f.field("deliver_date", &deliver_date);
        }
        if let Some(cp_dates) = self.cp_dates {
            f.field("cp_dates", &cp_dates);
        }
        if let Some(deliver_cp_dates) = self.deliver_cp_dates {
            f.field("deliver_cp_dates", &deliver_cp_dates);
        }
        if let Some(remain_cp_num) = self.remain_cp_num {
            f.field("remain_cp_num", &remain_cp_num);
        }
        if let Some(remain_days_to_deliver) = self.remain_days_to_deliver {
            f.field("remain_days_to_deliver", &remain_days_to_deliver);
        }
        if let Some(remain_cp_to_deliver) = self.remain_cp_to_deliver {
            f.field("remain_cp_to_deliver", &remain_cp_to_deliver);
        }
        if let Some(remain_cp_to_deliver_wm) = self.remain_cp_to_deliver_wm {
            f.field("remain_cp_to_deliver_wm", &remain_cp_to_deliver_wm);
        }
        if let Some(future_ytm) = self.future_ytm {
            f.field("future_ytm", &future_ytm);
        }
        f.finish()
    }
}
