mod evaluator;
mod impl_traits;
mod update_with_new_info;

pub use evaluator::TfEvaluator;

#[cfg(test)]
mod tests {
    use crate::*;
    use chrono::NaiveDate;
    fn assert_approx_eq(a: Option<f64>, b: f64) {
        assert!((a.unwrap() - b).abs() < 1e-10, "{} != {}", a.unwrap(), b);
    }

    fn get_evaluator() -> TfEvaluator {
        let json_str = r#"
        {
            "bond_code": "240006.IB",
            "mkt": "IB",
            "abbr": "24附息国债06",
            "par_value": 100.0,
            "cp_type": "Coupon_Bear",
            "interest_type": "Fixed",
            "cp_rate_1st": 0.0228,
            "base_rate": null,
            "rate_spread": null,
            "inst_freq": 1,
            "carry_date": "2024-03-25",
            "maturity_date": "2031-03-25",
            "day_count": "ACT/ACT"
        }
        "#;

        let bond: Bond = serde_json::from_str(json_str).unwrap();
        let future = Future::new("T2409");
        let future_price = FuturePrice {
            future: future.into(),
            price: 105.5,
        };
        let bond_ytm = BondYtm::new(bond, 2.115 * 0.01);
        TfEvaluator {
            date: NaiveDate::from_ymd_opt(2024, 8, 12).unwrap(),
            future: future_price,
            bond: bond_ytm,
            capital_rate: 1.9 / 100.,
            ..Default::default()
        }
    }

    #[test]
    fn test_calc_all() {
        let evaluator = get_evaluator().calc_all().unwrap();
        assert!(evaluator.is_deliverable().unwrap());
        assert_approx_eq(evaluator.clean_price, 101.00322640481077);
        assert_approx_eq(evaluator.dirty_price, 101.87774695275598);
        assert_approx_eq(evaluator.accrued_interest, 0.8745205479452056);
        assert_approx_eq(evaluator.duration, 6.040420842016215);
        assert_approx_eq(evaluator.cf, 0.958);
        // 期货的缴款日由于是中秋节，所以会顺延到下一个交易日，计算结果与原本有所不同
        assert_approx_eq(evaluator.deliver_accrued_interest, 1.1118904);
        assert_approx_eq(evaluator.deliver_cost, 101.87774695275598);
        assert_approx_eq(evaluator.future_dirty_price, 102.1808904);
        assert_approx_eq(evaluator.f_b_spread, 0.3031434472440253);
        assert_approx_eq(evaluator.basis_spread, -0.06577359518922776);
        assert_approx_eq(evaluator.net_basis_spread, -0.1016208902580227);
        assert_approx_eq(evaluator.carry, 0.03584729506879494);
        assert_approx_eq(evaluator.irr, 0.028581046130913425);
        assert_approx_eq(evaluator.future_ytm, 0.021016611992962765);
    }

    fn get_evaluator_without_cp_before_deliver_date() -> TfEvaluator {
        let bond_str = r#"
            {
                "bond_code": "230026.IB",
                "mkt": "IB",
                "abbr": "23附息国债26",
                "par_value": 100.0,
                "cp_type": "Coupon_Bear",
                "interest_type": "Fixed",
                "cp_rate_1st": 0.0267,
                "base_rate": null,
                "rate_spread": null,
                "inst_freq": 2,
                "carry_date": "2023-11-25",
                "maturity_date": "2033-11-25",
                "day_count": "ACT/ACT"
            }
        "#;
        let bond: Bond = serde_json::from_str(bond_str).unwrap();
        get_evaluator().calc_all().unwrap().update_with_new_info(
            NaiveDate::from_ymd_opt(2024, 2, 20).unwrap(),
            ("T2403", 105.5),
            (bond, 0.0267),
            0.019,
            None,
        )
    }

    #[test]
    fn test_without_cp_before_deliver_date_calc() {
        let evaluator = get_evaluator_without_cp_before_deliver_date()
            .calc_all()
            .unwrap();
        assert!(evaluator.is_deliverable().unwrap());
        assert_approx_eq(evaluator.accrued_interest, 0.6381593406593407);
        assert_approx_eq(evaluator.dirty_price, 100.63595079737546);
        assert_approx_eq(evaluator.clean_price, 99.99779145671612);
        assert_approx_eq(evaluator.duration, 8.48901852420599);
        assert_approx_eq(evaluator.cf, 0.9725);
        assert_approx_eq(evaluator.basis_spread, -2.6009585432838946);
        assert_approx_eq(evaluator.carry, 0.04402820079777488);
        assert_approx_eq(evaluator.net_basis_spread, -2.6449867440816694);
        assert_approx_eq(evaluator.f_b_spread, 2.754997002624549);
        assert_approx_eq(evaluator.deliver_accrued_interest, 0.7921978);
        assert_approx_eq(evaluator.deliver_cost, 100.63595079737546);
        assert_approx_eq(evaluator.future_dirty_price, 103.3909478);
        assert_approx_eq(evaluator.irr, 0.4758187440261421);
        assert_approx_eq(evaluator.future_ytm, 0.023684304298184574);
    }

