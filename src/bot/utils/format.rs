use chrono_tz::Tz;
use regex::Regex;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::{
    constants::currency::CURRENCY_DEFAULT,
    currency::{get_currency_from_code, get_default_currency, Currency},
    handlers::Payment,
    processor::{get_chat_setting, ChatSetting},
    redis::Debt,
    utils::time::reformat_datetime,
};

use super::BotError;

// Retrieves the currency given a currency code.
pub fn get_currency(code: &str) -> Result<Currency, BotError> {
    let currency = get_currency_from_code(code);
    match currency {
        Some(currency) => Ok(currency),
        None => Err(BotError::UserError(
            "Sorry, unknown currency...".to_string(),
        )),
    }
}

// Retrieves the default currency of a chat. Does not return an error, assumes default.
pub fn get_chat_default_currency(chat_id: &str) -> Currency {
    let setting = ChatSetting::DefaultCurrency(None);
    let currency = get_chat_setting(&chat_id, setting);
    match currency {
        Ok(ChatSetting::DefaultCurrency(Some(currency))) => {
            let currency = get_currency(&currency);
            if let Ok(currency) = currency {
                return currency;
            }
        }
        // Skips error, assumes default
        _ => {}
    }
    get_default_currency()
}

// Converts an amount from base value to actual representation in currency.
pub fn display_amount(amount: i64, decimal_places: i32) -> String {
    if decimal_places == 0 {
        return amount.to_string();
    } else if amount == 0 {
        return "0".to_string();
    }

    // Amount is not 0, and decimal places are not 0
    let factor = 10.0_f64.powi(decimal_places);
    let actual_amount = amount as f64 / factor;
    format!(
        "{:.decimals$}",
        actual_amount,
        decimals = decimal_places as usize
    )
}

// Displays an amount together with its currency
pub fn display_currency_amount(amount: i64, currency: Currency) -> String {
    if currency.0 == CURRENCY_DEFAULT.0 {
        format!("{}", display_amount(amount, currency.1))
    } else {
        format!("{} {}", display_amount(amount, currency.1), currency.0)
    }
}

// Gets the currency to be used when provided with the chosen currency, and the chat ID.
pub fn use_currency(currency: Currency, chat_id: &str) -> Currency {
    let default_currency = get_chat_default_currency(chat_id);
    if currency.0 == CURRENCY_DEFAULT.0 {
        default_currency
    } else {
        currency
    }
}

// Displays the header for the balances, depending on the statement option applied.
pub fn display_balance_header(chat_id: &str, currency: &str) -> String {
    let conversion = match get_chat_setting(chat_id, ChatSetting::CurrencyConversion(None)) {
        Ok(ChatSetting::CurrencyConversion(Some(value))) => value,
        _ => false,
    };
    let default_currency = match get_chat_setting(chat_id, ChatSetting::DefaultCurrency(None)) {
        Ok(ChatSetting::DefaultCurrency(Some(currency))) => currency,
        _ => CURRENCY_DEFAULT.0.to_string(),
    };

    if conversion {
        format!(
            "Updated balances, all converted to {}!\n\n",
            default_currency
        )
    } else if currency == CURRENCY_DEFAULT.0 {
        if default_currency != CURRENCY_DEFAULT.0 {
            format!("Updated balances in {}!\n\n", default_currency)
        } else {
            format!("Updated balances!\n\n")
        }
    } else {
        format!("Updated balances in {}!\n\n", currency)
    }
}

// Displays balances in a more readable format. Now only shows in one currency.
pub fn display_balances(debts: &Vec<Debt>) -> String {
    let mut message = String::new();
    for debt in debts {
        let currency = get_currency(&debt.currency);
        match currency {
            Ok(currency) => {
                message.push_str(&format!(
                    "{} owes {}: {}\n",
                    display_username(&debt.debtor),
                    display_username(&debt.creditor),
                    display_amount(debt.amount, currency.1),
                ));
            }
            // Should not occur, since code is already processed and stored in database
            Err(_err) => {
                continue;
            }
        }
    }

    if debts.is_empty() {
        "No outstanding balances! 🥳\n".to_string()
    } else {
        message
    }
}

// Displays debts in a more readable format.
pub fn display_debts(debts: &Vec<(String, i64)>, decimal_places: i32) -> String {
    let mut message = String::new();
    for debt in debts {
        message.push_str(&format!(
            "    {}: {}\n",
            display_username(&debt.0),
            display_amount(debt.1, decimal_places),
        ));
    }
    message
}

// Displays a single payment entry in a user-friendly format.
pub fn display_payment(payment: &Payment, serial_num: usize, time_zone: Tz) -> String {
    let actual_currency = use_currency(payment.currency.clone(), &payment.chat_id);

    format!(
        "__________________________\n{}. {}\nDate: {}\nPayer: {}\nTotal: {}\nSplit:\n{}",
        serial_num,
        payment.description,
        reformat_datetime(&payment.datetime, time_zone),
        display_username(&payment.creditor),
        display_currency_amount(payment.total, actual_currency.clone()),
        display_debts(&payment.debts, actual_currency.1)
    )
}

// Make a keyboard, button menu.
pub fn make_keyboard(options: Vec<&str>, columns: Option<usize>) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    if let Some(col) = columns {
        for chunk in options.chunks(col) {
            let mut row: Vec<InlineKeyboardButton> = Vec::new();
            for option in chunk {
                row.push(InlineKeyboardButton::callback(
                    option.to_string(),
                    option.to_string(),
                ));
            }
            keyboard.push(row);
        }
    } else {
        for option in options {
            keyboard.push(vec![InlineKeyboardButton::callback(option, option)]);
        }
    }

    InlineKeyboardMarkup::new(keyboard)
}

// Make debt selection keyboard
pub fn make_keyboard_debt_selection() -> InlineKeyboardMarkup {
    let buttons = vec!["Equal", "Exact", "Proportion"];
    make_keyboard(buttons, Some(1))
}

// Displays a username with the '@' symbol.
pub fn display_username(username: &str) -> String {
    format!("@{}", username)
}

// Ensures that a username has a leading '@'.
pub fn parse_username(username: &str) -> Result<String, BotError> {
    let text: &str;
    if username.starts_with('@') {
        text = username.trim_start_matches('@');
    } else {
        text = username;
    }

    if text.split_whitespace().count() == 1 && text.len() >= 5 {
        let re = Regex::new(r"^[a-zA-Z0-9_]+$");
        if let Ok(re) = re {
            if re.captures(text).is_some() {
                return Ok(text.to_string());
            }
        }
    }

    Err(BotError::UserError(
        "Uh-oh! ❌ Please give me a valid username!".to_string(),
    ))
}
