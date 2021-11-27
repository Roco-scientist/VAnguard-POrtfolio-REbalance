const US_STOCK_FRACTION: f32 = 2.0 / 3.0;
const EACH_US_STOCK: f32 = US_STOCK_FRACTION / 3.0;
const INT_STOCK_FRACTION: f32 = 1.0 / 3.0;
const INT_EMERGING: f32 = INT_STOCK_FRACTION / 3.0;
const INT_TOTAL: f32 = INT_STOCK_FRACTION * 2.0 / 3.0;
const US_BOND_FRACTION: f32 = 2.0 / 3.0;
const INT_BOND_FRACTION: f32 = 1.0 / 3.0;

pub fn to_buy(
    vanguard_holdings: crate::holdings::VanguardHoldings,
    alpaca_holdings: f32,
    args: crate::arguments::Args,
) {
    retirement_calc(
        &vanguard_holdings,
        args.roth_add,
        args.traditional_add,
        args.percent_stock_retirement,
        args.percent_bond_retirement,
    );
    brockerage_calc(
        vanguard_holdings.stock_quotes(),
        vanguard_holdings.brockerage_holdings(),
        alpaca_holdings,
        args.brokerage_add,
        args.percent_stock_brokerage,
        args.percent_bond_brokerage,
    );
}

fn brockerage_calc(
    quotes: crate::holdings::ShareValues,
    brokerage: crate::holdings::ShareValues,
    alpaca_holdings: f32,
    added_value: f32,
    percent_stock: f32,
    percent_bond: f32,
) {
    let total_value = brokerage.total_value() + alpaca_holdings + added_value;
    let target_holdings = crate::holdings::ShareValues {
        vxus: total_value * INT_TOTAL * percent_stock / 100.0,
        bndx: total_value * INT_BOND_FRACTION * percent_bond / 100.0,
        vwo: total_value * INT_EMERGING * percent_stock / 100.0,
        vo: (total_value * EACH_US_STOCK * percent_stock / 100.0) - (alpaca_holdings / 3.0),
        vb: (total_value * EACH_US_STOCK * percent_stock / 100.0) - (alpaca_holdings / 3.0),
        vtc: total_value * US_BOND_FRACTION * percent_bond / 100.0,
        vv: (total_value * EACH_US_STOCK * percent_stock / 100.0) - (alpaca_holdings / 3.0),
        vmfxx: 0.0,
        vtivx: 0.0,
    };
    let total_percent = INT_TOTAL * percent_stock / 100.0
        + INT_BOND_FRACTION * percent_bond / 100.0
        + INT_EMERGING * percent_stock / 100.0
        + EACH_US_STOCK * percent_stock / 100.0
        + EACH_US_STOCK * percent_stock / 100.0
        + US_BOND_FRACTION * percent_bond / 100.0
        + EACH_US_STOCK * percent_stock / 100.0;
    assert_eq!(total_percent, 1.0, "Fractions did not add up for brokerage account.  The bond to stock ratio is likely off and should add up to 100");
    let difference = target_holdings.subtract(&brokerage);
    let stock_purchase = difference.divide(&quotes);
    let brokerage_holdings = crate::holdings::AccountHoldings::new(brokerage, target_holdings, stock_purchase);
    println!("Brokerage:\n{}\n", brokerage_holdings);
}

fn retirement_calc(
    vanguard_holdings: &crate::holdings::VanguardHoldings,
    added_value_roth: f32,
    added_value_trad: f32,
    percent_stock: f32,
    percent_bond: f32,
) {
    let total_percent = INT_TOTAL * percent_stock / 100.0
        + INT_BOND_FRACTION * percent_bond / 100.0
        + INT_EMERGING * percent_stock / 100.0
        + EACH_US_STOCK * percent_stock / 100.0
        + EACH_US_STOCK * percent_stock / 100.0
        + US_BOND_FRACTION * percent_bond / 100.0
        + EACH_US_STOCK * percent_stock / 100.0;
    assert_eq!(total_percent, 1.0, "Fractions did not add up for brokerage account.  The bond to stock ratio is likely off and should add up to 100");

    let roth = vanguard_holdings.roth_ira_holdings();
    let trad = vanguard_holdings.traditional_ira_holdings();
    let total_value = roth.total_value() + trad.total_value() + added_value_roth + added_value_trad;
    let total_value_sub_vtivx = total_value - roth.vtivx - trad.vtivx;
    let overall_target = crate::holdings::ShareValues {
        vxus: total_value_sub_vtivx * INT_TOTAL * percent_stock / 100.0,
        bndx: total_value_sub_vtivx * INT_BOND_FRACTION * percent_bond / 100.0,
        vwo: total_value_sub_vtivx * INT_EMERGING * percent_stock / 100.0,
        vo: total_value_sub_vtivx * EACH_US_STOCK * percent_stock / 100.0,
        vb: total_value_sub_vtivx * EACH_US_STOCK * percent_stock / 100.0,
        vtc: total_value_sub_vtivx * US_BOND_FRACTION * percent_bond / 100.0,
        vv: total_value_sub_vtivx * EACH_US_STOCK * percent_stock / 100.0,
        vmfxx: 0.0,
        vtivx: roth.vtivx + trad.vtivx,
    };
    println!("Retirement target:\n{}\n", overall_target);
    let mut roth_total = roth.total_value() + added_value_roth;
    let mut roth_target = crate::holdings::ShareValues::new();
    if roth_total > 0.0 {
        roth_target.add_stock_value("VTIVX", roth.vtivx);
        roth_total -= roth.vtivx;
        let emerging_market = overall_target.vwo.min(roth_total);
        roth_total -= emerging_market;
        roth_target.add_stock_value("VWO", emerging_market);
        let foreign_market = overall_target.vxus.min(roth_total);
        roth_total -= foreign_market;
        roth_target.add_stock_value("VXUS", foreign_market);
        let small_cap = overall_target.vb.min(roth_total);
        roth_total -= small_cap;
        roth_target.add_stock_value("VB", small_cap);
        let mid_cap = overall_target.vo.min(roth_total);
        roth_total -= mid_cap;
        roth_target.add_stock_value("VO", mid_cap);
        let large_cap = overall_target.vv.min(roth_total);
        roth_total -= large_cap;
        roth_target.add_stock_value("VV", large_cap);
        let int_bond = overall_target.bndx.min(roth_total);
        roth_total -= int_bond;
        roth_target.add_stock_value("BNDX", int_bond);
        let us_bond = overall_target.vtc.min(roth_total);
        roth_total -= us_bond;
        roth_target.add_stock_value("VTC", us_bond);
        assert_eq!(roth_total, 0.0, "Unexpected leftover roth cash");
        assert_eq!(roth_target.total_value(), roth.total_value() + added_value_roth, "Roth target and total do not match\n\nRoth target:\n{}\n\nRoth:\n{}\n\nRoth added: ${}", roth_target, roth, added_value_roth);
    } 
    let roth_difference = roth_target.subtract(&roth);
    let roth_purchase = roth_difference.divide(&vanguard_holdings.stock_quotes());
    let roth_holdings = crate::holdings::AccountHoldings::new(roth, roth_target.clone(), roth_purchase);
    let traditional_target = overall_target.subtract(&roth_target);
    let traditional_difference = traditional_target.subtract(&trad);
    let traditional_purchase = traditional_difference.divide(&vanguard_holdings.stock_quotes());
    let traditional_holdings = crate::holdings::AccountHoldings::new(trad, traditional_target, traditional_purchase);
    println!("Roth:\n{}\n", roth_holdings);
    println!("Traditional:\n{}\n", traditional_holdings)
}
