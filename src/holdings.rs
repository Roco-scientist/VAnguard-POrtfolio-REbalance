use crate::asset::SubAllocations;
use anyhow::{anyhow, Result};
use chrono::{NaiveDate, Local, Datelike};
use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{BufRead, BufReader},
    ops::{Add, Div, Sub},
    vec::Vec,
};
use time::{macros::format_description, OffsetDateTime};
use yahoo_finance_api as yahoo;

// STOCK_DESCRIPTION holds the descriptions for the stock symbols which is used to print and
// display
lazy_static! {
    static ref STOCK_DESCRIPTION: HashMap<StockSymbol, &'static str> = {
        let mut m = HashMap::new();
        m.insert(StockSymbol::VV, "US large cap");
        m.insert(StockSymbol::VO, "US mid cap");
        m.insert(StockSymbol::VB, "US small cap");
        m.insert(StockSymbol::VTC, "US total corporate bond");
        m.insert(StockSymbol::BND, "US total bond");
        m.insert(StockSymbol::VXUS, "Total international stock");
        m.insert(StockSymbol::VWO, "Emerging markets stock");
        m.insert(StockSymbol::BNDX, "Total international bond");
        m.insert(StockSymbol::VTIP, "Inflation protected securities");
        m.insert(StockSymbol::VTI, "Total domestic stock");
        m.insert(StockSymbol::VTIVX, "2045 Retirement fund");
        m
    };
}

/// StockSymbol is an enum which holds all stock symbols which are supported.  Empty is used to
/// initiated structs which use this enum.  Other<String> is a holder of any stock that is not
/// supported, where the String is the stock symbol.
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum StockSymbol {
    VXUS,
    BNDX,
    VTIP,
    BND,
    VWO,
    VO,
    VB,
    VTC,
    VV,
    VMFXX,
    VTI,
    VTIVX,
    Empty,
    Other(String),
}

impl StockSymbol {
    /// new creates a new StockSymbol enum based on the string value.
    ///
    ///  # Example
    ///
    ///  ```
    ///  use vapore::holdings::StockSymbol;
    ///
    ///  let bnd = StockSymbol::new("BND");
    ///  assert_eq!(bnd, StockSymbol::BND);
    ///  ```
    pub fn new(symbol: &str) -> Self {
        match symbol {
            "VXUS" => StockSymbol::VXUS,
            "BNDX" => StockSymbol::BNDX,
            "VTIP" => StockSymbol::VTIP,
            "BND" => StockSymbol::BND,
            "VWO" => StockSymbol::VWO,
            "VO" => StockSymbol::VO,
            "VB" => StockSymbol::VB,
            "VTC" => StockSymbol::VTC,
            "VV" => StockSymbol::VV,
            "VMFXX" => StockSymbol::VMFXX,
            "VTI" => StockSymbol::VTI,
            "VTIVX" => StockSymbol::VTIVX,
            _ => {
                eprintln!("{} is not supported within this algorithm\n", symbol);
                StockSymbol::Other(symbol.to_string())
            }
        }
    }

    /// description returns a string of the StockSymbol description.  If the stock is not
    /// supported, a "No description" String is returned.
    ///
    /// # Example
    ///
    /// ```
    ///  use vapore::holdings::StockSymbol;
    ///
    ///  let bnd = StockSymbol::new("BND");
    ///  let bnd_description = bnd.description();
    ///  assert_eq!(bnd_description, "BND: US total bond");
    ///
    /// ```
    pub fn description(&self) -> String {
        let description_option = STOCK_DESCRIPTION.get(self);
        if let Some(description) = description_option {
            return format!("{:?}: {}", self, description);
        } else {
            return format!("No description for {:?}", self);
        }
    }
}

/// all_stock_descriptions returns a String containing the description of all stocks which are
/// supported with each separated by a new line.  This is used to display on screen or write to
/// file all of the descriptions.
///
/// # Example
///
/// ```
/// use vapore::holdings;
///
/// let descriptions = holdings::all_stock_descriptions();
/// println!("{}", descriptions);
///
/// ```
pub fn all_stock_descriptions() -> String {
    let mut descriptions = String::new();
    for symbol in [
        StockSymbol::VV,
        StockSymbol::VO,
        StockSymbol::VB,
        StockSymbol::VTC,
        StockSymbol::BND,
        StockSymbol::VXUS,
        StockSymbol::VWO,
        StockSymbol::BNDX,
        StockSymbol::VTIP,
        StockSymbol::VTI,
        StockSymbol::VTIVX,
    ] {
        descriptions.push_str(&symbol.description());
        descriptions.push('\n')
    }
    descriptions.pop();
    descriptions
}

#[derive(Clone)]
pub struct StockInfo {
    pub account_number: u32,
    pub symbol: StockSymbol,
    pub share_price: f32,
    pub shares: f32,
    pub total_value: f32,
    account_added: bool,
    symbol_added: bool,
    share_price_added: bool,
    shares_added: bool,
    total_value_added: bool,
}

impl StockInfo {
    /// new initializes a new StockInfo struct.  Account number, symbol, share price etc. can then
    /// be added with the other methods.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// ```
    pub fn new() -> Self {
        StockInfo {
            account_number: 0,
            symbol: StockSymbol::Empty,
            share_price: 0.0,
            shares: 0.0,
            total_value: 0.0,
            account_added: false,
            symbol_added: false,
            share_price_added: false,
            shares_added: false,
            total_value_added: false,
        }
    }

    /// add_account adds the vanguard account number to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    ///
    /// assert_eq!(new_stock.account_number, 123456789);
    /// ```
    pub fn add_account(&mut self, account_number: u32) {
        self.account_number = account_number;
        self.account_added = true;
    }

