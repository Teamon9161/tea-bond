// use chrono::NaiveDate;
// use itertools::izip;
// use tevec::prelude::*;

// pub fn evaluators_net_basis_spread<'a, VS, VT, V, VO>(
//     future: VS,
//     bond: VS,
//     date: VT,
//     future_price: V,
//     bond_ytm: V,
//     capital_rate: V,
//     reinvest_rate: Option<f64>,
// ) -> VO
// where
//     VS: Vec1View<Option<&'a str>>,
//     VT: Vec1View<Option<NaiveDate>>,
//     V: Vec1View<Option<f64>>,
//     VO: Vec1<Option<f64>>,
// {
//     let reinvest_rate = reinvest_rate.unwrap_or(0.0);
//     _ = izip!(
//         // future.titer().cycle(),
//         // bond.titer().cycle(),
//         date.titer().cycle(),
//         future_price.titer(),
//         bond_ytm.titer(),
//         capital_rate.titer().cycle(),
//     );
//     todo!()
// }
