use alpaca_finance::{Account, Alpaca};
use anyhow::Result;
use chrono::Local;
use std::{fs::File, io::Write};
use vapore::arguments;

#[tokio::main]
async fn main() -> Result<()> {
    let args = arguments::Args::new();
    let key_id = std::env::var("APCA_API_KEY_ID").unwrap_or_else(|_| String::new());
    let key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_else(|_| String::new());
    let additional_us_stock: f32;
    if !key_id.is_empty() && !key.is_empty() {
        let alpaca = Alpaca::live(&key_id, &key).await?;
        let account = Account::get(&alpaca).await?;
        additional_us_stock = account.equity as f32;
    } else {
        additional_us_stock = 0.0
    }
    let vanguard_holdings = vapore::holdings::parse_csv_download(&args.csv_path, args.clone())?;
    //    .unwrap_or_else(|err| panic!("Holdings error: {}", err));
    let rebalance = vapore::calc::to_buy(vanguard_holdings, additional_us_stock, args.clone());
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
