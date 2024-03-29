use anyhow::{Context, Result};
use apca::{api::v2::account, ApiInfo, Client};
use chrono::Local;
use std::{fs::File, io::Write};
use vapore::arguments;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = arguments::Args::new();
    let key_id = std::env::var("APCA_API_KEY_ID").unwrap_or_else(|_| String::new());
    let key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_else(|_| String::new());
    if !key_id.is_empty() && !key.is_empty() {
        let api_info = ApiInfo::from_parts("https://api.alpaca.markets/", &key_id, &key)
            .context("Failed to retrieve Alpaca Environment info")?;
        let client = Client::new(api_info);
        let alpaca_equity = client
            .issue::<account::Get>(&())
            .await?
            .equity
            .to_f64()
            .unwrap() as f32;
        args.brokerage_us_stock_add += alpaca_equity;
    }
    let mut vanguard_holdings =
        vapore::holdings::parse_csv_download(&args.csv_path, args.clone()).await?;

    // If an age is given, print the minumum distribution needed for the year
    // TODO: need to calculate this from the value on December 31st of the previous year
    if let Some(age) = args.age_option {
        if let Some(traditional_value) = vanguard_holdings.eoy_value(args.distribution_year).await? {
            let minimum_distribution = vapore::calc::calculate_minimum_distribution(
                age,
                traditional_value,
                &args.distribution_table_path,
            )?;
            println!("\n\nEnd of previous year traditional IRA account value: ${:?}\nMinimum distribution for {}: ${:.2}\nDistribution so far: ${:.2}\nDistribution needed: ${:.2}\n\n", traditional_value, args.distribution_year, minimum_distribution, vanguard_holdings.distributions(), (minimum_distribution - vanguard_holdings.distributions()).max(0.0));
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
