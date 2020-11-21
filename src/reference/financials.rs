extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use crate::helpers::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Financial {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol: String,
  pub period: String,
  #[serde(deserialize_with="string_to_naive_date", serialize_with="naive_date_to_string")]
  pub calendar_date: NaiveDate,
  #[serde(deserialize_with="string_to_naive_date", serialize_with="naive_date_to_string")]
  pub report_period: NaiveDate,
  #[serde(deserialize_with="string_to_naive_date", serialize_with="naive_date_to_string")]
  pub updated: NaiveDate,
  #[serde(deserialize_with="string_to_naive_date", serialize_with="naive_date_to_string")]
  pub date_key: NaiveDate,
  pub accumulated_other_comprehensive_income: i64,
  pub assets: i64,
  pub assets_average: i64,
  pub assets_current: i64,
  pub asset_turnover: f64,
  pub assets_non_current: i64,
  pub book_value_per_share: f64,
  pub capital_expenditure: i64,
  pub cash_and_equivalents: i64,
  #[serde(rename(deserialize = "cashAndEquivalentsUSD"))]
  pub cash_and_equivalents_usd: i64,
  pub cost_of_revenue: i64,
  pub consolidated_income: i64,
  pub current_ratio: f64,
  pub debt_to_equity_ratio: f64,
  pub debt: i64,
  pub debt_current: i64,
  pub debt_non_current: i64,
  #[serde(rename(deserialize = "debtUSD"))]
  pub debt_usd: i64,
  pub deferred_revenue: i64,
  pub depreciation_amortization_and_accretion: i64,
  pub deposits: i64,
  pub dividend_yield: f64,
  pub dividends_per_basic_common_share: f64,
  pub earning_before_interest_taxes: i64,
  pub earnings_before_interest_taxes_depreciation_amortization: i64,
  #[serde(rename(deserialize = "EBITDAMargin"))]
  pub ebitda_margin: f64,
  #[serde(rename(deserialize = "earningsBeforeInterestTaxesDepreciationAmortizationUSD"))]
  pub earnings_before_interest_taxes_depreciation_amortization_usd: i64,
  #[serde(rename(deserialize = "earningBeforeInterestTaxesUSD"))]
  pub earning_before_interest_taxes_usd: i64,
  pub earnings_before_tax: i64,
  pub earnings_per_basic_share: f64,
  pub earnings_per_diluted_share: f64,
  #[serde(rename(deserialize = "earningsPerBasicShareUSD"))]
  pub earnings_per_basic_share_usd: f64,
  pub shareholders_equity: i64,
  pub average_equity: i64,
  #[serde(rename(deserialize = "shareholdersEquityUSD"))]
  pub shareholders_equity_usd: i64,
  pub enterprise_value: i64,
  #[serde(rename(deserialize = "enterpriseValueOverEBIT"))]
  pub enterprise_value_over_ebit: i64,
  #[serde(rename(deserialize = "enterpriseValueOverEBITDA"))]
  pub enterprise_value_over_ebitda: f64,
  pub free_cash_flow: i64,
  pub free_cash_flow_per_share: f64,
  #[serde(rename(deserialize = "foreignCurrencyUSdExchangeRate"))]
  pub foreign_currency_usd_exchange_rate: i64,
  pub gross_profit: i64,
  pub gross_margin: f64,
  pub goodwill_and_intangible_assets: i64,
  pub interest_expense: i64,
  pub invested_capital: i64,
  pub invested_capital_average: i64,
  pub inventory: i64,
  pub investments: i64,
  pub investments_current: i64,
  pub investments_non_current: i64,
  pub total_liabilities: i64,
  pub current_liabilities: i64,
  pub liabilities_non_current: i64,
  pub market_capitalization: i64,
  pub net_cash_flow: i64,
  pub net_cash_flow_business_acquisitions_disposals: i64,
  pub issuance_equity_shares: i64,
  pub issuance_debt_securities: i64,
  pub payment_dividends_other_cash_distributions: i64,
  pub net_cash_flow_from_financing: i64,
  pub net_cash_flow_from_investing: i64,
  pub net_cash_flow_investment_acquisitions_disposals: i64,
  pub net_cash_flow_from_operations: i64,
  pub effect_of_exchange_rate_changes_on_cash: i64,
  pub net_income: i64,
  pub net_income_common_stock: i64,
  #[serde(rename(deserialize = "netIncomeCommonStockUSD"))]
  pub net_income_common_stock_usd: i64,
  pub net_loss_income_from_discontinued_operations: i64,
  pub net_income_to_non_controlling_interests: i64,
  pub profit_margin: f64,
  pub operating_expenses: i64,
  pub operating_income: i64,
  pub trade_and_non_trade_payables: i64,
  pub payout_ratio: f64,
  pub price_to_book_value: f64,
  pub price_earnings: f64,
  pub price_to_earnings_ratio: f64,
  pub property_plant_equipment_net: i64,
  pub preferred_dividends_income_statement_impact: i64,
  pub share_price_adjusted_close: f64,
  pub price_sales: f64,
  pub price_to_sales_ratio: f64,
  pub trade_and_non_trade_receivables: i64,
  pub accumulated_retained_earnings_deficit: i64,
  pub revenues: i64,
  #[serde(rename(deserialize = "revenuesUSD"))]
  pub revenues_usd: i64,
  pub research_and_development_expense: i64,
  pub return_on_average_assets: f64,
  pub return_on_average_equity: f64,
  pub return_on_invested_capital: f64,
  pub return_on_sales: f64,
  pub share_based_compensation: i64,
  pub selling_general_and_administrative_expense: i64,
  pub share_factor: i64,
  pub shares: i64,
  pub weighted_average_shares: i64,
  pub weighted_average_shares_diluted: i64,
  pub sales_per_share: f64,
  pub tangible_asset_value: i64,
  pub tax_assets: i64,
  pub income_tax_expense: i64,
  pub tax_liabilities: i64,
  pub tangible_assets_book_value_per_share: f64,
  pub working_capital: i64
}