    #[test]
    fn test_2_inst_freq_with_cp_before_deliver_date() {
        let evaluator = get_evaluator_without_cp_before_deliver_date()
            .calc_all()
            .unwrap();
        let bond_ytm = evaluator.bond.clone();
        let evaluator = evaluator
            .update_with_new_info(
                NaiveDate::from_ymd_opt(2024, 2, 20).unwrap(),
                ("T2406", 103.58),
                bond_ytm,
                0.019,
                None,
            )
            .calc_all()
            .unwrap();
        assert!(evaluator.is_deliverable().unwrap());
        assert_approx_eq(evaluator.accrued_interest, 0.6381593406593407);
        assert_approx_eq(evaluator.dirty_price, 100.63595079737546);
        assert_approx_eq(evaluator.clean_price, 99.99779145671612);
        assert_approx_eq(evaluator.duration, 8.48901852420599);
        assert_approx_eq(evaluator.cf, 0.9731);
        assert_approx_eq(evaluator.basis_spread, -0.795906543283877);
        assert_approx_eq(evaluator.carry, 0.24924742988075288);
        assert_approx_eq(evaluator.net_basis_spread, -1.0451539731646298);
        assert_approx_eq(evaluator.f_b_spread, 1.666877602624524);
        assert_approx_eq(evaluator.deliver_accrued_interest, 0.1741304);
        assert_approx_eq(evaluator.deliver_cost, 99.30095079737546);
        assert_approx_eq(evaluator.future_dirty_price, 100.96782839999999);
        assert_approx_eq(evaluator.irr, 0.050940117037820136);
        assert_approx_eq(evaluator.future_ytm, 0.025745910924140546);
    }

    #[test]
    fn test_2_inst_freq_with_2cps_before_deliver_date() {
        let evaluator = get_evaluator_without_cp_before_deliver_date()
            .calc_all()
            .unwrap();
        let bond = evaluator.bond.bond.clone();
        let evaluator = evaluator
            .update_with_new_info(
                NaiveDate::from_ymd_opt(2024, 4, 23).unwrap(),
                ("T2412", 104.745),
                (bond, 0.0226),
                0.019,
                None,
            )
            .calc_all()
            .unwrap();
        assert!(evaluator.is_deliverable().unwrap());
        assert_approx_eq(evaluator.accrued_interest, 1.1002747252747254);
        assert_approx_eq(evaluator.dirty_price, 104.6157328976577);
        assert_approx_eq(evaluator.clean_price, 103.51545817238296);
        assert_approx_eq(evaluator.duration, 8.360691052983789);
        assert_approx_eq(evaluator.cf, 0.9743);
        assert_approx_eq(evaluator.basis_spread, 1.4624046723829593);
        assert_approx_eq(evaluator.carry, 0.45174629893566354);
        assert_approx_eq(evaluator.net_basis_spread, 1.0106583734472958);
        assert_approx_eq(evaluator.f_b_spread, 0.26958580234230567);
        assert_approx_eq(evaluator.deliver_accrued_interest, 0.1622652);
        assert_approx_eq(evaluator.deliver_cost, 101.94573289765769);
        assert_approx_eq(evaluator.future_dirty_price, 102.2153187);
        assert_approx_eq(evaluator.irr, 0.00400090103229304);
        assert_approx_eq(evaluator.future_ytm, 0.024131859300099628);
    }

    #[test]
    fn test_cf_with_cffex() {
        let bond_vec = vec![
            "240006", "210009", "240013", "240013", "210017", "210017", "210017", "240017",
            "240017", "240017",
        ];
        let future_vec = vec![
            "T2409", "T2409", "T2409", "T2412", "T2409", "T2412", "T2503", "T2409", "T2412",
            "T2503",
        ];
        let cf_vec = vec![
            0.958, 1.0012, 0.9469, 0.9486, 0.9929, 0.9932, 0.9934, 0.9241, 0.9258, 0.9274,
        ];
        let mut evaluator = TfEvaluator::default();
        for ((code, future), expect_cf) in bond_vec.iter().zip(future_vec.iter()).zip(cf_vec.iter())
        {
            evaluator = evaluator
                .update_with_new_info(
                    NaiveDate::from_ymd_opt(2024, 9, 4).unwrap(),
                    (*future, f64::NAN),
                    (format!("{}.IB", code), f64::NAN),
                    f64::NAN,
                    None,
                )
                .with_cf()
                .unwrap();
            assert_approx_eq(evaluator.cf, *expect_cf);
        }
    }
}
