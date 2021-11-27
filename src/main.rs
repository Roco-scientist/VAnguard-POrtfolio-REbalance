use alpaca_finance::{Account, Alpaca};
use vanguard_buy::arguments;

#[tokio::main]
async fn main() {
    let args = arguments::Args::new();
    let key_id = std::env::var("Alpaca_Key_ID").unwrap();
    let key = std::env::var("Alpaca_Key").unwrap();
    let alpaca = Alpaca::live(&key_id, &key).await.unwrap();
    let account = Account::get(&alpaca).await.unwrap();
    let alpaca_total = account.equity as f32;
    let vanguard_holdings =
        vanguard_buy::holdings::parse_csv_download(&args.csv_path, args.clone()).unwrap();
    vanguard_buy::calc::to_buy(vanguard_holdings, alpaca_total, args)
}