    /// add_symbol adds the stock symbol to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    ///
    /// assert_eq!(new_stock.symbol, holdings::StockSymbol::BND);
    /// ```
    pub fn add_symbol(&mut self, symbol: StockSymbol) {
        self.symbol = symbol;
        self.symbol_added = true;
    }

    /// add_share_price adds the stock quote price to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    ///
    /// assert_eq!(new_stock.share_price, 234.50);
    /// ```
    pub fn add_share_price(&mut self, share_price: f32) {
        self.share_price = share_price;
        self.share_price_added = true;
    }

    /// add_share adds the stock total shares to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    /// new_stock.add_shares(10.0)
    ///
    /// assert_eq!(new_stock.shares, 10.0);
    /// ```
    pub fn add_shares(&mut self, share_num: f32) {
        self.shares = share_num;
        self.shares_added = true;
    }

    /// add_total_value adds the account total value of the stock to the StockInfo struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    /// new_stock.add_total_value(5000.00);
    ///
    /// assert_eq!(new_stock.total_value, 5000.00);
    /// ```
    pub fn add_total_value(&mut self, total_value: f32) {
        self.total_value = total_value;
        self.total_value_added = true;
    }

    /// finished returns a bool of whether or not all struct values have been added.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    /// new_stock.add_total_value(5000.00);
    /// new_stock.add_shares(10.0);
    ///
    /// assert!(new_stock.finished());
    ///
    /// let empty_stock = holdings::StockInfo::new();
    /// assert!(!empty_stock.finished())
    /// ```
    pub fn finished(&self) -> bool {
        [
            self.account_added,
            self.symbol_added,
            self.share_price_added,
            self.shares_added,
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

pub async fn get_yahoo_quote(stock_symbol: StockSymbol) -> Result<f32> {
    let stock_str = match stock_symbol {
        StockSymbol::VO => "VO",
        StockSymbol::VB => "VB",
        StockSymbol::VV => "VV",
        StockSymbol::BND => "BND",
        StockSymbol::VWO => "VWO",
        StockSymbol::VTC => "VTC",
        StockSymbol::VXUS => "VXUS",
        StockSymbol::BNDX => "BNDX",
        StockSymbol::VTIP => "VTIP",
        StockSymbol::VTI => "VTI",
        StockSymbol::VTIVX => "VTIVX",
        _ => "none",
    };
    if stock_str == "none" {
        anyhow!("Stock symbol not supported for yahoo retrieval");
        Ok(0.0)
    }else{
        let provider = yahoo::YahooConnector::new();
        let response = provider.get_latest_quotes(stock_str, "1m").await?;
        Ok(response.last_quote()?.close as f32)
    }
}

pub async fn get_yahoo_eoy_quote(stock_symbol: StockSymbol) -> Result<f32> {
    let stock_str = match stock_symbol {
        StockSymbol::VO => "VO",
        StockSymbol::VB => "VB",
        StockSymbol::VV => "VV",
        StockSymbol::BND => "BND",
        StockSymbol::VWO => "VWO",
        StockSymbol::VTC => "VTC",
        StockSymbol::VXUS => "VXUS",
        StockSymbol::BNDX => "BNDX",
        StockSymbol::VTIP => "VTIP",
        StockSymbol::VTI => "VTI",
        StockSymbol::VTIVX => "VTIVX",
        _ => "none",
    };
    if stock_str == "none" {
        anyhow!("Stock symbol not supported for yahoo retrieval");
        Ok(0.0)
    }else{
        let provider = yahoo::YahooConnector::new();
        let format = format_description!( "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]");
        let today = OffsetDateTime::now_utc();
        let start = OffsetDateTime::parse(&format!("{}-12-25 00:00:01 -05", today.year() - 1), format)?;
        let stop = OffsetDateTime::parse(&format!("{}-12-31 23:59:59 -05", today.year() - 1),format)?;
        let response = provider.get_quote_history(stock_str, start, stop).await?;
        Ok(response.quotes()?.last().unwrap().close as f32)
    }
}

/// AddType is an enum used to distinguish between when a stock quote or an account holdings is
/// wanted for input into a ShareValues struct.
pub enum AddType {
    StockPrice,
    HoldingValue,
}

/// ShareValues holds the values for the supported ETF stocks.  The value can represent price,
/// holding value, stock quantity etc.
#[derive(Clone, PartialEq, Debug, Copy)]
pub struct ShareValues {
    vxus: f32,
    bndx: f32,
    bnd: f32,
    vwo: f32,
    vo: f32,
    vb: f32,
    vtc: f32,
    vv: f32,
    vtip: f32,
    vti: f32,
    vtivx: f32,
    vmfxx: f32,
    outside_bond: f32,
    outside_stock: f32,
}

impl ShareValues {
    /// new creates a new ShareValues struct where all values are set to 0.  This is used within
    /// vapore to create a new struct for account holdings, etc.
    ///
    /// # Example
    /// ```
    /// use vapore::holdings;
    ///
    /// let new_values = holdings::ShareValues::new();
    /// ```
    pub fn new() -> Self {
        ShareValues {
            vxus: 0.0,
            bndx: 0.0,
            vtip: 0.0,
            vti: 0.0,
            vtivx: 0.0,
            bnd: 0.0,
            vwo: 0.0,
            vo: 0.0,
            vb: 0.0,
            vtc: 0.0,
            vv: 0.0,
            vmfxx: 0.0,
            outside_bond: 0.0,
            outside_stock: 0.0,
        }
    }
    /// new_quote creates a new ShareValues struct where all values are set to 1.  This is used for
    /// creating a new struct for stock quotes.  This way if any quotes are missing, they are
    /// automatically set to 1 to prevent any 0 division errors.  This also has the effect of
    /// outputting the dollar amount when target value is divided by quote price.  This division
    /// occurs to determine number of stocks to purchase/sell.
    ///
    /// # Example
    /// ```
    /// use vapore::holdings;
    ///
    /// let new_quotes = holdings::ShareValues::new_quote();
    /// ```
    pub fn new_quote() -> Self {
        ShareValues {
            vxus: 1.0,
            bndx: 1.0,
            vtip: 1.0,
            vti: 1.0,
            vtivx: 1.0,
            bnd: 1.0,
            vwo: 1.0,
            vo: 1.0,
            vb: 1.0,
            vtc: 1.0,
            vv: 1.0,
            vmfxx: 1.0,
            outside_bond: 1.0,
            outside_stock: 1.0,
        }
    }

    pub async fn add_missing_quotes(&mut self) -> Result<()> {
        for stock_symbol in [
            StockSymbol::VV,
            StockSymbol::VO,
            StockSymbol::VB,
            StockSymbol::VTC,
            StockSymbol::BND,
            StockSymbol::VXUS,
            StockSymbol::VWO,
            StockSymbol::BNDX,
            StockSymbol::VTIP,
            StockSymbol::VTI,
            StockSymbol::VTIVX,
        ] {
            if self.stock_value(stock_symbol.clone()) == 1.0 {
                let new_quote = get_yahoo_quote(stock_symbol.clone()).await?;
                self.add_stock_value(stock_symbol, new_quote);
            }
        }
        Ok(())
    }

    pub async fn add_missing_eoy_quotes(&mut self) -> Result<()> {
        for stock_symbol in [
            StockSymbol::VV,
            StockSymbol::VO,
            StockSymbol::VB,
            StockSymbol::VTC,
            StockSymbol::BND,
            StockSymbol::VXUS,
            StockSymbol::VWO,
            StockSymbol::BNDX,
            StockSymbol::VTIP,
            StockSymbol::VTI,
            StockSymbol::VTIVX,
        ] {
            if self.stock_value(stock_symbol.clone()) == 1.0 {
                let new_quote = get_yahoo_eoy_quote(stock_symbol.clone()).await?;
                self.add_stock_value(stock_symbol, new_quote);
            }
        }
        Ok(())
    }

    /// new_target creates a new target ShareValues struct which determines what to what values to
    /// rebalance to vanguard portfolio.
    ///
    /// # Panic
    ///
    /// Panics when the percentages and fractions do not add up to 1 when they are added together.
    /// This is necessary to make sure everything adds up to 100% of the total portfolio.  Adding
    /// up to less or more than 100% can happen when the const values determining balance distribution
    /// are changed without changing other values to make sure everything adds up.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::{asset, holdings};
    ///
    /// let sub_allocations = asset::SubAllocations::new().unwrap();
    ///
    /// let brokerage_target = holdings::ShareValues::new_target(sub_allocations, 10000.0, 0.0, 0.0, 0.0, 0.0);
    /// ```
    pub fn new_target(
        sub_allocations: SubAllocations,
        total_vanguard_value: f32,
        other_us_stock_value: f32,
        other_us_bond_value: f32,
        other_int_stock_value: f32,
        other_int_bond_value: f32,
    ) -> Self {
        // get total value
        let total_value = total_vanguard_value
            + other_us_stock_value
            + other_us_bond_value
            + other_int_bond_value
            + other_int_stock_value;

        // Calculate values for each stock
        let vxus_value = (total_value * sub_allocations.int_tot_stock / 100.0)
            - (other_int_stock_value * 2.0 / 3.0);
        let bndx_value = (total_value * sub_allocations.int_bond / 100.0) - other_int_bond_value;
        let bnd_value =
            (total_value * sub_allocations.us_tot_bond / 100.0) - (other_us_bond_value / 2.0);
        let vwo_value = (total_value * sub_allocations.int_emerging_stock / 100.0)
            - (other_int_stock_value / 3.0);
        let vo_value =
            (total_value * sub_allocations.us_stock_mid / 100.0) - (other_us_stock_value / 3.0);
        let vb_value =
            (total_value * sub_allocations.us_stock_small / 100.0) - (other_us_stock_value / 3.0);
        let vtc_value =
            (total_value * sub_allocations.us_corp_bond / 100.0) - (other_us_bond_value / 2.0);
        let vv_value =
            (total_value * sub_allocations.us_stock_large / 100.0) - (other_us_stock_value / 3.0);
        let vtip_value = total_value * sub_allocations.inflation_protected / 100.0;

        // set vmfxx, ie cash, target value to 0 and return ShareValues
        ShareValues {
            vxus: vxus_value,
            bndx: bndx_value,
            bnd: bnd_value,
            vwo: vwo_value,
            vo: vo_value,
            vb: vb_value,
            vtc: vtc_value,
            vv: vv_value,
            vtip: vtip_value,
            vti: 0.0,
            vtivx: 0.0,
            vmfxx: 0.0,
            outside_bond: other_int_bond_value + other_us_bond_value,
            outside_stock: other_us_stock_value + other_int_stock_value,
        }
    }

    /// add_stockinfo_value adds stock value to the ShareValues struct with a StockInfo input.  StockInfo
    /// structs are constructed when parsing the CSV file downloaded from vangaurd.  This is used
    /// for both creating the stock quotes ShareValues struct and holding values ShareValuues
    /// struc.  The add_type is used to distinguish between these two groups to know where from
    /// within the StockInfo struct to pull the dollar amount from.
    ///
    /// # Panic
    ///
    /// Panics when an empty stock symbol is passed.  This will happen if the StockInfo struct is
    /// initialized without any content added.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_stock = holdings::StockInfo::new();
    /// new_stock.add_account(123456789);
    /// new_stock.add_symbol(holdings::StockSymbol::BND);
    /// new_stock.add_share_price(234.50);
    /// new_stock.add_total_value(5000.00);
    ///
    /// let mut new_quotes = holdings::ShareValues::new_quote();
    /// new_quotes.add_stockinfo_value(new_stock, holdings::AddType::StockPrice);
    ///
    /// assert_eq!(new_quotes.stock_value(holdings::StockSymbol::BND), 234.50);
    ///
    /// ```
    pub fn add_stockinfo_value(&mut self, stock_info: StockInfo, add_type: AddType) {
        let value;
        match add_type {
            AddType::StockPrice => value = stock_info.share_price,
            AddType::HoldingValue => value = stock_info.total_value,
        }
        match stock_info.symbol {
            StockSymbol::VXUS => self.vxus = value,
            StockSymbol::BNDX => self.bndx = value,
            StockSymbol::VTIP => self.vtip = value,
            StockSymbol::VTI => self.vti = value,
            StockSymbol::VTIVX => self.vtivx = value,
            StockSymbol::BND => self.bnd = value,
            StockSymbol::VWO => self.vwo = value,
            StockSymbol::VO => self.vo = value,
            StockSymbol::VB => self.vb = value,
            StockSymbol::VTC => self.vtc = value,
            StockSymbol::VV => self.vv = value,
            StockSymbol::VMFXX => self.vmfxx = value,
            StockSymbol::Empty => panic!("Stock symbol not set before adding value"),
            StockSymbol::Other(_) => (),
        }
    }

    /// add_stock_value adds stock value to the ShareValues struct with a float.  
    ///
    /// # Panic
    ///
    /// Panics when an empty stock symbol is passed.  This will happen if the StockInfo struct is
    /// initialized without any content added.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_values = holdings::ShareValues::new();
    /// new_values.add_stock_value(holdings::StockSymbol::BND, 5000.0);
    ///
    /// assert_eq!(new_values.stock_value(holdings::StockSymbol::BND), 5000.0);
    ///
    /// ```
    pub fn add_stock_value(&mut self, stock_symbol: StockSymbol, value: f32) {
        match stock_symbol {
            StockSymbol::VXUS => self.vxus = value,
            StockSymbol::BNDX => self.bndx = value,
            StockSymbol::VTIP => self.vtip = value,
            StockSymbol::VTI => self.vti = value,
            StockSymbol::VTIVX => self.vtivx = value,
            StockSymbol::BND => self.bnd = value,
            StockSymbol::VWO => self.vwo = value,
            StockSymbol::VO => self.vo = value,
            StockSymbol::VB => self.vb = value,
            StockSymbol::VTC => self.vtc = value,
            StockSymbol::VV => self.vv = value,
            StockSymbol::VMFXX => self.vmfxx = value,
            StockSymbol::Empty => panic!("Stock symbol not set before adding value"),
            StockSymbol::Other(_) => (),
        }
    }

    /// Adds other stock value that is not included within the vanguard account.  This is used for
    /// calculating current stock/bond ratios
    pub fn add_outside_stock_value(&mut self, stock_value: f32) {
        self.outside_stock = stock_value
    }

    pub fn outside_stock_value(&self) -> f32 {
        self.outside_stock
    }

    /// Adds other bond value that is not included within the vanguard account.  This is used for
    /// calculating current stock/bond ratios
    pub fn add_outside_bond_value(&mut self, bond_value: f32) {
        self.outside_bond = bond_value
    }

    pub fn outside_bond_value(&self) -> f32 {
        self.outside_bond
    }

    /// stock_value retrieves the stored stock value within the ShareValues struct
    ///
    /// # Panic
    ///
    /// Panics when an empty stock symbol is passed.  This will happen if the StockInfo struct is
    /// initialized without any content added.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_values = holdings::ShareValues::new();
    /// new_values.add_stock_value(holdings::StockSymbol::BND, 5000.0);
    ///
    /// assert_eq!(new_values.stock_value(holdings::StockSymbol::BND), 5000.0);
    ///
    /// ```
    pub fn stock_value(&self, stock_symbol: StockSymbol) -> f32 {
        match stock_symbol {
            StockSymbol::VXUS => self.vxus,
            StockSymbol::BNDX => self.bndx,
            StockSymbol::VTIP => self.vtip,
            StockSymbol::VTI => self.vti,
            StockSymbol::VTIVX => self.vtivx,
            StockSymbol::BND => self.bnd,
            StockSymbol::VWO => self.vwo,
            StockSymbol::VO => self.vo,
            StockSymbol::VB => self.vb,
            StockSymbol::VTC => self.vtc,
            StockSymbol::VV => self.vv,
            StockSymbol::VMFXX => self.vmfxx,
            StockSymbol::Empty => panic!("Value retrieval not supported for empty stock symbol"),
            StockSymbol::Other(symbol) => panic!("Value retrieval not supported for {}", symbol),
        }
    }

    /// total_value returns the sum of all of the values within the StockValue struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let mut new_values = holdings::ShareValues::new();
    /// new_values.add_stock_value(holdings::StockSymbol::BND, 5000.0);
    /// new_values.add_stock_value(holdings::StockSymbol::BNDX, 2000.0);
    /// new_values.add_stock_value(holdings::StockSymbol::VB, 4000.0);
    ///
    /// assert_eq!(new_values.total_value(), 11000.0);
    ///
    /// ```
    pub fn total_value(&self) -> f32 {
        self.vxus
            + self.bndx
            + self.bnd
            + self.vwo
            + self.vo
            + self.vb
            + self.vtc
            + self.vv
            + self.vmfxx
            + self.vtip
            + self.vti
            + self.vtivx
    }

    /// percent_stock_bond_infl calculates the percent of stock, bond, and inflation protected
    /// assets within the ShareValues.  This should only be used when the struct contains dollar
    /// value amounts for the stock values.
    pub fn percent_stock_bond_infl(&self) -> (f32, f32, f32) {
        let total_bond = self.bndx + self.bnd + self.vtc + self.outside_bond;
        let total_stock = self.vwo + self.vo + self.vb + self.vv + self.vxus + self.outside_stock;
        let total = self.total_value() - self.vmfxx + self.outside_bond + self.outside_stock;
        (
            total_stock / total * 100.0,
            total_bond / total * 100.0,
            self.vtip / total * 100.0,
        )
    }
}

impl Default for ShareValues {
    fn default() -> Self {
        Self::new()
    }
}

impl Add for ShareValues {
    type Output = ShareValues;

    fn add(self, other: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus + other.vxus,
            bndx: self.bndx + other.bndx,
            vtip: self.vtip + other.vtip,
            vti: self.vti + other.vti,
            vtivx: self.vtivx + other.vtivx,
            bnd: self.bnd + other.bnd,
            vwo: self.vwo + other.vwo,
            vo: self.vo + other.vo,
            vb: self.vb + other.vb,
            vtc: self.vtc + other.vtc,
            vv: self.vv + other.vv,
            vmfxx: self.vmfxx + other.vmfxx,
            outside_bond: self.outside_bond + other.outside_bond,
            outside_stock: self.outside_stock + other.outside_stock,
        }
    }
}

impl Sub for ShareValues {
    type Output = ShareValues;

    fn sub(self, other: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus - other.vxus,
            bndx: self.bndx - other.bndx,
            vtip: self.vtip - other.vtip,
            vti: self.vti - other.vti,
            vtivx: self.vtivx - other.vtivx,
            bnd: self.bnd - other.bnd,
            vwo: self.vwo - other.vwo,
            vo: self.vo - other.vo,
            vb: self.vb - other.vb,
            vtc: self.vtc - other.vtc,
            vv: self.vv - other.vv,
            vmfxx: self.vmfxx - other.vmfxx,
            outside_bond: self.outside_bond - other.outside_bond,
            outside_stock: self.outside_stock - other.outside_stock,
        }
    }
}

impl Div for ShareValues {
    type Output = ShareValues;

