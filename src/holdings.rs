use custom_error::custom_error;
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fs::File,
    io::{BufRead, BufReader, Write},
    vec::Vec,
};

// Constants used for proportion of portfolio contained within each.
// Split by stocks and bonds
// US stock as 2/3 of total stock.  Then split by 3 for Large, medium, and small cap
const US_STOCK_FRACTION: f32 = 2.0 / 3.0;
const EACH_US_STOCK: f32 = US_STOCK_FRACTION / 3.0;
// International stock as 1/3 of total stock.  Then 1/3 of that as emerging markets and 2/3 as
// total international
const INT_STOCK_FRACTION: f32 = 1.0 / 3.0;
const INT_EMERGING: f32 = INT_STOCK_FRACTION / 3.0;
const INT_TOTAL: f32 = INT_STOCK_FRACTION * 2.0 / 3.0;
// 2/3 of total bonds in US corporate bonds, 1/3 in internation bonds
const US_BOND_FRACTION: f32 = 2.0 / 3.0;
const INT_BOND_FRACTION: f32 = 1.0 / 3.0;

pub enum AddType {
    StockPrice,
    HoldingValue,
}

#[derive(Clone)]
pub enum StockSymbols {
    VXUS,
    BNDX,
    VWO,
    VO,
    VB,
    VTC,
    VV,
    VMFXX,
    Empty,
    Other(String),
}

