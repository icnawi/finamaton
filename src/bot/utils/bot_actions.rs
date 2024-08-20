/* Common utilites for handlers. */

use teloxide::{
    payloads::SendMessage,
    prelude::*,
    requests::JsonRequest,
    types::{Message, MessageId},
    Bot,
};

use crate::bot::{
    constants::currency::CURRENCY_DEFAULT,
    processor::{assert_rate_limit, get_chat_setting, retrieve_valid_currencies, ChatSetting},
};

use super::{BotError, StatementOption};

// Checks and asserts the rate limit of 1 request per user per second.
// Returns true if okay, false if exceeded
pub fn assert_handle_request_limit(msg: Message) -> bool {
    if let Some(user) = msg.from() {
        let user_id = user.id.to_string();
        let timestamp = msg.date.timestamp();
        let request_status = assert_rate_limit(&user_id, timestamp);
        if let Err(_) = request_status {
            log::error!(
                "Rate limit exceeded for user: {} in chat: {}, with message timestamp: {}",
                user_id,
                msg.chat.id,
                timestamp
            );
            return false;
        }
    }

    true
}

// Wrapper function to send bot message to specific thread, if available
// Only replaces bot::send_message, as bot::edit_message_text edits specific msg ID
pub fn send_bot_message(bot: &Bot, msg: &Message, text: String) -> JsonRequest<SendMessage> {
    let thread_id = msg.thread_id;
    match thread_id {
        Some(thread_id) => bot
            .send_message(msg.chat.id, text)
            .message_thread_id(thread_id),
        None => bot.send_message(msg.chat.id, text),
    }
}

// Removes all old messages, given a chat and a list of message IDs
pub async fn delete_bot_messages(
    bot: &Bot,
    chat_id: &str,
    messages: Vec<MessageId>,
) -> Result<(), BotError> {
    for message in messages {
        bot.delete_message(chat_id.to_string(), message).await?;
    }
    Ok(())
}

// Checks if Erase Messages setting is enabled
pub fn is_erase_messages(chat_id: &str) -> bool {
    let erase = get_chat_setting(chat_id, ChatSetting::EraseMessages(None));
    if let Ok(ChatSetting::EraseMessages(Some(true))) = erase {
        true
    } else {
        false
    }
}

// Processes and retrieves appropriate valid currencies for balances and spendings.
pub fn process_valid_currencies(
    chat_id: &str,
    sender_id: &str,
    option: StatementOption,
    default_currency: String,
) -> Vec<String> {
    let mut valid_currencies = match retrieve_valid_currencies(&chat_id) {
        Ok(currencies) => currencies,
        Err(_) => {
            log::error!(
                "View Spendings - User {} failed to retrieve valid currencies for group {}",
                sender_id,
                chat_id
            );
            vec![]
        }
    };

    valid_currencies.retain(|x| x != CURRENCY_DEFAULT.0 && x != &default_currency);

    if let StatementOption::Currency(ref curr) = option {
        valid_currencies.retain(|x| x != curr);
    }

    // Add back default currency button if not NIL, and currently not default
    if default_currency != CURRENCY_DEFAULT.0 {
        if let StatementOption::Currency(ref curr) = option {
            if curr != &default_currency {
                valid_currencies.push(default_currency.clone());
            }
        } else if valid_currencies.len() > 0 {
            // Adds back default currency on convert, only if there are also other
            // currencies. Else, the converted is already equal to the default.
            valid_currencies.push(default_currency.clone());
        }
    }

    // Special buttons
    let conversion_button = format!("Convert To {default_currency}");
    // Add conversion button only if not currently on convert, and have default currency
    if option != StatementOption::ConvertCurrency
        && default_currency != CURRENCY_DEFAULT.0
        && valid_currencies.len() > 0
    {
        valid_currencies.push(conversion_button);
        // Add no currency button if no default currency, and not currently NIL
    } else if default_currency == CURRENCY_DEFAULT.0 {
        if let StatementOption::Currency(ref curr) = option {
            if curr != CURRENCY_DEFAULT.0 {
                valid_currencies.push("No Currency".to_string());
            }
        }
    }

    valid_currencies
}