    fn div(self, other: ShareValues) -> ShareValues {
        ShareValues {
            vxus: self.vxus / other.vxus,
            bndx: self.bndx / other.bndx,
            vtip: self.vtip / other.vtip,
            vti: self.vti / other.vti,
            vtivx: self.vtivx / other.vtivx,
            bnd: self.bnd / other.bnd,
            vwo: self.vwo / other.vwo,
            vo: self.vo / other.vo,
            vb: self.vb / other.vb,
            vtc: self.vtc / other.vtc,
            vv: self.vv / other.vv,
            vmfxx: self.vmfxx / other.vmfxx,
            outside_bond: self.outside_bond / other.outside_bond,
            outside_stock: self.outside_stock / other.outside_stock,
        }
    }
}

impl fmt::Display for ShareValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (stock, bond, inflation) = self.percent_stock_bond_infl();
        write!(
            f,
            "\
            Symbol         Value\n\
            -------------------------------\n\
            VV               {:.2}\n\
            VO               {:.2}\n\
            VB               {:.2}\n\
            VTC              {:.2}\n\
            BND              {:.2}\n\
            VXUS             {:.2}\n\
            VWO              {:.2}\n\
            BNDX             {:.2}\n\
            VTIP             {:.2}\n\
            VTI              {:.2}\n\
            VTIVX            {:.2}\n\
            -------------------------------\n\
            Cash             {:.2}\n\
            Total            {:.2}\n\
            Outside stock    {:.2}\n\
            Outside bond     {:.2}\n\
            Stock:Bond:Infl  {:.1}:{:.1}:{:.1}\n\
            ===============================
            ",
            self.vv,
            self.vo,
            self.vb,
            self.vtc,
            self.bnd,
            self.vxus,
            self.vwo,
            self.bndx,
            self.vtip,
            self.vti,
            self.vtivx,
            self.vmfxx,
            self.total_value(),
            self.outside_stock,
            self.outside_bond,
            stock,
            bond,
            inflation
        )
    }
}