impl StockSymbols {
    pub fn new(symbol: &str) -> Self {
        match symbol {
            "VXUS" => StockSymbols::VXUS,
            "BNDX" => StockSymbols::BNDX,
            "VWO" => StockSymbols::VWO,
            "VO" => StockSymbols::VO,
            "VB" => StockSymbols::VB,
            "VTC" => StockSymbols::VTC,
            "VV" => StockSymbols::VV,
            "VMFXX" => StockSymbols::VMFXX,
            _ => StockSymbols::Other(symbol.to_string()),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ShareValues {
    vxus: f32,
    bndx: f32,
    vwo: f32,
    vo: f32,
    vb: f32,
    vtc: f32,
    vv: f32,
    vmfxx: f32,
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
        }
    }
    pub fn new_quote() -> Self {
        ShareValues {
            vxus: 1.0,
            bndx: 1.0,
            vwo: 1.0,
            vo: 1.0,
            vb: 1.0,
            vtc: 1.0,
            vv: 1.0,
            vmfxx: 1.0,
        }
    }
    pub fn new_target(
        total_vanguard_value: f32,
        percent_bond: f32,
        percent_stock: f32,
        other_us_stock_value: f32,
        other_us_bond_value: f32,
        other_int_bond_value: f32,
        other_int_stock_value: f32,
    ) -> Self {
        let total_percent = INT_TOTAL * percent_stock / 100.0
            + INT_BOND_FRACTION * percent_bond / 100.0
            + INT_EMERGING * percent_stock / 100.0
            + EACH_US_STOCK * percent_stock / 100.0
            + EACH_US_STOCK * percent_stock / 100.0
            + US_BOND_FRACTION * percent_bond / 100.0
            + EACH_US_STOCK * percent_stock / 100.0;
        assert_eq!(total_percent, 1.0, "Fractions did not add up for brokerage account.  The bond to stock ratio is likely off and should add up to 100");
        let total_value = total_vanguard_value
            + other_us_stock_value
            + other_us_bond_value
            + other_int_bond_value
            + other_int_stock_value;
        ShareValues {
            vxus: (total_value * INT_TOTAL * percent_stock / 100.0)
                - (other_int_stock_value * 2.0 / 3.0),
            bndx: (total_value * INT_BOND_FRACTION * percent_bond / 100.0) - other_int_bond_value,
            vwo: (total_value * INT_EMERGING * percent_stock / 100.0)
                - (other_int_stock_value / 3.0),
            vo: (total_value * EACH_US_STOCK * percent_stock / 100.0)
                - (other_us_stock_value / 3.0),
            vb: (total_value * EACH_US_STOCK * percent_stock / 100.0)
                - (other_us_stock_value / 3.0),
            vtc: (total_value * US_BOND_FRACTION * percent_bond / 100.0) - (other_us_bond_value),
            vv: (total_value * EACH_US_STOCK * percent_stock / 100.0)
                - (other_us_stock_value / 3.0),
            vmfxx: 0.0,
        }
    }
    pub fn add_value(&mut self, stock_info: StockInfo, add_type: AddType) {
        let value;
        match add_type {
            AddType::StockPrice => value = stock_info.share_price,
            AddType::HoldingValue => value = stock_info.total_value,
        }
        match stock_info.symbol {
            StockSymbols::VXUS => self.vxus = value,
            StockSymbols::BNDX => self.bndx = value,
            StockSymbols::VWO => self.vwo = value,
            StockSymbols::VO => self.vo = value,
            StockSymbols::VB => self.vb = value,
            StockSymbols::VTC => self.vtc = value,
            StockSymbols::VV => self.vv = value,
            StockSymbols::VMFXX => self.vmfxx = value,
            StockSymbols::Empty => panic!("Stock symbol not set before adding value"),
            StockSymbols::Other(symbol) => eprintln!("Stock ticker not supported: {}", symbol),
        }
    }

    pub fn add_stock_value(&mut self, stock_symbol: StockSymbols, value: f32) {
        match stock_symbol {
            StockSymbols::VXUS => self.vxus = value,
            StockSymbols::BNDX => self.bndx = value,
            StockSymbols::VWO => self.vwo = value,
            StockSymbols::VO => self.vo = value,
            StockSymbols::VB => self.vb = value,
            StockSymbols::VTC => self.vtc = value,
            StockSymbols::VV => self.vv = value,
            StockSymbols::VMFXX => self.vmfxx = value,
            StockSymbols::Empty => panic!("Stock symbol not set before adding value"),
            StockSymbols::Other(symbol) => eprintln!("Stock ticker not supported: {}", symbol),
        }
    }

    pub fn stock_value(&self, stock_symbol: StockSymbols) -> f32 {
        match stock_symbol {
            StockSymbols::VXUS => self.vxus,
            StockSymbols::BNDX => self.bndx,
            StockSymbols::VWO => self.vwo,
            StockSymbols::VO => self.vo,
            StockSymbols::VB => self.vb,
            StockSymbols::VTC => self.vtc,
            StockSymbols::VV => self.vv,
            StockSymbols::VMFXX => self.vmfxx,
            StockSymbols::Empty => panic!("Value retrieval not supported for empty stock symbol"),
            StockSymbols::Other(symbol) => panic!("Value retrieval not supported for {}", symbol),
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
    }

    pub fn subtract(&self, other_value: &ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus - other_value.vxus,
            bndx: self.bndx - other_value.bndx,
            vwo: self.vwo - other_value.vwo,
            vo: self.vo - other_value.vo,
            vb: self.vb - other_value.vb,
            vtc: self.vtc - other_value.vtc,
            vv: self.vv - other_value.vv,
            vmfxx: self.vmfxx - other_value.vmfxx,
        }
    }

    pub fn add(&self, other_value: &ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus + other_value.vxus,
            bndx: self.bndx + other_value.bndx,
            vwo: self.vwo + other_value.vwo,
            vo: self.vo + other_value.vo,
            vb: self.vb + other_value.vb,
            vtc: self.vtc + other_value.vtc,
            vv: self.vv + other_value.vv,
            vmfxx: self.vmfxx + other_value.vmfxx,
        }
    }

    pub fn divide(&self, divisor: &ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus / divisor.vxus,
            bndx: self.bndx / divisor.bndx,
            vwo: self.vwo / divisor.vwo,
            vo: self.vo / divisor.vo,
            vb: self.vb / divisor.vb,
            vtc: self.vtc / divisor.vtc,
            vv: self.vv / divisor.vv,
            vmfxx: self.vmfxx / divisor.vmfxx,
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
            "VXUS     {:.2}\n\
            BNDX     {:.2}\n\
            VWO      {:.2}\n\
            VO       {:.2}\n\
            VB       {:.2}\n\
            VTC      {:.2}\n\
            VV       {:.2}\n\
            Cash     {:.2}\n\
            Total    {:.2}",
            self.vxus,
            self.bndx,
            self.vwo,
            self.vo,
            self.vb,
            self.vtc,
            self.vv,
            self.vmfxx,
            self.total_value(),
        )
    }
}

