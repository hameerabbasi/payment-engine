use std::convert::TryFrom;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::errors;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
/// Defines the type of a transaction
pub enum TransactionType {
    Withdrawal,
    Deposit,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
/// An unchecked transaction type.
struct TransactionUnchecked {
    #[serde(rename = "type")]
    pub r#type: TransactionType,
    pub client: u16,
    #[serde(alias = "tx")]
    pub id: u32,
    pub amount: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(try_from = "TransactionUnchecked")]
/// A transaction type with fields internally validated.
pub struct Transaction {
    #[serde(rename = "type")]
    pub r#type: TransactionType,
    pub client: u16,
    #[serde(alias = "tx")]
    pub id: u32,
    pub amount: Option<Decimal>,
}

impl Transaction {
    /// Creates a `Transaction` from its unchecked variant,
    /// without running any checks.
    fn from_unchecked(tx: TransactionUnchecked) -> Self {
        Transaction {
            amount: tx.amount,
            client: tx.client,
            id: tx.id,
            r#type: tx.r#type,
        }
    }
}

impl TryFrom<TransactionUnchecked> for Transaction {
    type Error = errors::TransactionError;

    /// Performs all necessary checks on an `UncheckedTransaction` and then converts
    /// it to a `Transaction`.
    fn try_from(tx: TransactionUnchecked) -> Result<Self, Self::Error> {
        match tx.r#type {
            TransactionType::Deposit | TransactionType::Withdrawal => match tx.amount {
                Some(amount) => {
                    if amount <= Decimal::default() {
                        Err(errors::TransactionError::AmountNotPositive(tx.id))
                    } else {
                        Ok(Self::from_unchecked(tx))
                    }
                }
                None => Err(errors::TransactionError::MissingAmount(tx.id)),
            },
            TransactionType::Dispute | TransactionType::Resolve | TransactionType::Chargeback => {
                match tx.amount {
                    Some(_) => Err(errors::TransactionError::SuperfluousAmount(tx.id)),
                    None => Ok(Self::from_unchecked(tx)),
                }
            }
        }
    }
}