pub enum HoldingType {
    Brokerage,
    TraditionalIra,
    RothIra,
}

/// VanguardHoldings contains ShareValues structs for all accounts along with for the quotes.  This
/// struct is creating during the parsing of the downloaded Vanguard file
#[derive(Clone, Debug)]
pub struct VanguardHoldings {
    brokerage: Option<ShareValues>,
    traditional_ira: Option<ShareValues>,
    roth_ira: Option<ShareValues>,
    quotes: ShareValues,
    transactions: Vec<Transaction>,
    traditional_shares_option: Option<ShareValues>,
}

impl VanguardHoldings {
    /// new creates a new VanguardHoldings struct with the quotes added.  The rest of the accounts
    /// needs to be added later
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let new_quotes = holdings::ShareValues::new_quote();
    ///
    /// let mut new_vanguard = holdings::VanguardHoldings::new(new_quotes);
    /// ```
    pub fn new(quotes: ShareValues) -> Self {
        VanguardHoldings {
            brokerage: None,
            traditional_ira: None,
            roth_ira: None,
            quotes,
            transactions: Vec::new(),
            traditional_shares_option: None,
        }
    }

    /// add_holding adds a new account to the VanguardHoldings struct
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let new_quotes = holdings::ShareValues::new_quote();
    ///
    /// let new_values = holdings::ShareValues::new();
    ///
    /// let mut new_vanguard = holdings::VanguardHoldings::new(new_quotes);
    /// new_vanguard.add_holding(new_values, holdings::HoldingType::RothIra);
    /// ```
    pub fn add_holding(&mut self, holding: ShareValues, holding_type: HoldingType) {
        match holding_type {
            HoldingType::RothIra => self.roth_ira = Some(holding),
            HoldingType::Brokerage => self.brokerage = Some(holding),
            HoldingType::TraditionalIra => self.traditional_ira = Some(holding),
        }
    }

