# Vanguard-Buy
Algorithm setup to determine proper spread of index funds within Vanguard-Buy  

## How to run
### required
- Rust installed
- Vanguard account with money in it

### Compile
```
git clone https://github.com/Roco-scientist/vanguard-buy
cd vanguard-buy
cargo install --path .
```

### Download vanguard transactions

Download transaction file from within the vanguard account  
1 Login to vanguard
2 Click on transaction history
3 Click on download button

### run
`vanguard-buy --brokerage-acct <#> --roth-acct <#> --trad-acct <#> <vanguard_csv>`
