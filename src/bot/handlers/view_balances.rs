use teloxide::{
    prelude::*,
    types::{Message, MessageId},
};

use crate::bot::{
    constants::{
        currency::CURRENCY_DEFAULT,
        messages::{STATEMENT_INSTRUCTIONS_MESSAGE, UNKNOWN_ERROR_MESSAGE},
    },
    processor::{get_chat_setting, retrieve_debts, ChatSetting},
    utils::{
        bot_actions::{assert_handle_request_limit, process_valid_currencies, send_bot_message},
        format::{display_balances, make_keyboard},
        HandlerResult, StatementOption, UserDialogue,
    },
    State,
};

/* Utilities */

async fn handle_balances_with_option(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    sender_id: String,
    mut option: StatementOption,
    id: Option<MessageId>,
) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    let balances_data = retrieve_debts(&chat_id, option.clone()).await;

    match balances_data {
        Ok(mut balances_data) => {
            let default_currency =
                match get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None)) {
                    Ok(ChatSetting::DefaultCurrency(Some(currency))) => currency,
                    _ => CURRENCY_DEFAULT.0.to_string(),
                };

            let mut valid_currencies = process_valid_currencies(
                &chat_id,
                &sender_id,
                option.clone(),
                default_currency.clone(),
            );

            // If no default currency, NIL has no balances, but other currencies do
            if balances_data.len() == 0 && valid_currencies.len() > 0 {
                let currency = valid_currencies.first().unwrap().clone();
                option = StatementOption::Currency(currency.clone());
                balances_data = match retrieve_debts(&chat_id, option.clone()).await {
                    Ok(new_data) => {
                        valid_currencies.retain(|curr| curr != &currency);
                        new_data
                    }
                    Err(_err) => balances_data,
                };
            }

            let ref_valid_currencies = valid_currencies
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<&str>>();

            let has_buttons = valid_currencies.len() > 0;
            let keyboard = make_keyboard(ref_valid_currencies, Some(2));

            let header = if let StatementOption::Currency(curr) = option {
                if curr == CURRENCY_DEFAULT.0 {
                    format!("📊 Current balances!")
                } else {
                    format!("📊 Current {curr} balances!")
                }
            } else if has_buttons {
                format!("📊 Current balances, converted to {default_currency}!")
            } else {
                format!("📊 Current balances!")
            };

            match id {
                Some(id) => {
                    bot.edit_message_text(
                        chat_id.clone(),
                        id,
                        format!(
                            "{}\n\n{}\n{}",
                            header,
                            display_balances(&balances_data),
                            if has_buttons {
                                STATEMENT_INSTRUCTIONS_MESSAGE
                            } else {
                                ""
                            }
                        ),
                    )
                    .reply_markup(keyboard)
                    .await?;
                }
                None => {
                    send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "{}\n\n{}\n{}",
                            header,
                            display_balances(&balances_data),
                            if has_buttons {
                                STATEMENT_INSTRUCTIONS_MESSAGE
                            } else {
                                ""
                            }
                        ),
                    )
                    .reply_markup(keyboard)
                    .await?;
                }
            }
            dialogue.update(State::BalancesMenu).await?;

            log::info!(
                "View Balances - User {} viewed balances for group {}: {}",
                sender_id,
                chat_id,
                display_balances(&balances_data)
            );
        }
        Err(err) => {
            match id {
                Some(id) => {
                    bot.edit_message_text(chat_id.clone(), id, UNKNOWN_ERROR_MESSAGE)
                        .await?;
                }
                None => {
                    send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;
                }
            }
            log::error!(
                "View Balances - User {} failed to view balances for group {}: {}",
                sender_id,
                chat_id,
                err.to_string()
            );
        }
    }

    Ok(())
}

/* View the balances for the group.
*/
pub async fn action_view_balances(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let chat_id = msg.chat.id.to_string();
    let sender_id = msg.from().as_ref().unwrap().id.to_string();
    let is_convert = match get_chat_setting(&chat_id, ChatSetting::CurrencyConversion(None)) {
        Ok(ChatSetting::CurrencyConversion(Some(value))) => value,
        _ => false,
    };
    let default_currency = match get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None)) {
        Ok(ChatSetting::DefaultCurrency(Some(currency))) => currency,
        _ => "NIL".to_string(),
    };

    let option = if is_convert {
        StatementOption::ConvertCurrency
    } else {
        StatementOption::Currency(default_currency.clone())
    };

    handle_balances_with_option(bot, dialogue, msg, sender_id, option, None).await?;

    Ok(())
}

/* Views the balances for the group.
 * Takes in a callback query representing the user option on format to display.
 */
pub async fn action_balances_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        let sender_id = query.from.id.to_string();

        if let Some(msg) = query.message {
            let id = msg.id;
            let chat_id = msg.chat.id.to_string();
            match button.as_str() {
                _ if button.as_str().starts_with("Convert To ") => {
                    let option = StatementOption::ConvertCurrency;
                    handle_balances_with_option(bot, dialogue, msg, sender_id, option, Some(id))
                        .await?;
                }
                _ if button.as_str() == "No Currency" => {
                    let option = StatementOption::Currency(CURRENCY_DEFAULT.0.to_string());
                    handle_balances_with_option(bot, dialogue, msg, sender_id, option, Some(id))
                        .await?;
                }
                _ if button.as_str().len() == 3 => {
                    let option = StatementOption::Currency(button.as_str().to_string());
                    handle_balances_with_option(bot, dialogue, msg, sender_id, option, Some(id))
                        .await?;
                }
                _ => {
                    log::error!(
                        "View Balances Menu - Invalid button in chat {} by user {}: {}",
                        chat_id,
                        sender_id,
                        button
                    );
                }
            }
        }
    }

    Ok(())
}