    /// add_holding adds a new account to the VanguardHoldings struct
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let new_quotes = holdings::ShareValues::new_quote();
    ///
    /// let new_values = holdings::ShareValues::new();
    ///
    /// let mut new_vanguard = holdings::VanguardHoldings::new(new_quotes);
    /// new_vanguard.add_holding(new_values, holdings::HoldingType::Brokerage);
    ///
    /// let new_values_comp = holdings::ShareValues::new();
    /// assert_eq!(new_vanguard.brokerage_holdings(), Some(new_values_comp));
    /// ```
    pub fn brokerage_holdings(&self) -> Option<ShareValues> {
        self.brokerage
    }
    pub fn traditional_ira_holdings(&self) -> Option<ShareValues> {
        self.traditional_ira
    }
    pub fn roth_ira_holdings(&self) -> Option<ShareValues> {
        self.roth_ira
    }
    pub fn stock_quotes(&self) -> ShareValues {
        self.quotes
    }
    pub async fn eoy_value(&self) -> Result<Option<f32>> {
        if let Some(_holdings) = self.eoy_traditional_holdings() {
            let mut quotes = ShareValues::new_quote();
            quotes.add_missing_eoy_quotes().await?;
            Ok(Some(0.0))
        }else{
            Ok(None)
        }

    }
    fn eoy_traditional_holdings(&self) -> Option<ShareValues> {
        let mut enough_transaction = false;
        if let Some(trad_holdings) = self.traditional_shares_option {
            if self.transactions.is_empty() {
                eprintln!("No transactions found to calculate EOY holdings for minimum distribution");
                None
            }else{
                let mut eoy_holdings = trad_holdings.clone();
                let today = Local::now().date();
                let previous_year = NaiveDate::from_ymd_opt(today.year() - 1, 12, 31)?;
                for transaction in &self.transactions {
                    if transaction.trade_date > previous_year {
                        eoy_holdings.add_stock_value(transaction.symbol.clone(), transaction.shares * -1.0);
                    }else{
                        enough_transaction = true;
                    }
                }
                if !enough_transaction {
                    eprintln!("Possibly not enough history in the downloaded Vanguard CSV to accurately calculate end of year holdings")
                }
                Some(eoy_holdings)
            }
        }else{
            None
        }
    }
}

