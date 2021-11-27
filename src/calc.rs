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
    brokerage_holdings: crate::holdings::ShareValues,
    alpaca_holdings: f32,
    added_value: f32,
    percent_stock: f32,
    percent_bond: f32,
) {
    let total_value = brokerage_holdings.total_value() + alpaca_holdings + added_value;
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
    println!("Current brokerage holdings:\n{}\n", brokerage_holdings);
    println!("Target:\n{}\n", target_holdings);

    let difference = target_holdings.subtract(brokerage_holdings);
    println!("Difference:\n{}\n", difference);

    let stock_purchase = difference.divide(quotes);
    println!("Stocks needed:\n{}\n", stock_purchase);
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
    let overall_target = crate::holdings::ShareValues {
        vxus: total_value * INT_TOTAL * percent_stock / 100.0,
        bndx: total_value * INT_BOND_FRACTION * percent_bond / 100.0,
        vwo: total_value * INT_EMERGING * percent_stock / 100.0,
        vo: total_value * EACH_US_STOCK * percent_stock / 100.0,
        vb: total_value * EACH_US_STOCK * percent_stock / 100.0,
        vtc: total_value * US_BOND_FRACTION * percent_bond / 100.0,
        vv: total_value * EACH_US_STOCK * percent_stock / 100.0,
        vmfxx: 0.0,
        vtivx: 0.0,
    };
    println!("Retirement target:\n{}\n", overall_target)
}
