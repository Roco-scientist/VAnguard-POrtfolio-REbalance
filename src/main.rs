use alpaca_finance::{Account, Alpaca};
use vanguard_buy::arguments;

#[tokio::main]
async fn main() {
    let args = arguments::Args::new();
    let key_id = std::env::var("APCA_API_KEY_ID").unwrap_or_else(|_| "None".to_string());
    let key = std::env::var("APCA_API_SECRET_KEY").unwrap_or_else(|_| "None".to_string());
    let additional_us_stock: f32;
    if key_id != *"None" && key != *"None" {
        let alpaca = Alpaca::live(&key_id, &key).await.unwrap();
        let account = Account::get(&alpaca).await.unwrap();
        additional_us_stock = account.equity as f32;
    } else {
        additional_us_stock = 0.0
    }
    let vanguard_holdings =
        vanguard_buy::holdings::parse_csv_download(&args.csv_path, args.clone()).unwrap_or_else(|err| panic!("Holdings error: {}", err));
    vanguard_buy::calc::to_buy(vanguard_holdings, additional_us_stock, args)
}
