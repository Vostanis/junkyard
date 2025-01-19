**spider** is the webscraper routine of **junkyard**; dealing in HTTP requests, data-deserialization, data-transformations, and data-loading.

----------------------------------------------------------------------------------------------------------------------------------------------

# Project Notes

Below is any of the projects, and subsequent notes, within the **spider** routine.

## 1. Standardized Financials (US)

AIM: To output a simplified table of common Company Financials;
    - Revenue
    - Earnings, USD & perc.
    - Earnings Per Share (EPS)
    - Free Cash Flow
    - Cash From Operations
    - Operating Expenses
    - Return on Asserts
    - Return on Equity
    - Debt
    - Equity
    - Debt vs. Equity, perc.
    - Outstanding Shares
    - Stock Buybacks

### Problem 1: Definitions
Within the table *stock.metrics* is a list of all the available US company financial information from Quarterly & Annual reporting.
The raw data has one main issue; there are a ***a lot*** of synonymous terms of each metric.

Depending on the company, definitions picked can vary; for example: [revenue] has been found to be one of: 
- "Revenues",
- "SalesRevenueNet", or 
- "RevenueFromContractWithCustomerExcludingAssessedTax".

- [ ] Revenue
    - Revenues
    - SalesRevenueNet
    - RevenueFromContractWithCustomerExcludingAssessedTax

- [ ] Earnings, USD & perc.
    - NetIncomeLoss

- [ ] Earnings Per Share (EPS)
    - EarningsPerShareBasic

- [ ] Free Cash Flow

- [ ] Cash From Operations

- [ ] Operating Expenses
    - OperatingExpenses (ResearchAndDevelopmentExpense)

- [ ] Return on Assets

- [ ] Return on Equity

- [ ] Debt
    - LongTermDebt
    - LongTermDebtNonCurrent
    - LongTermDebtCurrent
    - LiabilitiesCurrent
    - LiabilitiesNonCurrent
    - Liabilities (this appears to be the total)
    - DebtInstrumentCarryingAmount

- [ ] Equity
    - StockholdersEquity

- [ ] Outstanding Shares
    - EntityCommonStockSharesOutstanding

- [ ] Stock Buybacks
    - PaymentsForRepurchaseOfCommonStock
    - StocksRepurchasedDuringPeriodValue
    - StocksRepurchasedAndRetiredDuringPeriodValue
    - StocksRepurchasedDuringPeriodShares
    - StocksRepurchasedAndRetiredDuringPeriodShares

#### Useful terminology
[ Current Debt ]:       "Debt to be repaid within 12 months."\
[ Non-Current Debt ]:   "Debt to be repaid some time after the 12 month period."

### Problem 2: Insinuating missing inputs
For something reported quarterly, like Revenue, there typically exists 2 or 3 Quarterly reports in a row, followed by a 10-K