pub enum HoldingType {
    Brokerage,
    TraditionalIra,
    RothIra,
}

#[derive(Clone)]
pub struct VanguardHoldings {
    brokerage: Option<ShareValues>,
    traditional_ira: Option<ShareValues>,
    roth_ira: Option<ShareValues>,
    quotes: ShareValues,
}

impl VanguardHoldings {
    pub fn new(quotes: ShareValues) -> Self {
        VanguardHoldings {
            brokerage: None,
            traditional_ira: None,
            roth_ira: None,
            quotes,
        }
    }
    pub fn add_holding(&mut self, holding: ShareValues, holding_type: HoldingType) {
        match holding_type {
            HoldingType::RothIra => self.roth_ira = Some(holding),
            HoldingType::Brokerage => self.brokerage = Some(holding),
            HoldingType::TraditionalIra => self.traditional_ira = Some(holding),
        }
    }
    pub fn brockerage_holdings(&self) -> Option<ShareValues> {
        self.brokerage.clone()
    }
    pub fn traditional_ira_holdings(&self) -> Option<ShareValues> {
        self.traditional_ira.clone()
    }
    pub fn roth_ira_holdings(&self) -> Option<ShareValues> {
        self.roth_ira.clone()
    }
    pub fn stock_quotes(&self) -> ShareValues {
        self.quotes.clone()
    }
}

pub struct VanguardRebalance {
    brokerage: Option<AccountHoldings>,
    traditional_ira: Option<AccountHoldings>,
    roth_ira: Option<AccountHoldings>,
}

impl VanguardRebalance {
    pub fn new() -> Self {
        VanguardRebalance {
            brokerage: None,
            traditional_ira: None,
            roth_ira: None,
        }
    }
    pub fn add_account_holdings(&mut self, acct_holding: AccountHoldings, acct_type: HoldingType) {
        match acct_type {
            HoldingType::Brokerage => self.brokerage = Some(acct_holding),
            HoldingType::TraditionalIra => self.traditional_ira = Some(acct_holding),
            HoldingType::RothIra => self.roth_ira = Some(acct_holding),
        }
    }
}

impl Default for VanguardRebalance {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VanguardRebalance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out_string = String::new();
        if let Some(traditional_ira_account) = &self.traditional_ira {
            out_string.push_str(&format!(
                "Traditional IRA:\n{}\n\n",
                traditional_ira_account
            ))
        }
        if let Some(roth_ira_account) = &self.roth_ira {
            out_string.push_str(&format!("Roth IRA:\n{}\n\n", roth_ira_account))
        }
        if let Some(brokerage_account) = &self.brokerage {
            out_string.push_str(&format!("Brokerage:\n{}\n\n", brokerage_account))
        }
        write!(f, "{}", out_string.trim_end_matches('\n'))
    }
}

pub struct AccountHoldings {
    current: ShareValues,
    target: ShareValues,
    sale_purchases_needed: ShareValues,
}

impl AccountHoldings {
    pub fn new(
        current: ShareValues,
        target: ShareValues,
        sale_purchases_needed: ShareValues,
    ) -> Self {
        AccountHoldings {
            current,
            target,
            sale_purchases_needed,
        }
    }

    pub fn to_csv(&self, out: String) -> Result<(), Box<dyn Error>> {
        let out_text = format!(
            "symbol,purchase/sales,current,target\n\
            vxus,{},${},${}\n\
            bndx,{},${},${}\n\
            vwo,{},${},${}\n\
            vo,{},${},${}\n\
            vb,{},${},${}\n\
            vtc,{},${},${}\n\
            vv,{},${},${}\n\
            vmfxx,{},${},${}",
            self.sale_purchases_needed.vxus,
            self.current.vxus,
            self.target.vxus,
            self.sale_purchases_needed.bndx,
            self.current.bndx,
            self.target.bndx,
            self.sale_purchases_needed.vwo,
            self.current.vwo,
            self.target.vwo,
            self.sale_purchases_needed.vo,
            self.current.vo,
            self.target.vo,
            self.sale_purchases_needed.vb,
            self.current.vb,
            self.target.vb,
            self.sale_purchases_needed.vtc,
            self.current.vtc,
            self.target.vtc,
            self.sale_purchases_needed.vv,
            self.current.vv,
            self.target.vv,
            self.sale_purchases_needed.vmfxx,
            self.current.vmfxx,
            self.target.vmfxx,
        );
        let mut out_file = File::create(out)?;
        out_file.write_all(out_text.as_bytes())?;
        Ok(())
    }
}

