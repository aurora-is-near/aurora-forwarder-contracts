use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;

// We multiply percents to 100 here to get rid of the floating numbers.
const MIN_FEE_PERCENT: u64 = 1; // 0.01 %
const MAX_FEE_PERCENT: u64 = 1000; // 10 %
const DEFAULT_PERCENT: U64 = U64(500); // 5%

#[near_bindgen]
#[derive(Debug, BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FeesCalculator {
    percent: U64,
    owner: AccountId,
}

#[near_bindgen]
impl FeesCalculator {
    /// Contract's constructor.
    ///
    /// # Panics
    ///
    /// The constructor panics if the state already exists.
    #[init]
    #[must_use]
    pub fn new() -> Self {
        assert!(!env::state_exists());

        Self {
            percent: DEFAULT_PERCENT,
            owner: env::predecessor_account_id(),
        }
    }

    /// Calculates and returns the fee for the corresponding token and Aurora Network.
    #[must_use]
    pub fn calculate_fees(
        &self,
        amount: U128,
        token_id: &AccountId,
        target_network: &AccountId,
        target_address: String,
    ) -> U128 {
        let _ = (token_id, target_network, target_address);

        u128::from(self.percent.0)
            .checked_mul(amount.0)
            .unwrap_or_default()
            .saturating_div(10000)
            .into()
    }

    /// Set the percent of the fee.
    ///
    /// # Panics
    ///
    /// Panics if the invoker of the transaction is not owner.
    #[allow(clippy::needless_pass_by_value)]
    pub fn set_fee_percent(&mut self, percent: String) {
        assert_eq!(env::predecessor_account_id(), self.owner);

        match parse_percent(&percent) {
            Ok(value) => self.percent = value,
            Err(e) => env::panic_str(&format!("Couldn't parse percent: {e}")),
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_fee_percent(&self) -> String {
        format!("{:.2}", self.percent.0 as f64 / 100.0)
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn parse_percent(percent: &str) -> Result<U64, ParseError> {
    validate_decimal_part(percent)?;

    let result = f64::from_str(percent)
        .map(|p| (p * 100.0) as u64) // as conversion is safe here because we validate the number of decimals
        .map_err(ParseError::ParseFloat)?;

    if result < MIN_FEE_PERCENT {
        Err(ParseError::TooLowPercent)
    } else if result > MAX_FEE_PERCENT {
        Err(ParseError::TooHighPercent)
    } else {
        Ok(U64(result))
    }
}

#[derive(Debug)]
enum ParseError {
    ParseFloat(ParseFloatError),
    TooLowPercent,
    TooHighPercent,
    TooManyDecimals,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[allow(deprecated)]
        let msg = match self {
            Self::ParseFloat(error) => error.description(),
            Self::TooLowPercent => "provided percent is less than 0.01%",
            Self::TooHighPercent => "provided percent is more than 10%",
            Self::TooManyDecimals => "provided percent could contain only 2 decimals",
        };

        f.write_str(msg)
    }
}

fn validate_decimal_part(percent: &str) -> Result<(), ParseError> {
    match percent.split_once('.') {
        Some((_, decimal)) if decimal.len() > 2 => Err(ParseError::TooManyDecimals),
        _ => Ok(()), // no decimals or the number of decimals is less or equal 2.
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_percent, FeesCalculator, ParseError};

    #[test]
    fn test_parse_percent() {
        assert_eq!(parse_percent("10").unwrap(), 1000.into());
        assert_eq!(parse_percent("2").unwrap(), 200.into());
        assert_eq!(parse_percent("0.25").unwrap(), 25.into());
        assert_eq!(parse_percent("0.01").unwrap(), 1.into());
        assert!(matches!(
            parse_percent("0.015").err(),
            Some(ParseError::TooManyDecimals)
        ));
        assert!(matches!(
            parse_percent("0.009").err(),
            Some(ParseError::TooManyDecimals)
        ));
        assert!(matches!(
            parse_percent("10.1").err(),
            Some(ParseError::TooHighPercent)
        ));
        assert!(matches!(
            parse_percent("hello").err(),
            Some(ParseError::ParseFloat(_))
        ));
    }

    #[test]
    fn test_set_percent() {
        let mut contract = FeesCalculator::new();

        assert_eq!(contract.get_fee_percent(), "5.00");
        contract.set_fee_percent("6".to_string());
        assert_eq!(contract.get_fee_percent(), "6.00");
        contract.set_fee_percent("7.5".to_string());
        assert_eq!(contract.get_fee_percent(), "7.50");
    }

    #[test]
    #[should_panic(
        expected = "Couldn't parse percent: provided percent could contain only 2 decimals"
    )]
    fn test_set_percent_with_many_decimals() {
        let mut contract = FeesCalculator::new();
        contract.set_fee_percent("6.123".to_string());
    }

    #[test]
    #[should_panic(expected = "Couldn't parse percent: provided percent is more than 10%")]
    fn test_set_too_high_percents() {
        let mut contract = FeesCalculator::new();
        contract.set_fee_percent("12.12".to_string());
    }
}