/// AccountHoldings is a holder of current, target, and purchase/sales information for an account.
/// It also creates a Display for this information.
pub struct AccountHoldings {
    current: ShareValues,
    target: ShareValues,
    sale_purchases_needed: ShareValues,
}

impl AccountHoldings {
    /// new creates a new AccountHoldings struct from current, target, and sales/purchases
    /// Sharevalues structs.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::{asset, holdings};
    ///
    /// let sub_allocations = asset::SubAllocations::new().unwrap();
    ///
    /// let quotes = holdings::ShareValues::new_quote();
    ///
    /// let brokerage_current = holdings::ShareValues::new();
    /// let brokerage_target = holdings::ShareValues::new_target(sub_allocations, 10000.0, 0.0, 0.0, 0.0, 0.0);
    /// let purchase_sales = brokerage_current / quotes;
    ///
    /// let brokerage_account = holdings::AccountHoldings::new(brokerage_current, brokerage_target, purchase_sales);
    /// ```
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
}

impl fmt::Display for AccountHoldings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (current_stock, current_bond, current_inflation) =
            self.current.percent_stock_bond_infl();
        let current_stock_bond = format!(
            "{:.1}:{:.1}:{:.1}",
            current_stock, current_bond, current_inflation
        );

        let (target_stock, target_bond, target_inflation) = self.target.percent_stock_bond_infl();
        let target_stock_bond = format!(
            "{:.1}:{:.1}:{:.1}",
            target_stock, target_bond, target_inflation
        );

        write!(
            f,
            "Symbol   Purchase/Sell  Current         Target\n\
            ------------------------------------------------------\n\
            VV       {:<15.2}${:<15.2}${:<15.2}\n\
            VO       {:<15.2}${:<15.2}${:<15.2}\n\
            VB       {:<15.2}${:<15.2}${:<15.2}\n\
            VTC      {:<15.2}${:<15.2}${:<15.2}\n\
            BND      {:<15.2}${:<15.2}${:<15.2}\n\
            VXUS     {:<15.2}${:<15.2}${:<15.2}\n\
            VWO      {:<15.2}${:<15.2}${:<15.2}\n\
            BNDX     {:<15.2}${:<15.2}${:<15.2}\n\
            VTIP     {:<15.2}${:<15.2}${:<15.2}\n\
            VTI      {:<15.2}${:<15.2}${:<15.2}\n\
            VTIVX    {:<15.2}${:<15.2}${:<15.2}\n\
            ------------------------------------------------------\n\
            Cash                    ${:<15.2}${:<15.2}\n\
            Total                   ${:<15.2}\n\
            Outside stock           ${:<15.2}${:<15.2}\n\
            Outside bond            ${:<15.2}${:<15.2}\n\
            Stock:Bond:Inflation    {:<16}{:<15}\n\
            ======================================================",
            self.sale_purchases_needed.vv,
            self.current.vv,
            self.target.vv,
            self.sale_purchases_needed.vo,
            self.current.vo,
            self.target.vo,
            self.sale_purchases_needed.vb,
            self.current.vb,
            self.target.vb,
            self.sale_purchases_needed.vtc,
            self.current.vtc,
            self.target.vtc,
            self.sale_purchases_needed.bnd,
            self.current.bnd,
            self.target.bnd,
            self.sale_purchases_needed.vxus,
            self.current.vxus,
            self.target.vxus,
            self.sale_purchases_needed.vwo,
            self.current.vwo,
            self.target.vwo,
            self.sale_purchases_needed.bndx,
            self.current.bndx,
            self.target.bndx,
            self.sale_purchases_needed.vtip,
            self.current.vtip,
            self.target.vtip,
            self.sale_purchases_needed.vti,
            self.current.vti,
            self.target.vti,
            self.sale_purchases_needed.vtivx,
            self.current.vtivx,
            self.target.vtivx,
            self.current.vmfxx,
            self.target.vmfxx,
            self.current.total_value(),
            self.current.outside_stock,
            self.target.outside_stock,
            self.current.outside_bond,
            self.target.outside_bond,
            current_stock_bond,
            target_stock_bond,
        )
    }
}

