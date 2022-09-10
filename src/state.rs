use std::collections::HashMap;

use crate::errors::{self, ClientError, TransactionError};
use crate::transaction::{self, Transaction, TransactionType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
/// The state of one client at any given time.
struct Client {
    /// The client's unique ID.
    id: u16,
    /// The available funds.
    available: Decimal,
    /// The held/disputed funds.
    held: Decimal,
    /// Flag indicating whether the account is locked
    locked: bool,
}

impl Client {
    /// Create a new client account given an ID.
    pub fn from_id(id: u16) -> Client {
        Client {
            id,
            available: Decimal::default(),
            held: Decimal::default(),
            locked: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Final client state, with fields the same as `Client`.
/// An additional field is provided for total, but
/// calculated on the fly.
/// Used for serialization.
struct CsvClient {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl From<&Client> for CsvClient {
    fn from(in_state: &Client) -> Self {
        CsvClient {
            client: in_state.id,
            available: in_state.available,
            held: in_state.held,
            total: in_state.available + in_state.held,
            locked: in_state.locked,
        }
    }
}

type Transactions = HashMap<u32, Transaction>;
type Disputes = HashMap<u32, Transaction>;
type ClientStates = HashMap<u16, Client>;

#[derive(Debug, Default)]
/// The overall state of the program at any given time.
pub struct CurrentState {
    /// A map from transaction IDs to deposits/withdrawals.
    transactions: Transactions,
    /// A list of active disputes.
    disputes: Disputes,
    /// The intermediate client states.
    client_states: ClientStates,
}

impl CurrentState {
    /// Performs various checks on deposits and withdrawals.
    fn check_regular(
        &mut self,
        tx: &Transaction,
    ) -> Result<&mut Client, crate::errors::Error> {
        if self.transactions.contains_key(&tx.id) {
            return Err(TransactionError::AlreadyExists(tx.id).into());
        }
        let client = self
            .client_states
            .entry(tx.client)
            .or_insert_with(|| Client::from_id(tx.client));
        if client.locked {
            return Err(ClientError::Locked(tx.id).into());
        }

        Ok(client)
    }

    /// Performs checks on dispute and dispute results.
    fn check_irregular(
        &mut self,
        tx: &Transaction,
    ) -> Result<(&mut Client, &Transaction), crate::errors::Error> {
        let rtx = self
            .transactions
            .get(&tx.id)
            .ok_or(TransactionError::NonexistentTransaction(tx.id))?;
        if tx.client != rtx.client {
            return Err(TransactionError::ClientMismatch(tx.id).into());
        }
        // If the transaction exists, the client is guaranteed to exist.
        let client = self.client_states.get_mut(&tx.client).unwrap();
        if client.locked {
            return Err(ClientError::Locked(tx.id).into());
        }

        if tx.r#type != transaction::TransactionType::Dispute {
            let dispute = self.disputes.remove(&tx.id);
            if dispute.is_none() {
                return Err(TransactionError::NoxexistentDispute(tx.id).into());
            }
        } else if self.disputes.contains_key(&tx.id) {
            return Err(TransactionError::DisputeAlreadyExists(tx.id).into());
        }

        Ok((client, rtx))
    }

    /// Processes one record, and updates the state.
    pub fn add(&mut self, tx: &Transaction) -> Result<(), crate::errors::Error> {
        match tx.r#type {
            TransactionType::Withdrawal => {
                let client = self.check_regular(tx)?;
                if tx.amount.unwrap() >= client.available {
                    return Err(ClientError::InsufficientFunds(tx.id).into());
                }
                client.available -= tx.amount.unwrap();
            }
            TransactionType::Deposit => {
                let client = self.check_regular(tx)?;
                client.available += tx.amount.unwrap();
            }
            TransactionType::Dispute => {
                let (client, rtx) = self.check_irregular(tx)?;
                client.held += rtx.amount.unwrap();
                client.available -= rtx.amount.unwrap();
                self.disputes.insert(tx.id, *tx);
            }
            TransactionType::Resolve => {
                let (client, rtx) = self.check_irregular(tx)?;
                client.held += rtx.amount.unwrap();
                client.available -= rtx.amount.unwrap();
            }
            TransactionType::Chargeback => {
                let (client, rtx) = self.check_irregular(tx)?;
                client.locked = true;
                client.held -= rtx.amount.unwrap();
            }
        }
        Ok(())
    }

    /// Processes everything from a CSV stream.
    pub fn process_from_csv(&mut self, reader: impl std::io::Read) -> Result<(), crate::errors::Error> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(reader);
        rdr.deserialize().try_for_each(|tx| {
            let tx = tx?;
            let result = self.add(&tx);
            if let Err(err) = result {
                eprintln!("Warning: {}", err);
            }
            Ok::<_, errors::Error>(())
        })?;
        Ok(())
    }

    /// Writes results into a CSV stream.
    pub fn into_csv(self, writer: impl std::io::Write) -> Result<(), csv::Error> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);
        self.client_states
            .values()
            .try_for_each(|item| wtr.serialize(CsvClient::from(item)))?;
        Ok(())
    }
}
