use clap::{crate_version, App, Arg};

/// Args struct holds all CLI argument values passed
///
/// # Panic
///
/// Panics if percent stock and bond do not add up to 100
#[derive(Clone)]
pub struct Args {
    pub csv_path: String, // Path of the downloaded vanguard transactions file
    pub retirement_year_option: Option<i32>,
    pub percent_stock_brokerage: f32, // Percent of stocks for brokerage account
    pub percent_bond_brokerage: f32,  // Percent of bonds for brokerage account
    pub percent_stock_retirement_option: Option<f32>, // Percent of stock for retirement account
    pub percent_bond_retirement_option: Option<f32>, // Percent bond for retirement account
    pub brokerage_cash_add: f32,      // Amount of cash added to brokerage account
    pub brokerage_us_stock_add: f32,
    pub brokerage_us_bond_add: f32,
    pub brokerage_int_stock_add: f32,
    pub brokerage_int_bond_add: f32,
    pub traditional_cash_add: f32,
    pub traditional_us_stock_add: f32,
    pub traditional_us_bond_add: f32,
    pub traditional_int_stock_add: f32,
    pub traditional_int_bond_add: f32,
    pub roth_cash_add: f32,
    pub roth_us_stock_add: f32,
    pub roth_us_bond_add: f32,
    pub roth_int_stock_add: f32,
    pub roth_int_bond_add: f32,
    pub brok_acct_option: Option<u32>, // Vanguard brokerage account number
    pub trad_acct_option: Option<u32>, // Vanguard traditional IRA account number
    pub roth_acct_option: Option<u32>, // Vanguard roth IRA account number
    pub output: bool,                  // Whether or not to output calculations to a txt file
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
                Arg::with_name("retirement-year")
                    .long("retirement-year")
                    .short("Y")
                    .takes_value(true)
                    .help("Retirment year in the format of YYYY"),
            )
            .arg(
                Arg::with_name("percent-bond-brokerage")
                    .long("bond-percent-brokerage")
                    .takes_value(true)
                    .default_value("40")
                    .help("Percentage to allocate in bonds in the brokerage account"),
            )
            .arg(
                Arg::with_name("percent-stock-brokerage")
                    .long("stock-percent-brokerage")
                    .takes_value(true)
                    .default_value("60")
                    .help("Percentage to allocate in stocks in the brokerage account"),
            )
            .arg(
                Arg::with_name("percent-bond-retirement")
                    .long("bond-percent-retirement")
                    .takes_value(true)
                    .help("Percentage to allocate in bonds in the retirement account"),
            )
            .arg(
                Arg::with_name("percent-stock-retirement")
                    .long("stock-percent-retirement")
                    .takes_value(true)
                    .help("Percentage to allocate in stocks in the retirement account"),
            )
            .arg(
                Arg::with_name("add-cash-brokerage")
                    .long("adjust-cash-brokerage")
                    .short("B")
                    .allow_hyphen_values(true)
                    .takes_value(true)
                    .default_value("0")
                    .help("Amount of cash added to or withdraw from the brokerage account"),
            )
            .arg(
                Arg::with_name("add-us-stock-brokerage")
                    .long("add-us-stock-brokerage")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of brokerage US stock held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-us-bond-brokerage")
                    .long("add-us-bond-brokerage")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of brokerage US bond held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-int-stock-brokerage")
                    .long("add-int-stock-brokerage")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of brokerage international stock held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-int-bond-brokerage")
                    .long("add-int-bond-brokerage")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of brokerage international bond held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-cash-traditional")
                    .long("adjust-cash-traditional")
                    .short("T")
                    .allow_hyphen_values(true)
                    .takes_value(true)
                    .default_value("0")
                    .help("Amount of cash added to or withdraw from the traditional IRA account"),
            )
            .arg(
                Arg::with_name("add-us-stock-traditional")
                    .long("add-us-stock-traditional")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of traditional IRA US stock held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-us-bond-traditional")
                    .long("add-us-bond-traditional")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of traditional IRA US bond held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-int-stock-traditional")
                    .long("add-int-stock-traditional")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of traditional IRA international stock held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-int-bond-traditional")
                    .long("add-int-bond-traditional")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of traditional IRA international bond held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-cash-roth")
                    .long("adjust-cash-roth")
                    .short("R")
                    .allow_hyphen_values(true)
                    .takes_value(true)
                    .default_value("0")
                    .help("Amount of cash added to or withdraw from the roth IRA account"),
            )
            .arg(
                Arg::with_name("add-us-stock-roth")
                    .long("add-us-stock-roth")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of roth IRA US stock held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-us-bond-roth")
                    .long("add-us-bond-roth")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of roth IRA US bond held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-int-stock-roth")
                    .long("add-int-stock-roth")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of roth IRA international stock held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("add-int-bond-roth")
                    .long("add-int-bond-roth")
                    .takes_value(true)
                    .default_value("0")
                    .help("Value of roth IRA international bond held outside of Vanguard.  Used for calculating stock/bond ratios."),
            )
            .arg(
                Arg::with_name("acct-num-b")
                    .long("brokerage-acct")
                    .short("b")
                    .takes_value(true)
                    .required_unless_one(&["acct-num-r", "acct-num-t"])
                    .help("Brokerage account number"),
            )
            .arg(
                Arg::with_name("acct-num-r")
                    .long("roth-acct")
                    .short("r")
                    .requires("retirement-year")
                    .takes_value(true)
                    .required_unless_one(&["acct-num-b", "acct-num-t"])
                    .help("Roth IRA account number"),
            )
            .arg(
                Arg::with_name("acct-num-t")
                    .long("trad-acct")
                    .short("t")
                    .requires("retirement-year")
                    .takes_value(true)
                    .required_unless_one(&["acct-num-b", "acct-num-r"])
                    .help("Traditional IRA account number"),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(false)
                    .help("Output to text file in current directory"),
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
        let brokerage_cash_add = args
            .value_of("add-cash-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let brokerage_us_stock_add = args
            .value_of("add-us-stock-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let brokerage_us_bond_add = args
            .value_of("add-us-bond-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let brokerage_int_stock_add = args
            .value_of("add-int-stock-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let brokerage_int_bond_add = args
            .value_of("add-int-bond-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();

        let traditional_cash_add = args
            .value_of("add-cash-traditional")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let traditional_us_stock_add = args
            .value_of("add-us-stock-traditional")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let traditional_us_bond_add = args
            .value_of("add-us-bond-traditional")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let traditional_int_stock_add = args
            .value_of("add-int-stock-traditional")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let traditional_int_bond_add = args
            .value_of("add-int-bond-traditional")
            .unwrap()
            .parse::<f32>()
            .unwrap();

        let roth_cash_add = args
            .value_of("add-cash-roth")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let roth_us_stock_add = args
            .value_of("add-us-stock-roth")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let roth_us_bond_add = args
            .value_of("add-us-bond-roth")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let roth_int_stock_add = args
            .value_of("add-int-stock-roth")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let roth_int_bond_add = args
            .value_of("add-int-bond-roth")
            .unwrap()
            .parse::<f32>()
            .unwrap();

        let mut retirement_year_option = None;
        if let Some(retirement_year) = args.value_of("retirement-year") {
            retirement_year_option = Some(retirement_year.parse::<i32>().unwrap())
        }
        let mut percent_stock_retirement_option = None;
        if let Some(percent_stock_retirement) = args.value_of("percent-stock-retirement") {
            percent_stock_retirement_option = Some(percent_stock_retirement.parse::<f32>().unwrap())
        }
        let mut percent_bond_retirement_option = None;
        if let Some(percent_bond_retirement) = args.value_of("percent-bond-retirement") {
            percent_bond_retirement_option = Some(percent_bond_retirement.parse::<f32>().unwrap())
        }

        let mut brok_acct_option = None;
        if let Some(brok_acct_str) = args.value_of("acct-num-b") {
            brok_acct_option = Some(brok_acct_str.parse::<u32>().unwrap())
        }
        let mut trad_acct_option = None;
        if let Some(trad_acct_str) = args.value_of("acct-num-t") {
            trad_acct_option = Some(trad_acct_str.parse::<u32>().unwrap())
        }
        let mut roth_acct_option = None;
        if let Some(roth_acct_str) = args.value_of("acct-num-r") {
            roth_acct_option = Some(roth_acct_str.parse::<u32>().unwrap())
        }
        let output = args.is_present("output");
        Args {
            csv_path,
            retirement_year_option,
            percent_stock_brokerage,
            percent_bond_brokerage,
            percent_stock_retirement_option,
            percent_bond_retirement_option,
            brokerage_cash_add,
            brokerage_us_stock_add,
            brokerage_us_bond_add,
            brokerage_int_stock_add,
            brokerage_int_bond_add,
            traditional_cash_add,
            traditional_us_stock_add,
            traditional_us_bond_add,
            traditional_int_stock_add,
            traditional_int_bond_add,
            roth_cash_add,
            roth_us_stock_add,
            roth_us_bond_add,
            roth_int_stock_add,
            roth_int_bond_add,
            brok_acct_option,
            trad_acct_option,
            roth_acct_option,
            output,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}
