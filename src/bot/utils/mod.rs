use teloxide::{
    dispatching::dialogue::{Dialogue, InMemStorage, InMemStorageError},
    RequestError,
};

use crate::bot::{processor::ProcessError, State};

/* Types */
pub type UserDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), BotError>;

#[derive(PartialEq, Debug, Clone)]
pub enum StatementOption {
    Currency(String),
    ConvertCurrency,
}

#[derive(Debug, Clone)]
pub enum SelectPaymentType {
    EditPayment,
    DeletePayment,
}

#[derive(thiserror::Error, Debug)]
pub enum BotError {
    #[error("{0}")]
    UserError(String),
    #[error("Process error: {0}")]
    ProcessError(ProcessError),
    #[error("Request error: {0}")]
    RequestError(RequestError),
}

impl From<RequestError> for BotError {
    fn from(request_error: RequestError) -> BotError {
        BotError::RequestError(request_error)
    }
}

impl From<InMemStorageError> for BotError {
    fn from(storage_error: InMemStorageError) -> BotError {
        BotError::UserError(storage_error.to_string())
    }
}

impl From<ProcessError> for BotError {
    fn from(process_error: ProcessError) -> BotError {
        BotError::ProcessError(process_error)
    }
}

pub mod amounts;
pub mod bot_actions;
pub mod format;
pub mod time;
