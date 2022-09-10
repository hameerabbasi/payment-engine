use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("transation with ID `{0}` already exists")]
    AlreadyExists(u32),
    #[error("transation with ID `{0}` does not exist")]
    NonexistentTransaction(u32),
    #[error("transation with ID `{0}` had a negative or zero amount")]
    AmountNotPositive(u32),
    #[error("transation with ID `{0}` had a different client from the one specified")]
    ClientMismatch(u32),
    #[error("dispute for transaction ID `{0}` does not exist")]
    NoxexistentDispute(u32),
    #[error("dispute for transaction ID `{0}` already exists")]
    DisputeAlreadyExists(u32),
    #[error("missing amount for transaction ID `{0}`")]
    MissingAmount(u32),
    #[error("superfluous amount for transaction ID `{0}`")]
    SuperfluousAmount(u32),
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("client locked for transaction ID `{0}`")]
    Locked(u32),
    #[error("client for transaction ID `{0}` had insufficient funds")]
    InsufficientFunds(u32),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("transation error: {0}")]
    Transaction(#[from] TransactionError),
    #[error("client error: {0}")]
    Client(#[from] ClientError),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
