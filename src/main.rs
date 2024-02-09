use alpaca_finance::{Account, Alpaca};
use anyhow::Result;
use chrono::Local;
use std::{fs::File, io::Write};
use vapore::arguments;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = arguments::Args::new();
    let key_id = std::env::var("APCA_API_KEY_ID").unwrap_or_else(|_| String::new());
    let key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_else(|_| String::new());
    if !key_id.is_empty() && !key.is_empty() {
        let alpaca = Alpaca::live(&key_id, &key).await?;
        let account = Account::get(&alpaca).await?;
        args.brokerage_us_stock_add += account.equity as f32;
    }
    let vanguard_holdings = vapore::holdings::parse_csv_download(&args.csv_path, args.clone())?;

    // If an age is given, print the minumum distribution needed for the year
    // TODO: need to calculate this from the value on December 31st of the previous year
    if let Some(age) = args.age_option {
        if let Some(tradtional) = vanguard_holdings.traditional_ira_holdings() {
            let minimum_distribution =
                vapore::calc::calculate_minimum_distribution(age, tradtional.total_value());
            println!("\n\nMinimum distribution: ${:.2}\n\n", minimum_distribution)
        }
    }
    //    .unwrap_or_else(|err| panic!("Holdings error: {}", err));
    let rebalance = vapore::calc::to_buy(vanguard_holdings, args.clone())?;
    println!(
        "DESCRIPTIONS:\n{}\n\n{}",
        vapore::holdings::all_stock_descriptions(),
        rebalance
    );
    if args.output {
        let datetime = Local::now().format("%Y-%m-%d_%H:%M");
        let outfile = format!("{}_vanguard_rebalance.txt", datetime);
        let mut file = File::create(outfile)?;
        file.write_all(
            format!(
                "DESCRIPTIONS:\n{}\n\n{}",
                vapore::holdings::all_stock_descriptions(),
                rebalance
            )
            .as_bytes(),
        )?
    }
    Ok(())
}
