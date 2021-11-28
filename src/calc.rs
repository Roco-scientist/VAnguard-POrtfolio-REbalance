const HIGH_TO_LOW_RISK: [crate::holdings::StockSymbols; 7] = [
    crate::holdings::StockSymbols::VWO,
    crate::holdings::StockSymbols::VXUS,
    crate::holdings::StockSymbols::VB,
    crate::holdings::StockSymbols::VO,
    crate::holdings::StockSymbols::VV,
    crate::holdings::StockSymbols::BNDX,
    crate::holdings::StockSymbols::VTC,
];

/// to_buy calculates how much of each stock and bond should be bought and sold to rebalance the
/// portfolio.
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
    if let Some(brokerage_holdings) = vanguard_holdings.brockerage_holdings() {
        brokerage_calc(
            vanguard_holdings.stock_quotes(),
            brokerage_holdings,
            alpaca_holdings,
            args.brokerage_add,
            args.percent_stock_brokerage,
            args.percent_bond_brokerage,
        );
    }
}

/// brokerage_calc calculates the amount of stocks and bonds that should be bought/sold within the
/// brokerage account in order to rebalance
fn brokerage_calc(
    quotes: crate::holdings::ShareValues,
    mut brokerage: crate::holdings::ShareValues,
    other_us_stock_value: f32,
    added_value: f32,
    percent_stock: f32,
    percent_bond: f32,
) {
    brokerage.add_stock_value(
        crate::holdings::StockSymbols::VMFXX,
        brokerage.vmfxx + added_value,
    );
    let target_holdings = crate::holdings::ShareValues::new_target(
        brokerage.total_value(),
        percent_bond,
        percent_stock,
        other_us_stock_value,
        0.0,
        0.0,
        0.0,
    );
    let difference = target_holdings.subtract(&brokerage);
    let stock_purchase = difference.divide(&quotes);
    let brokerage_account =
        crate::holdings::AccountHoldings::new(brokerage, target_holdings, stock_purchase);
    println!("Brokerage:\n{}\n", brokerage_account);
}

/// retirement_calc calculates the amount of stocks and bonds that should be bought/sold within the
/// retirement account in order to rebalance.  If there are both a roth and traditional IRA
/// account, the riskiest assets are shifted towards the roth account while the less risky assets
/// are within the tradiitonal account.  This is to keep the largest growth within the account that
/// is not taxed after withdrawals
fn retirement_calc(
    vanguard_holdings: &crate::holdings::VanguardHoldings,
    added_value_roth: f32,
    added_value_trad: f32,
    percent_stock: f32,
    percent_bond: f32,
) {
    if let Some(mut roth_holdings) = vanguard_holdings.roth_ira_holdings() {
        roth_holdings.add_stock_value(
            crate::holdings::StockSymbols::VMFXX,
            roth_holdings.vmfxx + added_value_roth,
        );
        // If there are both Roth and Traditional accounts, shift the risky assets to the roth
        // account
        if let Some(mut traditional_holdings) = vanguard_holdings.traditional_ira_holdings() {
            traditional_holdings.add_stock_value(
                crate::holdings::StockSymbols::VMFXX,
                traditional_holdings.vmfxx + added_value_trad,
            );
            let total_value = roth_holdings.total_value() + traditional_holdings.total_value();
            let total_value_sub_vtivx =
                total_value - roth_holdings.vtivx - traditional_holdings.vtivx;
            let mut overall_target = crate::holdings::ShareValues::new_target(
                total_value_sub_vtivx,
                percent_bond,
                percent_stock,
                0.0,
                0.0,
                0.0,
                0.0,
            );
            overall_target.add_stock_value(
                crate::holdings::StockSymbols::VTIVX,
                roth_holdings.vtivx + traditional_holdings.vtivx,
            );
            println!("Retirement target:\n{}\n", overall_target);
            let mut roth_total = roth_holdings.total_value();
            let mut roth_target = crate::holdings::ShareValues::new();
            roth_target.add_stock_value(crate::holdings::StockSymbols::VTIVX, roth_holdings.vtivx);
            roth_total -= roth_holdings.vtivx;
            for stock_symbol in HIGH_TO_LOW_RISK {
                let value = overall_target.stock_value(stock_symbol.clone()).min(roth_total);
                roth_total -= value;
                roth_target.add_stock_value(stock_symbol.clone(), value);
                if roth_total <= 0.0 {
                    break;
                }
            }
            assert_eq!(roth_total, 0.0, "Unexpected leftover roth cash");
            assert!(
                roth_target.total_value() > (0.99 * roth_holdings.total_value())
                    && roth_target.total_value() < (1.01 * roth_holdings.total_value()),
                "Roth target and total do not match\n\nRoth target:\n{}\n\nRoth:\n{}",
                roth_target,
                roth_holdings
            );
            let roth_difference = roth_target.subtract(&roth_holdings);
            let roth_purchase = roth_difference.divide(&vanguard_holdings.stock_quotes());
            let roth_account = crate::holdings::AccountHoldings::new(
                roth_holdings,
                roth_target.clone(),
                roth_purchase,
            );
            let traditional_target = overall_target.subtract(&roth_target);
            let traditional_difference = traditional_target.subtract(&traditional_holdings);
            let traditional_purchase =
                traditional_difference.divide(&vanguard_holdings.stock_quotes());
            let traditional_account = crate::holdings::AccountHoldings::new(
                traditional_holdings,
                traditional_target,
                traditional_purchase,
            );
            println!("Roth IRA:\n{}\n", roth_account);
            println!("Traditional IRA:\n{}\n", traditional_account)
        } else {
            let total_value_sub_vtivx = roth_holdings.total_value() - roth_holdings.vtivx;
            let mut roth_target = crate::holdings::ShareValues::new_target(
                total_value_sub_vtivx,
                percent_bond,
                percent_stock,
                0.0,
                0.0,
                0.0,
                0.0,
            );
            roth_target.add_stock_value(crate::holdings::StockSymbols::VTIVX, roth_holdings.vtivx);
            let roth_difference = roth_target.subtract(&roth_holdings);
            let roth_purchase = roth_difference.divide(&vanguard_holdings.stock_quotes());
            let roth_account = crate::holdings::AccountHoldings::new(
                roth_holdings,
                roth_target.clone(),
                roth_purchase,
            );
            println!("Roth IRA:\n{}\n", roth_account);
        }
    } else if let Some(mut traditional_holdings) = vanguard_holdings.traditional_ira_holdings() {
        traditional_holdings.add_stock_value(
            crate::holdings::StockSymbols::VMFXX,
            traditional_holdings.vmfxx + added_value_trad,
        );
        let total_value_sub_vtivx = traditional_holdings.total_value() - traditional_holdings.vtivx;
        let mut traditional_target = crate::holdings::ShareValues::new_target(
            total_value_sub_vtivx,
            percent_bond,
            percent_stock,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        traditional_target.add_stock_value(
            crate::holdings::StockSymbols::VTIVX,
            traditional_holdings.vtivx,
        );
        let traditional_difference = traditional_target.subtract(&traditional_holdings);
        let traditional_purchase = traditional_difference.divide(&vanguard_holdings.stock_quotes());
        let traditional_account = crate::holdings::AccountHoldings::new(
            traditional_holdings,
            traditional_target,
            traditional_purchase,
        );
        println!("Traditional IRA:\n{}\n", traditional_account)
    }
}
