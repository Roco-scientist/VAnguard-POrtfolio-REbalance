use crate::holdings::{AccountHoldings, HoldingType, ShareValues, StockSymbol, VanguardHoldings, VanguardRebalance};

const HIGH_TO_LOW_RISK: [StockSymbol; 7] = [
    StockSymbol::VWO,
    StockSymbol::VXUS,
    StockSymbol::VB,
    StockSymbol::VO,
    StockSymbol::VV,
    StockSymbol::BNDX,
    StockSymbol::VTC,
];

/// to_buy calculates how much of each stock and bond should be bought and sold to rebalance the
/// portfolio.
pub fn to_buy(
    vanguard_holdings: VanguardHoldings,
    alpaca_holdings: f32,
    args: crate::arguments::Args,
) -> VanguardRebalance {
    let mut rebalance = VanguardRebalance::new();
    let (traditional_ira_account_option, roth_ira_account_option, target_overall_retirement_option) = retirement_calc(
        &vanguard_holdings,
        args.roth_add,
        args.traditional_add,
        args.percent_stock_retirement,
        args.percent_bond_retirement,
    );
    if let Some(traditional_account) = traditional_ira_account_option {
        rebalance.add_account_holdings(
            traditional_account,
            HoldingType::TraditionalIra,
        )
    }
    if let Some(roth_account) = roth_ira_account_option {
        rebalance.add_account_holdings(roth_account, HoldingType::RothIra)
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
            HoldingType::Brokerage,
        )
    }
    if let Some(target_overall_retirement) = target_overall_retirement_option {
        rebalance.add_retirement_target(target_overall_retirement);
    }
    rebalance
}

/// brokerage_calc calculates the amount of stocks and bonds that should be bought/sold within the
/// brokerage account in order to rebalance
fn brokerage_calc(
    quotes: ShareValues,
    mut brokerage: ShareValues,
    other_us_stock_value: f32,
    added_value: f32,
    percent_stock: f32,
    percent_bond: f32,
) -> AccountHoldings {
    brokerage.add_stock_value(
        StockSymbol::VMFXX,
        brokerage.stock_value(StockSymbol::VMFXX) + added_value,
    );
    let target_holdings = ShareValues::new_target(
        brokerage.total_value(),
        percent_bond,
        percent_stock,
        other_us_stock_value,
        0.0,
        0.0,
        0.0,
    );
    let difference = target_holdings - brokerage;
    let stock_purchase = difference / quotes;
    AccountHoldings::new(brokerage, target_holdings, stock_purchase)
}

type TraditionalIraAccount = AccountHoldings;
type RothIraAccount = AccountHoldings;
type TargetOverallRetirement = ShareValues;

/// retirement_calc calculates the amount of stocks and bonds that should be bought/sold within the
/// retirement account in order to rebalance.  If there are both a roth and traditional IRA
/// account, the riskiest assets are shifted towards the roth account while the less risky assets
/// are within the tradiitonal account.  This is to keep the largest growth within the account that
/// is not taxed after withdrawals
fn retirement_calc(
    vanguard_holdings: &VanguardHoldings,
    added_value_roth: f32,
    added_value_trad: f32,
    percent_stock: f32,
    percent_bond: f32,
) -> (
    Option<TraditionalIraAccount>,
    Option<RothIraAccount>,
    Option<TargetOverallRetirement>
) {
    let mut traditional_ira_account_option = None;
    let mut roth_ira_account_option = None;
    let mut target_overall_retirement_option = None;
    if let Some(mut roth_holdings) = vanguard_holdings.roth_ira_holdings() {
        roth_holdings.add_stock_value(
            StockSymbol::VMFXX,
            roth_holdings.stock_value(StockSymbol::VMFXX) + added_value_roth,
        );
        // If there are both Roth and Traditional accounts, shift the risky assets to the roth
        // account
        if let Some(mut traditional_holdings) = vanguard_holdings.traditional_ira_holdings() {
            traditional_holdings.add_stock_value(
                StockSymbol::VMFXX,
                traditional_holdings.stock_value(StockSymbol::VMFXX)
                    + added_value_trad,
            );
            let target_overall_retirement = ShareValues::new_target(
                roth_holdings.total_value() + traditional_holdings.total_value(),
                percent_bond,
                percent_stock,
                0.0,
                0.0,
                0.0,
                0.0,
            );

            target_overall_retirement_option = Some(target_overall_retirement.clone());
            let mut roth_total = roth_holdings.total_value();
            let mut roth_target = ShareValues::new();
            for stock_symbol in HIGH_TO_LOW_RISK {
                let value = target_overall_retirement
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
            let roth_difference = roth_target - roth_holdings;
            let roth_purchase = roth_difference / vanguard_holdings.stock_quotes();
            let roth_account = AccountHoldings::new(
                roth_holdings,
                roth_target,
                roth_purchase,
            );
            let traditional_target = target_overall_retirement - roth_target;
            let traditional_difference = traditional_target - traditional_holdings;
            let traditional_purchase =
                traditional_difference / vanguard_holdings.stock_quotes();
            let traditional_account = AccountHoldings::new(
                traditional_holdings,
                traditional_target,
                traditional_purchase,
            );
            roth_ira_account_option = Some(roth_account);
            traditional_ira_account_option = Some(traditional_account);
        } else {
            let roth_target = ShareValues::new_target(
                roth_holdings.total_value(),
                percent_bond,
                percent_stock,
                0.0,
                0.0,
                0.0,
                0.0,
            );
            let roth_difference = roth_target - roth_holdings;
            let roth_purchase = roth_difference / vanguard_holdings.stock_quotes();
            let roth_account =
                AccountHoldings::new(roth_holdings, roth_target, roth_purchase);
            roth_ira_account_option = Some(roth_account);
        }
    } else if let Some(mut traditional_holdings) = vanguard_holdings.traditional_ira_holdings() {
        traditional_holdings.add_stock_value(
            StockSymbol::VMFXX,
            traditional_holdings.stock_value(StockSymbol::VMFXX)
                + added_value_trad,
        );
        let traditional_target = ShareValues::new_target(
            traditional_holdings.total_value(),
            percent_bond,
            percent_stock,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        let traditional_difference = traditional_target - traditional_holdings;
        let traditional_purchase = traditional_difference / vanguard_holdings.stock_quotes();
        let traditional_account = AccountHoldings::new(
            traditional_holdings,
            traditional_target,
            traditional_purchase,
        );
        traditional_ira_account_option = Some(traditional_account)
    }
    (traditional_ira_account_option, roth_ira_account_option, target_overall_retirement_option)
}
