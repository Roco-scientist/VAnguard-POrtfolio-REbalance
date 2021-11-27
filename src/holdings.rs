use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fs::File,
    io::{BufRead, BufReader},
    vec::Vec,
};

pub enum AddType {
    StockPrice,
    HoldingValue,
}

#[derive(Clone)]
pub struct ShareValues {
    pub vxus: f32,
    pub bndx: f32,
    pub vwo: f32,
    pub vo: f32,
    pub vb: f32,
    pub vtc: f32,
    pub vv: f32,
    pub vmfxx: f32,
    pub vtivx: f32,
}

impl ShareValues {
    pub fn new() -> Self {
        ShareValues {
            vxus: 0.0,
            bndx: 0.0,
            vwo: 0.0,
            vo: 0.0,
            vb: 0.0,
            vtc: 0.0,
            vv: 0.0,
            vmfxx: 0.0,
            vtivx: 0.0,
        }
    }
    pub fn add_value(&mut self, stock_info: StockInfo, add_type: AddType) {
        let value;
        match add_type {
            AddType::StockPrice => value = stock_info.share_price,
            AddType::HoldingValue => value = stock_info.total_value,
        }
        match stock_info.symbol.as_str() {
            "VXUS" => self.vxus = value,
            "BNDX" => self.bndx = value,
            "VWO" => self.vwo = value,
            "VO" => self.vo = value,
            "VB" => self.vb = value,
            "VTC" => self.vtc = value,
            "VV" => self.vv = value,
            "VMFXX" => self.vmfxx = value,
            "VTIVX" => self.vtivx = value,
            _ => (),
        }
    }

    pub fn total_value(&self) -> f32 {
        self.vxus
            + self.bndx
            + self.vwo
            + self.vo
            + self.vb
            + self.vtc
            + self.vv
            + self.vmfxx
            + self.vtivx
    }

    pub fn subtract(&self, other_value: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus - other_value.vxus,
            bndx: self.bndx - other_value.bndx,
            vwo: self.vwo - other_value.vwo,
            vo: self.vo - other_value.vo,
            vb: self.vb - other_value.vb,
            vtc: self.vtc - other_value.vtc,
            vv: self.vv - other_value.vv,
            vmfxx: self.vmfxx - other_value.vmfxx,
            vtivx: self.vtivx - other_value.vtivx,
        }
    }

    pub fn divide(&self, divisor: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus / divisor.vxus,
            bndx: self.bndx / divisor.bndx,
            vwo: self.vwo / divisor.vwo,
            vo: self.vo / divisor.vo,
            vb: self.vb / divisor.vb,
            vtc: self.vtc / divisor.vtc,
            vv: self.vv / divisor.vv,
            vmfxx: self.vmfxx / divisor.vmfxx,
            vtivx: self.vtivx / divisor.vtivx,
        }
    }
}

impl Default for ShareValues {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ShareValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VXUS: {}\nBNDX: {}\nVWO: {}\nVO: {}\nVB: {}\nVTC: {}\nVV: {}\nVMFXX: {}\nVTIVX: {}",
            self.vxus,
            self.bndx,
            self.vwo,
            self.vo,
            self.vb,
            self.vtc,
            self.vv,
            self.vmfxx,
            self.vtivx
        )
    }
}

#[derive(Clone)]
pub struct VanguardHoldings {
    brokerage: ShareValues,
    traditional_ira: ShareValues,
    roth_ira: ShareValues,
    quotes: ShareValues,
}

impl VanguardHoldings {
    pub fn brockerage_holdings(&self) -> ShareValues {
        self.brokerage.clone()
    }
    pub fn traditional_ira_holdings(&self) -> ShareValues {
        self.traditional_ira.clone()
    }
    pub fn roth_ira_holdings(&self) -> ShareValues {
        self.roth_ira.clone()
    }
    pub fn stock_quotes(&self) -> ShareValues {
        self.quotes.clone()
    }
}

#[derive(Clone)]
pub struct StockInfo {
    pub account_number: u32,
    pub symbol: String,
    pub share_price: f32,
    pub total_value: f32,
    account_added: bool,
    symbol_added: bool,
    share_price_added: bool,
    total_value_added: bool,
}

impl StockInfo {
    pub fn new() -> Self {
        StockInfo {
            account_number: 0,
            symbol: String::new(),
            share_price: 0.0,
            total_value: 0.0,
            account_added: false,
            symbol_added: false,
            share_price_added: false,
            total_value_added: false,
        }
    }
    pub fn add_account(&mut self, account_number: u32) {
        self.account_number = account_number;
        self.account_added = true;
    }
    pub fn add_symbol(&mut self, symbol: String) {
        self.symbol = symbol;
        self.symbol_added = true;
    }
    pub fn add_share_price(&mut self, share_price: f32) {
        self.share_price = share_price;
        self.share_price_added = true;
    }
    pub fn add_total_value(&mut self, total_value: f32) {
        self.total_value = total_value;
        self.total_value_added = true;
    }
    pub fn finished(&self) -> bool {
        [
            self.account_added,
            self.symbol_added,
            self.share_price_added,
            self.total_value_added,
        ]
        .iter()
        .all(|value| *value)
    }
}

impl Default for StockInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_csv_download(
    csv_path: &str,
    args: crate::arguments::Args,
) -> Result<VanguardHoldings, Box<dyn Error>> {
    let mut header = Vec::new();
    let csv_file = File::open(csv_path)?;
    let mut accounts: HashMap<u32, ShareValues> = HashMap::new();
    let mut stock_quotes = ShareValues::new();
    for row_result in BufReader::new(csv_file).lines() {
        let row = row_result?;
        if row.contains(',') {
            if row.contains("Trade Date") {
                break;
            }
            let row_split = row
                .split(',')
                .map(|value| value.to_string())
                .collect::<Vec<String>>();
            let mut stock_info = StockInfo::new();
            if header.is_empty() {
                header = row_split
            } else {
                for (value, head) in row_split.iter().zip(&header) {
                    match head.as_str() {
                        "Account Number" => stock_info.add_account(value.parse::<u32>()?),
                        "Symbol" => stock_info.add_symbol(value.to_string()),
                        "Share Price" => stock_info.add_share_price(value.parse::<f32>()?),
                        "Total Value" => stock_info.add_total_value(value.parse::<f32>()?),
                        _ => continue,
                    }
                }
                if stock_info.finished() {
                    let account_value = accounts
                        .entry(stock_info.account_number)
                        .or_insert_with(ShareValues::new);
                    account_value.add_value(stock_info.clone(), AddType::HoldingValue);
                    stock_quotes.add_value(stock_info, AddType::StockPrice);
                }
            }
        }
    }
    Ok(VanguardHoldings {
        brokerage: accounts.get(&args.brok_acct).unwrap().clone(),
        traditional_ira: accounts.get(&args.trad_acct).unwrap().clone(),
        roth_ira: accounts.get(&args.roth_acct).unwrap().clone(),
        quotes: stock_quotes,
    })
}