impl Default for Financial {
  fn default() -> Self {
    Financial {
      symbol: String::new(),
      period: String::new(),
      calendar_date: NaiveDate::from_ymd(1970, 1, 1),
      report_period: NaiveDate::from_ymd(1970, 1, 1),
      updated: NaiveDate::from_ymd(1970, 1, 1),
      date_key: NaiveDate::from_ymd(1970, 1, 1),
      accumulated_other_comprehensive_income: 0,
      assets: 0,
      assets_average: 0,
      assets_current: 0,
      asset_turnover: 0.0,
      assets_non_current: 0,
      book_value_per_share: 0.0,
      capital_expenditure: 0,
      cash_and_equivalents: 0,
      cash_and_equivalents_usd: 0,
      cost_of_revenue: 0,
      consolidated_income: 0,
      current_ratio: 0.0,
      debt_to_equity_ratio: 0.0,
      debt: 0,
      debt_current: 0,
      debt_non_current: 0,
      debt_usd: 0,
      deferred_revenue: 0,
      depreciation_amortization_and_accretion: 0,
      deposits: 0,
      dividend_yield: 0.0,
      dividends_per_basic_common_share: 0.0,
      earning_before_interest_taxes: 0,
      earnings_before_interest_taxes_depreciation_amortization: 0,
      ebitda_margin: 0.0,
      earnings_before_interest_taxes_depreciation_amortization_usd: 0,
      earning_before_interest_taxes_usd: 0,
      earnings_before_tax: 0,
      earnings_per_basic_share: 0.0,
      earnings_per_diluted_share: 0.0,
      earnings_per_basic_share_usd: 0.0,
      shareholders_equity: 0,
      average_equity: 0,
      shareholders_equity_usd: 0,
      enterprise_value: 0,
      enterprise_value_over_ebit: 0,
      enterprise_value_over_ebitda: 0.0,
      free_cash_flow: 0,
      free_cash_flow_per_share: 0.0,
      foreign_currency_usd_exchange_rate: 0,
      gross_profit: 0,
      gross_margin: 0.0,
      goodwill_and_intangible_assets: 0,
      interest_expense: 0,
      invested_capital: 0,
      invested_capital_average: 0,
      inventory: 0,
      investments: 0,
      investments_current: 0,
      investments_non_current: 0,
      total_liabilities: 0,
      current_liabilities: 0,
      liabilities_non_current: 0,
      market_capitalization: 0,
      net_cash_flow: 0,
      net_cash_flow_business_acquisitions_disposals: 0,
      issuance_equity_shares: 0,
      issuance_debt_securities: 0,
      payment_dividends_other_cash_distributions: 0,
      net_cash_flow_from_financing: 0,
      net_cash_flow_from_investing: 0,
      net_cash_flow_investment_acquisitions_disposals: 0,
      net_cash_flow_from_operations: 0,
      effect_of_exchange_rate_changes_on_cash: 0,
      net_income: 0,
      net_income_common_stock: 0,
      net_income_common_stock_usd: 0,
      net_loss_income_from_discontinued_operations: 0,
      net_income_to_non_controlling_interests: 0,
      profit_margin: 0.0,
      operating_expenses: 0,
      operating_income: 0,
      trade_and_non_trade_payables: 0,
      payout_ratio: 0.0,
      price_to_book_value: 0.0,
      price_earnings: 0.0,
      price_to_earnings_ratio: 0.0,
      property_plant_equipment_net: 0,
      preferred_dividends_income_statement_impact: 0,
      share_price_adjusted_close: 0.0,
      price_sales: 0.0,
      price_to_sales_ratio: 0.0,
      trade_and_non_trade_receivables: 0,
      accumulated_retained_earnings_deficit: 0,
      revenues: 0,
      revenues_usd: 0,
      research_and_development_expense: 0,
      return_on_average_assets: 0.0,
      return_on_average_equity: 0.0,
      return_on_invested_capital: 0.0,
      return_on_sales: 0.0,
      share_based_compensation: 0,
      selling_general_and_administrative_expense: 0,
      share_factor: 0,
      shares: 0,
      weighted_average_shares: 0,
      weighted_average_shares_diluted: 0,
      sales_per_share: 0.0,
      tangible_asset_value: 0,
      tax_assets: 0,
      income_tax_expense: 0,
      tax_liabilities: 0,
      tangible_assets_book_value_per_share: 0.0,
      working_capital: 0,
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FinancialsResponse {
  pub results: Vec<Financial>,
  // For debugging
  pub status: String,
}

impl Client {
  pub fn get_financials(
    &self,
    symbol: &str
  ) -> std::io::Result<FinancialsResponse> {
    let uri = format!(
      "{}/v2/reference/financials/{}?apikey={}",
      self.api_uri,
      symbol,
      self.key
    );

    let resp = get_response(&uri)?;
    let resp = resp.into_json_deserialize::<FinancialsResponse>()?;

    Ok(resp)
  }
}

#[cfg(test)]
mod financials {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let financials = client
      .get_financials("AAPL")
      .unwrap();
    assert!(financials.results.len() > 400);
  }
}

