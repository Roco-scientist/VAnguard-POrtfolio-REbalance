const HIGH_TO_LOW_RISK: [crate::holdings::StockSymbol; 7] = [
    crate::holdings::StockSymbol::VWO,
    crate::holdings::StockSymbol::VXUS,
    crate::holdings::StockSymbol::VB,
    crate::holdings::StockSymbol::VO,
    crate::holdings::StockSymbol::VV,
    crate::holdings::StockSymbol::BNDX,
    crate::holdings::StockSymbol::VTC,
];

/// to_buy calculates how much of each stock and bond should be bought and sold to rebalance the
/// portfolio.
pub fn to_buy(
    vanguard_holdings: crate::holdings::VanguardHoldings,
    alpaca_holdings: f32,
    args: crate::arguments::Args,
) -> crate::holdings::VanguardRebalance {
    let mut rebalance = crate::holdings::VanguardRebalance::new();
    let (traditional_ira_account_option, roth_ira_account_option) = retirement_calc(
        &vanguard_holdings,
        args.roth_add,
        args.traditional_add,
        args.percent_stock_retirement,
        args.percent_bond_retirement,
    );
    if let Some(traditional_account) = traditional_ira_account_option {
        rebalance.add_account_holdings(
            traditional_account,
            crate::holdings::HoldingType::TraditionalIra,
        )
    }
    if let Some(roth_account) = roth_ira_account_option {
        rebalance.add_account_holdings(roth_account, crate::holdings::HoldingType::RothIra)
    }
    if let Some(brokerage_holdings) = vanguard_holdings.brokerage_holdings() {
        rebalance.add_account_holdings(
            brokerage_calc(
                vanguard_holdings.stock_quotes(),
                brokerage_holdings,
                alpaca_holdings,
                args.brokerage_add,
                args.percent_stock_brokerage,
                args.percent_bond_brokerage,
            ),
            crate::holdings::HoldingType::Brokerage,
        )
    }
    rebalance
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
) -> crate::holdings::AccountHoldings {
    brokerage.add_stock_value(
        crate::holdings::StockSymbol::VMFXX,
        brokerage.stock_value(crate::holdings::StockSymbol::VMFXX) + added_value,
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
    crate::holdings::AccountHoldings::new(brokerage, target_holdings, stock_purchase)
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
) -> (
    Option<crate::holdings::AccountHoldings>,
    Option<crate::holdings::AccountHoldings>,
) {
    let mut traditional_ira_account_option = None;
    let mut roth_ira_account_option = None;
    if let Some(mut roth_holdings) = vanguard_holdings.roth_ira_holdings() {
        roth_holdings.add_stock_value(
            crate::holdings::StockSymbol::VMFXX,
            roth_holdings.stock_value(crate::holdings::StockSymbol::VMFXX) + added_value_roth,
        );
        // If there are both Roth and Traditional accounts, shift the risky assets to the roth
        // account
        if let Some(mut traditional_holdings) = vanguard_holdings.traditional_ira_holdings() {
            traditional_holdings.add_stock_value(
                crate::holdings::StockSymbol::VMFXX,
                traditional_holdings.stock_value(crate::holdings::StockSymbol::VMFXX)
                    + added_value_trad,
            );
            let overall_target = crate::holdings::ShareValues::new_target(
                roth_holdings.total_value() + traditional_holdings.total_value(),
                percent_bond,
                percent_stock,
                0.0,
                0.0,
                0.0,
                0.0,
            );
            println!("Retirement target:\n{}\n", overall_target);
            let mut roth_total = roth_holdings.total_value();
            let mut roth_target = crate::holdings::ShareValues::new();
            for stock_symbol in HIGH_TO_LOW_RISK {
                let value = overall_target
                    .stock_value(stock_symbol.clone())
                    .min(roth_total);
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
            roth_ira_account_option = Some(roth_account);
            traditional_ira_account_option = Some(traditional_account);
        } else {
            let roth_target = crate::holdings::ShareValues::new_target(
                roth_holdings.total_value(),
                percent_bond,
                percent_stock,
                0.0,
                0.0,
                0.0,
                0.0,
            );
            let roth_difference = roth_target.subtract(&roth_holdings);
            let roth_purchase = roth_difference.divide(&vanguard_holdings.stock_quotes());
            let roth_account =
                crate::holdings::AccountHoldings::new(roth_holdings, roth_target, roth_purchase);
            roth_ira_account_option = Some(roth_account);
        }
    } else if let Some(mut traditional_holdings) = vanguard_holdings.traditional_ira_holdings() {
        traditional_holdings.add_stock_value(
            crate::holdings::StockSymbol::VMFXX,
            traditional_holdings.stock_value(crate::holdings::StockSymbol::VMFXX)
                + added_value_trad,
        );
        let traditional_target = crate::holdings::ShareValues::new_target(
            traditional_holdings.total_value(),
            percent_bond,
            percent_stock,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        let traditional_difference = traditional_target.subtract(&traditional_holdings);
        let traditional_purchase = traditional_difference.divide(&vanguard_holdings.stock_quotes());
        let traditional_account = crate::holdings::AccountHoldings::new(
            traditional_holdings,
            traditional_target,
            traditional_purchase,
        );
        traditional_ira_account_option = Some(traditional_account)
    }
    (traditional_ira_account_option, roth_ira_account_option)
}