impl fmt::Display for AccountHoldings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Symbol   Purchase/Sell  Current         Target\n\
            --------------------------------------------------\n\
            VXUS     {:<15.2}${:<15.2}${:<15.2}\n\
            BNDX     {:<15.2}${:<15.2}${:<15.2}\n\
            VWO      {:<15.2}${:<15.2}${:<15.2}\n\
            VO       {:<15.2}${:<15.2}${:<15.2}\n\
            VB       {:<15.2}${:<15.2}${:<15.2}\n\
            VTC      {:<15.2}${:<15.2}${:<15.2}\n\
            VV       {:<15.2}${:<15.2}${:<15.2}\n\
            --------------------------------------------------\n\
            Cash                    ${:<15.2}${:<15.2}\n\
            Total                   ${:<15.2}\n\
            ==================================================",
            self.sale_purchases_needed.vxus,
            self.current.vxus,
            self.target.vxus,
            self.sale_purchases_needed.bndx,
            self.current.bndx,
            self.target.bndx,
            self.sale_purchases_needed.vwo,
            self.current.vwo,
            self.target.vwo,
            self.sale_purchases_needed.vo,
            self.current.vo,
            self.target.vo,
            self.sale_purchases_needed.vb,
            self.current.vb,
            self.target.vb,
            self.sale_purchases_needed.vtc,
            self.current.vtc,
            self.target.vtc,
            self.sale_purchases_needed.vv,
            self.current.vv,
            self.target.vv,
            self.current.vmfxx,
            self.target.vmfxx,
            self.current.total_value()
        )
    }
}

#[derive(Clone)]
pub struct StockInfo {
    pub account_number: u32,
    pub symbol: StockSymbols,
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
            symbol: StockSymbols::Empty,
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
    pub fn add_symbol(&mut self, symbol: StockSymbols) {
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

custom_error! {AccountNumberError
    Brokerage = "Brokerage account number not found within vanguard download file",
    TraditionIra =  "Traditional IRA account number not found within vanguard download file",
    RothIra =  "Roth IRA account number not found within vanguard download file",
}

pub fn parse_csv_download(
    csv_path: &str,
    args: crate::arguments::Args,
) -> Result<VanguardHoldings, Box<dyn Error>> {
    let mut header = Vec::new();
    let csv_file = File::open(csv_path)?;
    let mut accounts: HashMap<u32, ShareValues> = HashMap::new();
    let mut quotes = ShareValues::new_quote();
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
                        "Symbol" => stock_info.add_symbol(StockSymbols::new(value)),
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
                    quotes.add_value(stock_info, AddType::StockPrice);
                }
            }
        }
    }
    let mut brokerage = None;
    if let Some(brokerage_acct) = args.brok_acct_option {
        if let Some(brokerage_holdings) = accounts.get(&brokerage_acct) {
            brokerage = Some(brokerage_holdings.clone())
        } else {
            return Err(Box::new(AccountNumberError::Brokerage));
        }
    }

    let mut traditional_ira = None;
    if let Some(traditional_acct) = args.trad_acct_option {
        if let Some(traditional_holdings) = accounts.get(&traditional_acct) {
            traditional_ira = Some(traditional_holdings.clone())
        } else {
            return Err(Box::new(AccountNumberError::TraditionIra));
        }
    }

    let mut roth_ira = None;
    if let Some(roth_acct) = args.roth_acct_option {
        if let Some(roth_holdings) = accounts.get(&roth_acct) {
            roth_ira = Some(roth_holdings.clone())
        } else {
            return Err(Box::new(AccountNumberError::RothIra));
        }
    }
    Ok(VanguardHoldings {
        brokerage,
        traditional_ira,
        roth_ira,
        quotes,
    })
}
