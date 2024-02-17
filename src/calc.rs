use anyhow::{ensure, Context, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{
    arguments::Args,
    asset::{Allocations, SubAllocations},
    holdings::{
        AccountHoldings, HoldingType, ShareValues, StockSymbol, VanguardHoldings, VanguardRebalance,
    },
};

const HIGH_TO_LOW_RISK: [StockSymbol; 9] = [
    StockSymbol::VWO,
    StockSymbol::VXUS,
    StockSymbol::VB,
    StockSymbol::VO,
    StockSymbol::VV,
    StockSymbol::BNDX,
    StockSymbol::BND,
    StockSymbol::VTC,
    StockSymbol::VTIP,
];

/// to_buy calculates how much of each stock and bond should be bought and sold to rebalance the
/// portfolio.
pub fn to_buy(vanguard_holdings: VanguardHoldings, args: Args) -> Result<VanguardRebalance> {
    let mut rebalance = VanguardRebalance::new();
    let (
        traditional_ira_account_option,
        roth_ira_account_option,
        brokerage_account_option,
        target_overall_retirement_option,
    ) = retirement_calc(&vanguard_holdings, args.clone())?;
    if let Some(traditional_account) = traditional_ira_account_option {
        rebalance.add_account_holdings(traditional_account, HoldingType::TraditionalIra)
    }
    if let Some(roth_account) = roth_ira_account_option {
        rebalance.add_account_holdings(roth_account, HoldingType::RothIra)
    }
    if let Some(brokerage_account) = brokerage_account_option {
        rebalance.add_account_holdings(brokerage_account, HoldingType::Brokerage)
    } else if let Some(brokerage_holdings) = vanguard_holdings.brokerage_holdings() {
        rebalance.add_account_holdings(
            brokerage_calc(vanguard_holdings.stock_quotes(), brokerage_holdings, args)?,
            HoldingType::Brokerage,
        )
    }
    if let Some(target_overall_retirement) = target_overall_retirement_option {
        rebalance.add_retirement_target(target_overall_retirement);
    }
    Ok(rebalance)
}

/// brokerage_calc calculates the amount of stocks and bonds that should be bought/sold within the
/// brokerage account in order to rebalance
fn brokerage_calc(
    quotes: ShareValues,
    mut brokerage: ShareValues,
    args: Args,
) -> Result<AccountHoldings> {
    brokerage.add_stock_value(
        StockSymbol::VMFXX,
        brokerage.stock_value(StockSymbol::VMFXX) + args.brokerage_cash_add,
    );
    brokerage.add_outside_stock_value(args.brokerage_us_stock_add + args.brokerage_int_stock_add);
    brokerage.add_outside_bond_value(args.brokerage_us_bond_add + args.brokerage_int_bond_add);
    let asset_allocations = Allocations::custom(
        args.percent_stock_brokerage,
        args.percent_bond_brokerage,
        0.0,
    )?;
    let sub_allocations = SubAllocations::new_custom(asset_allocations)?;
    let target_holdings = ShareValues::new_target(
        sub_allocations,
        brokerage.total_value(),
        args.brokerage_us_stock_add,
        args.brokerage_us_bond_add,
        args.brokerage_int_stock_add,
        args.brokerage_int_bond_add,
    );
    let difference = target_holdings - brokerage;
    let stock_purchase = difference / quotes;
    Ok(AccountHoldings::new(
        brokerage,
        target_holdings,
        stock_purchase,
    ))
}

type TraditionalIraAccount = AccountHoldings;
type RothIraAccount = AccountHoldings;
type BrokerageAccount = AccountHoldings;
type TargetOverallRetirement = ShareValues;

/// retirement_calc calculates the amount of stocks and bonds that should be bought/sold within the
/// retirement account in order to rebalance.  If there are both a roth and traditional IRA
/// account, the riskiest assets are shifted towards the roth account while the less risky assets
/// are within the tradiitonal account.  This is to keep the largest growth within the account that
/// is not taxed after withdrawals
fn retirement_calc(
    vanguard_holdings: &VanguardHoldings,
    args: Args,
) -> Result<(
    Option<TraditionalIraAccount>,
    Option<RothIraAccount>,
    Option<BrokerageAccount>,
    Option<TargetOverallRetirement>,
)> {
    let mut traditional_ira_account_option = None;
    let mut roth_ira_account_option = None;
    let mut brokerage_account_option = None;
    let mut target_overall_retirement_option = None;

    let mut allocations = Allocations::new();

    if let Some(retirement_year) = args.retirement_year_option {
        allocations = Allocations::retirement(retirement_year)?;
    }
    if let Some(stock_percent) = args.percent_stock_retirement_option {
        let bond_percent;
        if let Some(input_bond_percent) = args.percent_bond_retirement_option {
            bond_percent = input_bond_percent;
        } else {
            bond_percent = 100.0 - stock_percent;
        }
        allocations = Allocations::custom(stock_percent, bond_percent, 0.0)?;
    } else if let Some(bond_percent) = args.percent_bond_retirement_option {
        let stock_percent = 100.0 - bond_percent;
        allocations = Allocations::custom(stock_percent, bond_percent, 0.0)?;
    };

    let sub_allocations = SubAllocations::new_custom(allocations)?;

    let mut holdings_value = 0.0;
    let mut us_stock_add = 0.0;
    let mut us_bond_add = 0.0;
    let mut int_stock_add = 0.0;
    let mut int_bond_add = 0.0;

    let mut include_roth = false;
    let mut include_traditional = false;
    let mut include_brokerage = false;

    let mut roth_total = 0.0;
    let mut brokerage_total = 0.0;

    let mut roth_holdings_final = ShareValues::new();
    let mut brokerage_holdings_final = ShareValues::new();
    let mut traditional_holdings_final = ShareValues::new();
    if let Some(mut roth_holdings) = vanguard_holdings.roth_ira_holdings() {
        roth_holdings.add_stock_value(
            StockSymbol::VMFXX,
            roth_holdings.stock_value(StockSymbol::VMFXX) + args.roth_cash_add,
        );
        holdings_value += roth_holdings.total_value();
        us_stock_add += args.roth_us_stock_add;
        us_bond_add += args.roth_us_bond_add;
        int_stock_add += args.roth_int_stock_add;
        int_bond_add += args.roth_int_bond_add;
        include_roth = true;
        roth_total = roth_holdings.total_value();
        roth_holdings_final = roth_holdings;
    }
    // If there are both Roth and Traditional accounts, shift the risky assets to the roth
    // account
    if let Some(mut traditional_holdings) = vanguard_holdings.traditional_ira_holdings() {
        traditional_holdings.add_stock_value(
            StockSymbol::VMFXX,
            traditional_holdings.stock_value(StockSymbol::VMFXX) + args.traditional_cash_add,
        );
        holdings_value += traditional_holdings.total_value();
        us_stock_add += args.traditional_us_stock_add;
        us_bond_add += args.traditional_us_bond_add;
        int_stock_add += args.traditional_int_stock_add;
        int_bond_add += args.traditional_int_bond_add;
        include_traditional = true;
        traditional_holdings_final = traditional_holdings;
    }
    if args.use_brokerage_retirement {
        if let Some(mut brokerage_holdings) = vanguard_holdings.brokerage_holdings() {
            brokerage_holdings.add_stock_value(
                StockSymbol::VMFXX,
                brokerage_holdings.stock_value(StockSymbol::VMFXX) + args.brokerage_cash_add,
            );
            holdings_value += brokerage_holdings.total_value();
            us_stock_add += args.brokerage_us_stock_add;
            us_bond_add += args.brokerage_us_bond_add;
            int_stock_add += args.brokerage_int_stock_add;
            int_bond_add += args.brokerage_int_bond_add;
            include_brokerage = true;
            brokerage_total = brokerage_holdings.total_value();
            brokerage_holdings_final = brokerage_holdings;
        }
    }

    let mut target_overall_retirement = ShareValues::new();
    if [include_brokerage, include_traditional, include_roth]
        .iter()
        .any(|&x| x)
    {
        target_overall_retirement = ShareValues::new_target(
            sub_allocations,
            holdings_value,
            us_stock_add,
            us_bond_add,
            int_stock_add,
            int_bond_add,
        );
        target_overall_retirement_option = Some(target_overall_retirement);
    }

    let mut remaining_target = target_overall_retirement;
    if include_roth {
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
        ensure!(roth_total == 0.0, "Unexpected leftover roth cash");
        ensure!(
            roth_target.total_value() > (0.99 * roth_holdings_final.total_value())
                && roth_target.total_value() < (1.01 * roth_holdings_final.total_value()),
            "Roth target and total do not match\n\nRoth target:\n{}\n\nRoth:\n{}",
            roth_target,
            roth_holdings_final
        );
        let roth_difference = roth_target - roth_holdings_final;
        let roth_purchase = roth_difference / vanguard_holdings.stock_quotes();
        let roth_account = AccountHoldings::new(roth_holdings_final, roth_target, roth_purchase);
        remaining_target = remaining_target - roth_target;
        roth_ira_account_option = Some(roth_account);
    }

    if include_brokerage {
        let mut brokerage_target = ShareValues::new();
        for stock_symbol in HIGH_TO_LOW_RISK.iter().rev() {
            let value = target_overall_retirement
                .stock_value(stock_symbol.clone())
                .min(brokerage_total);
            brokerage_total -= value;
            brokerage_target.add_stock_value(stock_symbol.clone(), value);
            if brokerage_total <= 0.0 {
                break;
            }
        }
        ensure!(brokerage_total == 0.0, "Unexpected leftover brokerage cash");
        ensure!(
            brokerage_target.total_value() > (0.99 * brokerage_holdings_final.total_value())
                && brokerage_target.total_value() < (1.01 * brokerage_holdings_final.total_value()),
            "brokerage target and total do not match\n\nbrokerage target:\n{}\n\nbrokerage:\n{}",
            brokerage_target,
            brokerage_holdings_final
        );
        let brokerage_difference = brokerage_target - brokerage_holdings_final;
        let brokerage_purchase = brokerage_difference / vanguard_holdings.stock_quotes();
        let brokerage_account = AccountHoldings::new(
            brokerage_holdings_final,
            brokerage_target,
            brokerage_purchase,
        );
        remaining_target = remaining_target - brokerage_target;
        brokerage_account_option = Some(brokerage_account);
    }

    if include_traditional {
        let traditional_target = remaining_target;
        let traditional_difference = traditional_target - traditional_holdings_final;
        let traditional_purchase = traditional_difference / vanguard_holdings.stock_quotes();
        let traditional_account = AccountHoldings::new(
            traditional_holdings_final,
            traditional_target,
            traditional_purchase,
        );
        traditional_ira_account_option = Some(traditional_account);
    }

    // } else {
    //     let roth_target = ShareValues::new_target(
    //         sub_allocations,
    //         roth_holdings.total_value(),
    //         args.roth_us_stock_add,
    //         args.roth_us_bond_add,
    //         args.roth_int_stock_add,
    //         args.roth_int_bond_add,
    //     );
    //     let roth_difference = roth_target - roth_holdings;
    //     let roth_purchase = roth_difference / vanguard_holdings.stock_quotes();
    //     let roth_account = AccountHoldings::new(roth_holdings, roth_target, roth_purchase);
    //     roth_ira_account_option = Some(roth_account);
    // }
    // } else if let Some(mut traditional_holdings) = vanguard_holdings.traditional_ira_holdings() {
    //     traditional_holdings.add_stock_value(
    //         StockSymbol::VMFXX,
    //         traditional_holdings.stock_value(StockSymbol::VMFXX) + args.traditional_cash_add,
    //     );
    //     let traditional_target = ShareValues::new_target(
    //         sub_allocations,
    //         traditional_holdings.total_value(),
    //         args.traditional_us_stock_add,
    //         args.traditional_us_bond_add,
    //         args.traditional_int_stock_add,
    //         args.traditional_int_bond_add,
    //     );
    //     let traditional_difference = traditional_target - traditional_holdings;
    //     let traditional_purchase = traditional_difference / vanguard_holdings.stock_quotes();
    //     let traditional_account = AccountHoldings::new(
    //         traditional_holdings,
    //         traditional_target,
    //         traditional_purchase,
    //     );
    //     traditional_ira_account_option = Some(traditional_account)
    // }
    Ok((
        traditional_ira_account_option,
        roth_ira_account_option,
        brokerage_account_option,
        target_overall_retirement_option,
    ))
}

// Calculates the minimum distribution for an unmarried individual or someone without a spouse
// greater than 10 years younger.
pub fn calculate_minimum_distribution(
    age: u8,
    traditional_value: f32,
    csv_path: &str,
) -> Result<f32> {
    // Distribution table retrieved from here appendix B: https://www.irs.gov/publications/p590b#en_US_2022_publink100090310
    // May need to periodically be updated
    let csv_file = File::open(csv_path).context("Minimum distribution file from IRS not found")?;
    let mut header = Vec::new();
    let mut distribution_table = HashMap::new();
    for row_result in BufReader::new(csv_file).lines() {
        let row = row_result?;
        if row.contains(',') {
            let row_split = row
                .split(',')
                .map(|value| value.to_string())
                .collect::<Vec<String>>();
            if row_split.len() > 1 {
                if header.is_empty() {
                    header = row_split
                } else {
                    ensure!(header.iter().take(2).collect::<Vec<&String>>() == ["Age", "Distribution Period"], "Header of distribution table ({:?}) does not match ['Age','Distribution Period']", header);
                    distribution_table
                        .insert(row_split[0].parse::<u8>()?, row_split[1].parse::<f32>()?);
                }
            }
        }
    }

    if distribution_table.contains_key(&age) {
        Ok(traditional_value / distribution_table[&age])
    } else {
        Ok(0.0)
    }
}
