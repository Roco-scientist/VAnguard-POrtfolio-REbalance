use clap::{crate_version, App, Arg};

/// Args struct holds all CLI argument values passed
///
/// # Panic
///
/// Panics if percent stock and bond do not add up to 100
#[derive(Clone)]
pub struct Args {
    pub csv_path: String, // Path of the downloaded vanguard transactions file
    pub percent_stock_brokerage: f32, // Percent of stocks for brokerage account
    pub percent_bond_brokerage: f32, // Percent of bonds for brokerage account
    pub percent_stock_retirement: f32, // Percent of stock for retirement account
    pub percent_bond_retirement: f32, // Percent bond for retirement account
    pub brokerage_add: f32, // Amount of cash added to brokerage account
    pub traditional_add: f32, // Amount of cash added to traditional IRA account
    pub roth_add: f32,    // Amount of cash added to Roth IRA account
    pub brok_acct_option: Option<u32>, // Vanguard brokerage account number
    pub trad_acct_option: Option<u32>, // Vanguard traditional IRA account number
    pub roth_acct_option: Option<u32>, // Vanguard roth IRA account number
}

impl Args {
    pub fn new() -> Self {
        let args = App::new("Vanguard Stock Adjustment")
            .version(crate_version!())
            .author("Rory Coffey <coffeyrt@gmail.com")
            .about("Return the number of vanguard stock that should be bought")
            .arg(
                Arg::with_name("Vanguard-Download")
                    .required(true)
                    .help("CSV download file from Vanguard with holdings"),
            )
            .arg(
                Arg::with_name("percent-bond-brokerage")
                    .short("b")
                    .long("bond-percent-brokerage")
                    .takes_value(true)
                    .default_value("40")
                    .help("Percentage to allocate in bonds in the brokerage account"),
            )
            .arg(
                Arg::with_name("percent-stock-brokerage")
                    .short("s")
                    .long("stock-percent-brokerage")
                    .takes_value(true)
                    .default_value("60")
                    .help("Percentage to allocate in stocks in the brokerage account"),
            )
            .arg(
                Arg::with_name("percent-bond-retirement")
                    .long("bond-percent-retirement")
                    .takes_value(true)
                    .default_value("10")
                    .help("Percentage to allocate in bonds in the retirement account"),
            )
            .arg(
                Arg::with_name("percent-stock-retirement")
                    .long("stock-percent-retirement")
                    .takes_value(true)
                    .default_value("90")
                    .help("Percentage to allocate in stocks in the retirement account"),
            )
            .arg(
                Arg::with_name("add-brokerage")
                    .long("add-brokerage")
                    .takes_value(true)
                    .default_value("0")
                    .help("Amount of cash added to the brokerage account"),
            )
            .arg(
                Arg::with_name("add-traditional")
                    .long("add-traditional")
                    .takes_value(true)
                    .default_value("0")
                    .help("Amount of cash added to the traditional IRA account"),
            )
            .arg(
                Arg::with_name("add-roth")
                    .long("add-roth")
                    .takes_value(true)
                    .default_value("0")
                    .help("Amount of cash added to the roth IRA account"),
            )
            .arg(
                Arg::with_name("brokerage-acct")
                    .long("brokerage-acct")
                    .takes_value(true)
                    .required_unless_one(&["roth-acct", "trad-acct"])
                    .help("Brokerage account number"),
            )
            .arg(
                Arg::with_name("roth-acct")
                    .long("roth-acct")
                    .takes_value(true)
                    .required_unless_one(&["brokerage-acct", "trad-acct"])
                    .help("Roth IRA account number"),
            )
            .arg(
                Arg::with_name("trad-acct")
                    .long("trad-acct")
                    .takes_value(true)
                    .required_unless_one(&["brokerage-acct", "roth-acct"])
                    .help("Traditional IRA account number"),
            )
            .get_matches();
        let csv_path = args.value_of("Vanguard-Download").unwrap().to_string();
        let percent_stock_brokerage = args
            .value_of("percent-stock-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let percent_bond_brokerage = args
            .value_of("percent-bond-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        assert_eq!(
            percent_stock_brokerage + percent_bond_brokerage,
            100.0,
            "Brokerage stock and bond percentage does not add up to 100"
        );
        let brokerage_add = args
            .value_of("add-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let traditional_add = args
            .value_of("add-traditional")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let roth_add = args.value_of("add-roth").unwrap().parse::<f32>().unwrap();
        let percent_stock_retirement = args
            .value_of("percent-stock-retirement")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let percent_bond_retirement = args
            .value_of("percent-bond-retirement")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        assert_eq!(
            percent_stock_retirement + percent_bond_retirement,
            100.0,
            "Retirement stock and bond percentage does not add up to 100"
        );
        let mut brok_acct_option = None;
        if let Some(brok_acct_str) = args.value_of("brokerage-acct") {
            brok_acct_option = Some(brok_acct_str.parse::<u32>().unwrap())
        }
        let mut trad_acct_option = None;
        if let Some(trad_acct_str) = args.value_of("trad-acct") {
            trad_acct_option = Some(trad_acct_str.parse::<u32>().unwrap())
        }
        let mut roth_acct_option = None;
        if let Some(roth_acct_str) = args.value_of("roth-acct") {
            roth_acct_option = Some(roth_acct_str.parse::<u32>().unwrap())
        }
        Args {
            csv_path,
            percent_stock_brokerage,
            percent_bond_brokerage,
            percent_stock_retirement,
            percent_bond_retirement,
            brokerage_add,
            traditional_add,
            roth_add,
            brok_acct_option,
            trad_acct_option,
            roth_acct_option,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}