/// VanguardRebalance holds AccountHoldings structs for each account; brokerage, traditional IRA,
/// and roth IRA.  Each AccountHoldings struct holds the information of current holdings, target
/// holdings, and the amount of stocks needed to purchase/sell in order to rebalance
pub struct VanguardRebalance {
    brokerage: Option<AccountHoldings>,
    traditional_ira: Option<AccountHoldings>,
    roth_ira: Option<AccountHoldings>,
    retirement_target: Option<ShareValues>,
}

impl VanguardRebalance {
    /// new creates a new empty VanguardRebalance struct
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::holdings;
    ///
    /// let vanguard_rebalance = holdings::VanguardRebalance::new();
    /// ```
    pub fn new() -> Self {
        VanguardRebalance {
            brokerage: None,
            traditional_ira: None,
            roth_ira: None,
            retirement_target: None,
        }
    }

    /// add_account_holdings adds either roth IRA, traditional IRA, or brokerage AccountHoldings
    /// struct to the current VanguardRebalance struct.
    ///
    /// # Example
    ///
    /// ```
    /// use vapore::{asset, holdings};
    ///
    /// let quotes = holdings::ShareValues::new_quote();
    ///
    /// let sub_allocations = asset::SubAllocations::new().unwrap();
    ///
    /// let brokerage_current = holdings::ShareValues::new();
    /// let brokerage_target = holdings::ShareValues::new_target(sub_allocations, 10000.0, 0.0, 0.0, 0.0, 0.0);
    /// let purchase_sales = brokerage_current / quotes;
    ///
    /// let brokerage_account = holdings::AccountHoldings::new(brokerage_current, brokerage_target, purchase_sales);
    ///
    /// let mut vanguard_rebalance = holdings::VanguardRebalance::new();
    /// vanguard_rebalance.add_account_holdings(brokerage_account, holdings::HoldingType::Brokerage);
    /// ```
    pub fn add_account_holdings(&mut self, acct_holding: AccountHoldings, acct_type: HoldingType) {
        match acct_type {
            HoldingType::Brokerage => self.brokerage = Some(acct_holding),
            HoldingType::TraditionalIra => self.traditional_ira = Some(acct_holding),
            HoldingType::RothIra => self.roth_ira = Some(acct_holding),
        }
    }

