use clap::{crate_version, App, Arg};

#[derive(Clone)]
pub struct Args {
    pub csv_path: String,
    pub percent_stock: f32,
    pub percent_bond: f32,
    pub brokerage_add: f32,
    pub brok_acct: u32,
    pub trad_acct: u32,
    pub roth_acct: u32,
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
                Arg::with_name("percent-bond")
                    .short("b")
                    .long("bond-percent")
                    .takes_value(true)
                    .default_value("40")
                    .help("Percentage to allocate in bonds"),
            )
            .arg(
                Arg::with_name("percent-stock")
                    .short("s")
                    .long("stock-percent")
                    .takes_value(true)
                    .default_value("60")
                    .help("Percentage to allocate in stocks"),
            )
            .arg(
                Arg::with_name("add-brokerage")
                    .long("add-brokerage")
                    .takes_value(true)
                    .default_value("0")
                    .help("Amount of cash added to the brokerage account"),
            )
            .arg(
                Arg::with_name("brokerage-acct")
                    .long("brokerage-acct")
                    .takes_value(true)
                    .required(true)
                    .help("Brokerage account number"),
            )
            .arg(
                Arg::with_name("roth-acct")
                    .long("roth-acct")
                    .takes_value(true)
                    .required(true)
                    .help("Roth IRA account number"),
            )
            .arg(
                Arg::with_name("trad-acct")
                    .long("trad-acct")
                    .takes_value(true)
                    .required(true)
                    .help("Traditional IRA account number"),
            )
            .get_matches();
        let csv_path = args.value_of("Vanguard-Download").unwrap().to_string();
        let percent_stock = args
            .value_of("percent-stock")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let percent_bond = args
            .value_of("percent-bond")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let brokerage_add = args
            .value_of("add-brokerage")
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let brok_acct = args.value_of("brokerage-acct").unwrap().parse::<u32>().unwrap();
        let trad_acct = args.value_of("trad-acct").unwrap().parse::<u32>().unwrap();
        let roth_acct = args.value_of("roth-acct").unwrap().parse::<u32>().unwrap();
        Args {
            csv_path,
            percent_stock,
            percent_bond,
            brokerage_add,
            brok_acct,
            trad_acct,
            roth_acct
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}
