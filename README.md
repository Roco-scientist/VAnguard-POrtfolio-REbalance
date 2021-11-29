# Vanguard-Buy
Algorithm setup to determine proper spread of Vanguard ETF index funds and adjust with the downloaded vanguards transaction file.  Current allocation:  
  
|Symbol|Description              |Type |% of type|
|------|-------------------------|-----|---------|
|VV    |US large cap stock       |Stock|22.222   |
|VO    |US mid cap stock         |Stock|22.222   |
|VB    |US small cap stock       |Stock|22.222   |
|VTC   |US total corp bond       |Bond |66.666   |
|VXUS  |Total international stock|Stock|22.222   |
|VWO   |Emerging markets stock   |Stock|11.111   |
|BNDX  |Total international bond |Bond |33.333   |
  
The value depends on overall distribution.  As in, 90% stock and 10% bond, then each percentation
would be that fraction of the stock vs. bond distribution.  The values above are stored within
constants in the holdings source file and can be changed there.  The default stock vs. bond distribution
is 90 vs 10 for retirement accounts and 60 vs 40 for brokerage investment accounts.  These can be changed 
through command line arguments.  The algorithm is also setup to shift the riskiest assets to the roth accounts
and the less risky assets to the traditional IRA accounts.  This is donw to allow for the most growth to happen
within the account which does not get taxed.

## How to run
### required
- Rust installed
- Vanguard account with money in it
- At least one of each ETF stock purchased:
  - VXUS, BNDX, VWO, VO, VB, VTC, VV

### Compile
Install and compile from source  
```
git clone https://github.com/Roco-scientist/VAnguard-POrtoflio-REbalance
cd VAnguard-POrtoflio-REbalance
cargo install --path .
```
  

Install and compile from crates.io  
`cargo install vapore`

### Download vanguard transactions

Download transaction file from within the vanguard account  
1. Login to vanguard
2. Click on `My accounts`
3. Click on `Transaction history`
4. Click the `download` button on the right hand side
5. For Step 1, select `A spreadsheet-compatible CSV file`
6. Step 2, leave at `1 month`
7. Step 3, select all accounts
8. Click `Download` located at the bottom right
9. Move the downloaded CSV file to where you want to run this program

### run
`vapore --brokerage-acct <#> --roth-acct <#> --trad-acct <#> <vanguard_csv>`  
  
If money is being added to any of the accounts, add one of the following flags along with the amount:
- --add-brokerage <#>
- --add-traditional <#>
- --add-roth <#>  
  
Where the latter two are for IRA additions