    pub fn add_retirement_target(&mut self, retirement_target: ShareValues) {
        self.retirement_target = Some(retirement_target);
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
        if let Some(retirement_target_values) = &self.retirement_target {
            out_string.push_str(&format!(
                "Retirement target:\n{}\n\n",
                retirement_target_values
            ))
        }
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

#[derive(Clone, Debug)]
pub struct Transaction {
    account_number: u32,
    trade_date: NaiveDate,
    // type: TransactionType,
    symbol: StockSymbol,
    shares: f32,
}

// #[derive(Clone, Eq, Hash, PartialEq, Debug)]
// enum TransactionType {
//     BUY,
//     SELL,
//     FUNDS_IN,
//     FOUNDS_OUT,
//     SWEEP_IN,
//     SWEEP_OUT,
//     DIVIDEND,
//     REINVESTMENT,
//     CONVERSION_OUT,
//     CONVERSION_IN,
// }

/// parse_csv_download takes in the file path of the downloaded file from Vanguard and parses it
/// into VanguardHoldings.  The VanguardHoldings is a struct which holds the values of what is
/// contained within the vangaurd account along with quotes for each of the ETFs
pub async fn parse_csv_download(
    csv_path: &str,
    args: crate::arguments::Args,
) -> Result<VanguardHoldings> {
    let mut header = Vec::new();
    let mut transaction_header = Vec::new();
    let csv_file = File::open(csv_path)?;
    let mut accounts: HashMap<u32, ShareValues> = HashMap::new();
    let mut quotes = ShareValues::new_quote();
    let mut traditional_shares_option: Option<ShareValues> = None;

    let mut holdings_row = true;
    let mut transactions = Vec::new();

    // iterate through all of the rows of the vanguard downlaoaded file and add the information to
    // StockInfo structs, which then are aggregated into the accounts hashmap where the account
    // number is the key
    for row_result in BufReader::new(csv_file).lines() {
        let row = row_result?;
        if row.contains(',') {
            if row.contains("Trade Date") {
                holdings_row = false;
            }
            let row_split = row
                .split(',')
                .map(|value| value.to_string())
                .collect::<Vec<String>>();
            if row_split.len() > 4 {
                if holdings_row {
                    let mut stock_info = StockInfo::new();
                    if header.is_empty() {
                        header = row_split
                    } else {
                        for (value, head) in row_split.iter().zip(&header) {
                            match head.as_str() {
                                "Account Number" => stock_info.add_account(value.parse::<u32>()?),
                                "Symbol" => if value.chars().count() > 1 {stock_info.add_symbol(StockSymbol::new(value))}else{break},
                                "Shares" => stock_info.add_shares(value.parse::<f32>()?),
                                "Share Price" => stock_info.add_share_price(value.parse::<f32>()?),
                                "Total Value" => stock_info.add_total_value(value.parse::<f32>()?),
                                _ => continue,
                            }
                        }
                        if stock_info.finished() {
                            let account_value = accounts
                                .entry(stock_info.account_number)
                                .or_insert_with(ShareValues::new);
                            account_value
                                .add_stockinfo_value(stock_info.clone(), AddType::HoldingValue);
                            quotes.add_stockinfo_value(stock_info.clone(), AddType::StockPrice);
                            if let Some(traditional_acct_num) = args.trad_acct_option {
                                if stock_info.account_number == traditional_acct_num {
                                    if let Some(mut traditional_shares) = traditional_shares_option {
                                        traditional_shares.add_stock_value(stock_info.symbol, stock_info.shares);
                                    }else{
                                        let mut shares = ShareValues::new();
                                        shares.add_stock_value(stock_info.symbol, stock_info.shares);
                                        traditional_shares_option = Some(shares);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    if transaction_header.is_empty() {
                        transaction_header = row_split
                    } else {
                        let mut account_num_option = None;
                        let mut trade_date_option = None;
                        let mut symbol_option = None;
                        let mut shares_option = None;
                        for (value, head) in row_split.iter().zip(&transaction_header) {
                            match head.as_str() {
                                "Account Number" => account_num_option = Some(value.parse::<u32>()?),
                                "Symbol" => if value.chars().count() > 1 {symbol_option = Some(StockSymbol::new(value))}else{break},
                                "Shares" => shares_option = Some(value.parse::<f32>()?),
                                "Trade Date" => {
                                    trade_date_option =
                                        Some(NaiveDate::parse_from_str(value, "%Y-%m-%d")?)
                                }
                                _ => continue,
                            }
                        }
                        if let Some(account_number) = account_num_option {
                            if let Some(symbol) = symbol_option {
                                if let Some(shares) = shares_option {
                                    if let Some(trade_date) = trade_date_option {
                                        if let Some(trad_account_num) = args.trad_acct_option {
                                            if trad_account_num == account_number {
                                                transactions.push(Transaction {
                                                    account_number,
                                                    symbol,
                                                    shares,
                                                    trade_date,
                                                })
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    quotes.add_missing_quotes().await?;

    let account_numbers = accounts.keys().cloned().collect::<Vec<u32>>();

    // if the brokerage account is input through CLI arguments, pull the data from the accounts
    // hashmap and place the information into a variable which will be input into the
    // VanguardHoldings struct
    let mut brokerage = None;
    if let Some(brokerage_acct) = args.brok_acct_option {
        if let Some(brokerage_holdings) = accounts.get(&brokerage_acct) {
            brokerage = Some(*brokerage_holdings)
        } else {
            return Err(anyhow!("{account_type} account number not found within vanguard download file\nInput account: {input:?}\nPossible accounts: {all_accounts:?}\n",
                               account_type= "Brokerage",
                               input=brokerage_acct,
                               all_accounts=account_numbers));
        }
    }

    // if the traditional IRA account is input through CLI arguments, pull the data from the accounts
    // hashmap and place the information into a variable which will be input into the
    // VanguardHoldings struct
    let mut traditional_ira = None;
    if let Some(traditional_acct) = args.trad_acct_option {
        if let Some(traditional_holdings) = accounts.get(&traditional_acct) {
            traditional_ira = Some(*traditional_holdings)
        } else {
            return Err(anyhow!("{account_type} account number not found within vanguard download file\nInput account: {input:?}\nPossible accounts: {all_accounts:?}\n",
                account_type= "Traditional IRA",
                input= traditional_acct,
                all_accounts= account_numbers)
            );
        }
    }

    // if the roth IRA account is input through CLI arguments, pull the data from the accounts
    // hashmap and place the information into a variable which will be input into the
    // VanguardHoldings struct
    let mut roth_ira = None;
    if let Some(roth_acct) = args.roth_acct_option {
        if let Some(roth_holdings) = accounts.get(&roth_acct) {
            roth_ira = Some(*roth_holdings)
        } else {
            return Err(anyhow!("{account_type:?} account number not found within vanguard download file\nInput account: {input:?}\nPossible accounts: {all_accounts:?}\n",
                account_type= "Roth IRA",
                input= roth_acct,
                all_accounts= account_numbers)
            );
        }
    }
    Ok(VanguardHoldings {
        brokerage,
        traditional_ira,
        roth_ira,
        quotes,
        transactions,
        traditional_shares_option,
    })
}
