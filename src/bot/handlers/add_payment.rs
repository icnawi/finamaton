use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{Message, MessageId},
};

use crate::bot::{
    constants::{
        commands::COMMAND_CANCEL,
        messages::{
            CANCEL_ADD_MESSAGE, DEBT_EQUAL_DESCRIPTION_MESSAGE, DEBT_EQUAL_INSTRUCTIONS_MESSAGE,
            DEBT_EXACT_DESCRIPTION_MESSAGE, DEBT_EXACT_INSTRUCTIONS_MESSAGE,
            DEBT_RATIO_DESCRIPTION_MESSAGE, DEBT_RATIO_INSTRUCTIONS_MESSAGE, NO_TEXT_MESSAGE,
            TOTAL_INSTRUCTIONS_MESSAGE, UNKNOWN_ERROR_MESSAGE,
        },
    },
    currency::Currency,
    dispatcher::State,
    processor::add_payment,
    utils::{
        amounts::{parse_currency_amount, process_debts},
        bot_actions::{
            assert_handle_request_limit, delete_bot_messages, is_erase_messages, send_bot_message,
        },
        format::{
            display_balance_header, display_balances, display_currency_amount, display_debts,
            display_username, make_keyboard, make_keyboard_debt_selection, parse_username,
            use_currency,
        },
        HandlerResult, UserDialogue,
    },
};

// use super::utils::{
//     assert_handle_request_limit, delete_bot_messages, is_erase_messages, send_bot_message,
// };

/* Utilities */
#[derive(Clone, Debug)]
pub struct AddPaymentParams {
    chat_id: String,
    sender_id: String,
    sender_username: String,
    datetime: String,
    description: Option<String>,
    creditor: Option<String>,
    currency: Option<Currency>,
    total: Option<i64>,
    debts: Option<Vec<(String, i64)>>,
}

#[derive(Clone, Debug)]
pub enum AddPaymentEdit {
    Description,
    Creditor,
    Total,
    DebtsEqual,
    DebtsExact,
    DebtsRatio,
}

#[derive(Clone, Debug)]
pub enum AddDebtsFormat {
    Equal,
    Exact,
    Ratio,
}

// Controls the state for misc handler actions that return to same state.
async fn repeat_state(
    dialogue: UserDialogue,
    state: State,
    new_message: MessageId,
) -> HandlerResult {
    match state {
        State::AddDescription { mut messages } => {
            messages.push(new_message);
            dialogue.update(State::AddDescription { messages }).await?;
        }
        State::AddCreditor {
            mut messages,
            payment,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::AddCreditor { messages, payment })
                .await?;
        }
        State::AddTotal {
            mut messages,
            payment,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::AddTotal { messages, payment })
                .await?;
        }
        State::AddDebtSelection {
            mut messages,
            payment,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::AddDebtSelection { messages, payment })
                .await?;
        }
        State::AddDebt {
            mut messages,
            payment,
            debts_format,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::AddDebt {
                    messages,
                    payment,
                    debts_format,
                })
                .await?;
        }
        State::AddConfirm {
            mut messages,
            payment,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::AddConfirm { messages, payment })
                .await?;
        }
        State::AddEditMenu {
            mut messages,
            payment,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::AddEditMenu { messages, payment })
                .await?;
        }
        State::AddEdit {
            mut messages,
            payment,
            edit,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::AddEdit {
                    messages,
                    payment,
                    edit,
                })
                .await?;
        }
        State::AddEditDebtsMenu {
            mut messages,
            payment,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::AddEditDebtsMenu { messages, payment })
                .await?;
        }
        _ => (),
    }
    Ok(())
}

// Controls the dialogue for ending an add payment operation.
async fn complete_add_payment(
    bot: &Bot,
    dialogue: UserDialogue,
    chat_id: &str,
    messages: Vec<MessageId>,
) -> HandlerResult {
    if is_erase_messages(chat_id) {
        delete_bot_messages(&bot, chat_id, messages).await?;
    }
    dialogue.exit().await?;
    Ok(())
}

/* Displays a payment entry (being added) in String format.
*/
fn display_add_payment(payment: &AddPaymentParams) -> String {
    let description = match &payment.description {
        Some(desc) => format!("Description: {}\n", desc),
        None => "".to_string(),
    };
    let creditor = match &payment.creditor {
        Some(cred) => format!("Payer: {}\n", display_username(cred)),
        None => "".to_string(),
    };
    let total = match &payment.total {
        Some(total) => match &payment.currency {
            Some(currency) => format!(
                "Total: {}\n",
                display_currency_amount(*total, use_currency(currency.clone(), &payment.chat_id))
            ),
            None => "".to_string(),
        },
        None => "".to_string(),
    };
    let debts = match &payment.debts {
        Some(debts) => match &payment.currency {
            Some(currency) => format!("Split:\n{}", display_debts(&debts, currency.1)),
            None => "".to_string(),
        },
        None => "".to_string(),
    };

    format!("{}{}{}{}\n", description, creditor, total, debts)
}

/* Add a payment entry in a group chat.
 * Displays an overview of the current details provided.
 * Is not a normal endpoint function, just a temporary transition function.
 */
async fn display_add_overview(
    bot: &Bot,
    dialogue: &UserDialogue,
    msg: &Message,
    mut messages: Vec<MessageId>,
    payment: AddPaymentParams,
) -> HandlerResult {
    let buttons = vec!["Cancel", "Edit", "Confirm"];
    let keyboard = make_keyboard(buttons, Some(2));

    let new_message = send_bot_message(
        &bot,
        &msg,
        format!(
            "Here you go! 📝\n\n{}Do you submit this entry or do you want to make any changes?",
            display_add_payment(&payment)
        ),
    )
    .reply_markup(keyboard)
    .await?
    .id;
    messages.push(new_message);
    dialogue
        .update(State::AddConfirm { messages, payment })
        .await?;
    Ok(())
}

/* Add a payment entry in a group chat.
 * Displays a button menu for user to choose which part of the payment details to edit.
 */
async fn display_add_edit_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    messages: Vec<MessageId>,
    payment: AddPaymentParams,
) -> HandlerResult {
    let buttons = vec!["Description", "Payer", "Total", "Split", "Back"];
    let keyboard = make_keyboard(buttons, Some(2));

    if let Some(Message { id, chat, .. }) = query.message {
        bot.edit_message_text(
            chat.id,
            id,
            format!(
                "{}Sure! What expense do you wish to edit?",
                display_add_payment(&payment)
            ),
        )
        .reply_markup(keyboard)
        .await?;
        dialogue
            .update(State::AddEditMenu { messages, payment })
            .await?;
    }
    Ok(())
}

/* Parses a string representing debts, and handles it accordingly
*/
async fn handle_debts(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
    messages: Vec<MessageId>,
    payment: AddPaymentParams,
    debts_format: AddDebtsFormat,
) -> HandlerResult {
    let error_msg = match debts_format {
        AddDebtsFormat::Equal => DEBT_EQUAL_INSTRUCTIONS_MESSAGE,
        AddDebtsFormat::Exact => DEBT_EXACT_INSTRUCTIONS_MESSAGE,
        AddDebtsFormat::Ratio => DEBT_RATIO_INSTRUCTIONS_MESSAGE,
    };
    match msg.text() {
        Some(text) => {
            let debts = process_debts(
                debts_format,
                text,
                &payment.creditor,
                payment.currency.clone(),
                payment.total,
            );
            if let Err(err) = debts {
                let new_message =
                    send_bot_message(&bot, &msg, format!("{}\n\n{error_msg}", err.to_string()))
                        .await?
                        .id;
                repeat_state(dialogue, state, new_message).await?;
                return Ok(());
            }

            let new_payment = AddPaymentParams {
                chat_id: payment.chat_id,
                sender_id: payment.sender_id,
                sender_username: payment.sender_username,
                datetime: payment.datetime,
                description: payment.description,
                creditor: payment.creditor,
                currency: payment.currency,
                total: payment.total,
                debts: Some(debts?),
            };

            display_add_overview(&bot, &dialogue, &msg, messages, new_payment).await?;
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, error_msg.to_string())
                .await?
                .id;
            repeat_state(dialogue, state, new_message).await?;
        }
    }
    Ok(())
}

/* Calls processor to execute the adding of the payment entry.
*/
async fn call_processor_add_payment(
    bot: Bot,
    dialogue: UserDialogue,
    messages: Vec<MessageId>,
    payment: AddPaymentParams,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(msg) = query.message {
        let chat_id = msg.chat.id;
        let payment_clone = payment.clone();
        let description = match payment.description {
            Some(desc) => desc,
            None => {
                log::error!(
                    "Add Payment Submission - Description not found for payment: {:?}",
                    payment_clone
                );
                send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;
                complete_add_payment(&bot, dialogue, &chat_id.to_string(), messages).await?;
                return Ok(());
            }
        };
        let creditor = match payment.creditor {
            Some(cred) => cred,
            None => {
                log::error!(
                    "Add Payment Submission - Creditor not found for payment: {:?}",
                    payment_clone
                );
                send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;
                complete_add_payment(&bot, dialogue, &chat_id.to_string(), messages).await?;
                return Ok(());
            }
        };
        let currency = match payment.currency {
            Some(curr) => curr,
            None => {
                log::error!(
                    "Add Payment Submission - Currency not found for payment: {:?}",
                    payment_clone
                );
                send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;
                complete_add_payment(&bot, dialogue, &chat_id.to_string(), messages).await?;
                return Ok(());
            }
        };
        let total = match payment.total {
            Some(tot) => tot,
            None => {
                log::error!(
                    "Add Payment Submission - Total not found for payment: {:?}",
                    payment_clone
                );
                send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;
                complete_add_payment(&bot, dialogue, &chat_id.to_string(), messages).await?;
                return Ok(());
            }
        };
        let debts = match payment.debts {
            Some(debts) => debts,
            None => {
                log::error!(
                    "Add Payment Submission - Debts not found for payment: {:?}",
                    payment_clone
                );
                send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;
                complete_add_payment(&bot, dialogue, &chat_id.to_string(), messages).await?;
                return Ok(());
            }
        };
        let payment_overview = display_add_payment(&payment_clone);
        let updated_balances = add_payment(
            payment.chat_id.clone(),
            payment.sender_username,
            payment.sender_id,
            payment.datetime,
            &description,
            &creditor,
            &currency.0,
            total,
            debts,
        )
        .await;
        match updated_balances {
            Ok(balances) => {
                send_bot_message(
                    &bot,
                    &msg,
                    format!("Payment successfully added!\n\n{}", payment_overview,),
                )
                .await?;
                send_bot_message(
                    &bot,
                    &msg,
                    format!(
                        "{}{}",
                        display_balance_header(&payment.chat_id, &currency.0),
                        display_balances(&balances)
                    ),
                )
                .await?;

                // Logging
                log::info!(
                    "Add Payment Submission - Processor updated balances successfully for user {} in chat {}: {:?}",
                    payment_clone.sender_id,
                    payment_clone.chat_id,
                    payment_clone
                    );
            }
            Err(err) => {
                send_bot_message(
                    &bot,
                    &msg,
                    format!(
                        "🤷 Oops! Something went wrong! I can't add the payment right now. Please try again later!\n\n"
                    ),
                )
                .await?;

                // Logging
                log::error!(
                    "Add Payment Submission - Processor failed to update balances for user {} in chat {} with payment {:?}: {}",
                    payment_clone.sender_id,
                    payment_clone.chat_id,
                    payment_clone,
                    err.to_string()
                    );
            }
        }
        complete_add_payment(&bot, dialogue, &chat_id.to_string(), messages).await?;
    }
    Ok(())
}

/* Action handler functions */

/* Handles a repeated call to add payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_add_payment(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let new_message = send_bot_message(&bot,
        &msg,
        format!("Oops! Probably you forgot to add payment! Please finish or {COMMAND_CANCEL} this before starting another one with me."),
        ).await?.id;

    repeat_state(dialogue, state, new_message).await?;
    Ok(())
}

/* Cancels the add payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_add_payment(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    send_bot_message(&bot, &msg, CANCEL_ADD_MESSAGE.to_string()).await?;

    match state {
        State::AddDescription { messages }
        | State::AddCreditor { messages, .. }
        | State::AddTotal { messages, .. }
        | State::AddDebtSelection { messages, .. }
        | State::AddDebt { messages, .. }
        | State::AddConfirm { messages, .. }
        | State::AddEditMenu { messages, .. }
        | State::AddEdit { messages, .. }
        | State::AddEditDebtsMenu { messages, .. } => {
            complete_add_payment(&bot, dialogue, &msg.chat.id.to_string(), messages).await?;
        }
        _ => (),
    }
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_add_payment(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let new_message = send_bot_message(
        &bot,
        &msg,
        format!("Oops! Probably you forgot to add payment! Please finish or {COMMAND_CANCEL} this before starting something new with me."),
        ).await?.id;

    repeat_state(dialogue, state, new_message).await?;
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot will ask for user to send messages to fill in required information,
 * before presenting the compiled information for confirmation with a menu.
 */
pub async fn action_add_payment(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let new_message = send_bot_message(
        &bot,
        &msg,
        format!("Absolutely, let's get started! \n\nWhat's the description for this new expense?"),
    )
    .await?
    .id;

    dialogue
        .update(State::AddDescription {
            messages: vec![new_message],
        })
        .await?;
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a description string from user, and proceeds to ask for creditor.
 */
pub async fn action_add_description(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
    mut messages: Vec<MessageId>,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let user = msg.from();
            if let Some(user) = user {
                if let Some(username) = &user.username {
                    let username = parse_username(username);

                    if let Err(err) = &username {
                        let new_message =
                            send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string())
                                .await?
                                .id;
                        repeat_state(dialogue.clone(), state, new_message).await?;

                        // Logging
                        log::error!(
                            "Add Payment Description - Failed to parse username for user {}: {}",
                            user.id,
                            err.to_string()
                        );
                    }

                    let payment = AddPaymentParams {
                        chat_id: msg.chat.id.to_string(),
                        sender_id: user.id.to_string(),
                        sender_username: username?,
                        datetime: msg.date.to_string(),
                        description: Some(text.to_string()),
                        creditor: None,
                        currency: None,
                        total: None,
                        debts: None,
                    };
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "{}Great! What's the Telegram username of the one who paid?",
                            display_add_payment(&payment)
                        ),
                    )
                    .await?
                    .id;
                    messages.push(new_message);
                    dialogue
                        .update(State::AddCreditor { messages, payment })
                        .await?;
                }
            }
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}"))
                .await?
                .id;
            repeat_state(dialogue, state, new_message).await?;
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a creditor string from user, and proceeds to ask for total.
 */
pub async fn action_add_creditor(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
    (mut messages, payment): (Vec<MessageId>, AddPaymentParams),
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let text = parse_username(text);

            if let Err(err) = text {
                let new_message = send_bot_message(&bot, &msg, err.to_string()).await?.id;
                repeat_state(dialogue, state, new_message).await?;
                return Ok(());
            }

            let new_payment = AddPaymentParams {
                chat_id: payment.chat_id,
                sender_id: payment.sender_id,
                sender_username: payment.sender_username,
                datetime: payment.datetime,
                description: payment.description,
                creditor: Some(text?),
                currency: None,
                total: None,
                debts: None,
            };
            let new_message = send_bot_message(
                &bot,
                &msg,
                format!(
                    "{}Nice! What was the budget?\n\n{TOTAL_INSTRUCTIONS_MESSAGE}",
                    display_add_payment(&new_payment)
                ),
            )
            .await?
            .id;
            messages.push(new_message);
            dialogue
                .update(State::AddTotal {
                    messages,
                    payment: new_payment,
                })
                .await?;
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}"))
                .await?
                .id;
            repeat_state(dialogue, state, new_message).await?;
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a total f64 from user, and proceeds to ask for debts.
 */
pub async fn action_add_total(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
    (mut messages, payment): (Vec<MessageId>, AddPaymentParams),
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let currency_amount = parse_currency_amount(text);
            match currency_amount {
                Ok((total, currency)) => {
                    let new_payment = AddPaymentParams {
                        chat_id: payment.chat_id,
                        sender_id: payment.sender_id,
                        sender_username: payment.sender_username,
                        datetime: payment.datetime,
                        description: payment.description,
                        creditor: payment.creditor,
                        currency: Some(currency),
                        total: Some(total),
                        debts: None,
                    };
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "{}Great! How do we split?\n\n{DEBT_EQUAL_DESCRIPTION_MESSAGE}{DEBT_EXACT_DESCRIPTION_MESSAGE}{DEBT_RATIO_DESCRIPTION_MESSAGE}",
                            display_add_payment(&new_payment)
                            ),
                            )
                        .reply_markup(make_keyboard_debt_selection())
                        .await?.id;
                    messages.push(new_message);
                    dialogue
                        .update(State::AddDebtSelection {
                            messages,
                            payment: new_payment,
                        })
                        .await?;
                }
                Err(err) => {
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!("{}\n\n{TOTAL_INSTRUCTIONS_MESSAGE}", err.to_string()),
                    )
                    .await?
                    .id;
                    repeat_state(dialogue, state, new_message).await?;
                    return Ok(());
                }
            }
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}"))
                .await?
                .id;
            repeat_state(dialogue, state, new_message).await?;
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a callback query from the user indicating how they want to split.
 * No Cancel button required.
 */
pub async fn action_add_debt_selection(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    (messages, payment): (Vec<MessageId>, AddPaymentParams),
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        match button.as_str() {
            "Equal" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "{}Okay! Who is involved in the payment?\n\n{DEBT_EQUAL_INSTRUCTIONS_MESSAGE}",
                            display_add_payment(&payment)
                            ),
                            )
                        .await?;
                    dialogue
                        .update(State::AddDebt {
                            messages,
                            payment,
                            debts_format: AddDebtsFormat::Equal,
                        })
                        .await?;
                }
            }
            "Exact" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "{}Okay! Provide usernames of debtors and their debt amounts.\n\n{DEBT_EXACT_INSTRUCTIONS_MESSAGE}",
                            display_add_payment(&payment)
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::AddDebt {
                            messages,
                            payment,
                            debts_format: AddDebtsFormat::Exact,
                        })
                        .await?;
                }
            }
            "Proportion" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "{}Okay! Provide usernames of debtors and their debt amounts.\n\n{DEBT_RATIO_INSTRUCTIONS_MESSAGE}",
                            display_add_payment(&payment))
                        ).await?;
                    dialogue
                        .update(State::AddDebt {
                            messages,
                            payment,
                            debts_format: AddDebtsFormat::Ratio,
                        })
                        .await?;
                }
            }
            _ => {
                log::error!("Add Payment Debt Selection - Invalid button for user {} in chat {} with payment {:?}: {}",
                            payment.sender_id, payment.chat_id, payment, button);
            }
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a Debt from user, and checks if the total amounts tally.
 * If so, it presents an overview. Else, it asks for more debts.
 */
pub async fn action_add_debt(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
    (messages, payment, debts_format): (Vec<MessageId>, AddPaymentParams, AddDebtsFormat),
) -> HandlerResult {
    handle_debts(bot, dialogue, state, msg, messages, payment, debts_format).await
}

/* Add a payment entry in a group chat.
 * Bot receives a callback query from a button menu, on user decision after seeing the overview.
 * If user chooses to edit, proceed to edit.
 * If user confirms, proceeds to add the payment.
 */
pub async fn action_add_confirm(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    query: CallbackQuery,
    (messages, payment): (Vec<MessageId>, AddPaymentParams),
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        match button.as_str() {
            "Cancel" => {
                if let Some(msg) = query.message {
                    cancel_add_payment(bot, dialogue, state, msg).await?;
                }
            }
            "Edit" => {
                display_add_edit_menu(bot, dialogue, query, messages, payment).await?;
            }
            "Confirm" => {
                call_processor_add_payment(bot, dialogue, messages, payment, query).await?;
            }
            _ => {
                log::error!("Add Payment Confirm - Invalid button for user {} in chat {} with payment {:?}: {}",
                            payment.sender_id, payment.chat_id, payment, button);
            }
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a callback query on user decision on what to edit.
 * If the user chooses to go back, return to confirm page.
 */
pub async fn action_add_edit_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    (messages, payment): (Vec<MessageId>, AddPaymentParams),
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        if let Some(msg) = query.message {
            let payment_clone = payment.clone();
            let chat_id = msg.chat.id;
            let id = msg.id;
            match button.as_str() {
                "Description" => {
                    bot.edit_message_text(
                        chat_id,
                        id,
                        format!(
                            "Current description: {}\n\nWhat should the description be?",
                            payment_clone.description.unwrap()
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::AddEdit {
                            messages,
                            payment,
                            edit: AddPaymentEdit::Description,
                        })
                        .await?;
                }
                "Payer" => {
                    bot.edit_message_text(
                        chat_id,
                        id,
                        format!(
                            "Current payer: {}\n\nWho should the payer be?",
                            display_username(&payment_clone.creditor.unwrap())
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::AddEdit {
                            messages,
                            payment,
                            edit: AddPaymentEdit::Creditor,
                        })
                        .await?;
                }
                "Total" => {
                    bot.edit_message_text(
                        chat_id,
                        id,
                        format!(
                            "Current total: {}\n\nWhat should the total be?\n\n{TOTAL_INSTRUCTIONS_MESSAGE}",
                            display_currency_amount(payment_clone.total.unwrap(), use_currency(payment_clone.currency.unwrap(), &payment_clone.chat_id))
                            ),
                            )
                        .await?;
                    dialogue
                        .update(State::AddEdit {
                            messages,
                            payment,
                            edit: AddPaymentEdit::Total,
                        })
                        .await?;
                }
                "Split" => {
                    bot.edit_message_text(
                        chat_id,
                        id,
                        format!(
                            "Current split:\n{}\nHow should we split this?\n\n{DEBT_EQUAL_DESCRIPTION_MESSAGE}{DEBT_EXACT_DESCRIPTION_MESSAGE}{DEBT_RATIO_DESCRIPTION_MESSAGE}",
                            display_debts(&payment_clone.debts.unwrap(), payment_clone.currency.unwrap().1)
                            ),
                            ).reply_markup(make_keyboard_debt_selection())
                        .await?;
                    dialogue
                        .update(State::AddDebtSelection { messages, payment })
                        .await?;
                }
                "Back" => {
                    display_add_overview(&bot, &dialogue, &msg, messages, payment).await?;
                }
                _ => {
                    log::error!("Add Payment Edit Menu - Invalid button for user {} in chat {} with payment {:?}: {}",
                                payment_clone.sender_id, payment_clone.chat_id, payment_clone, button);
                }
            }
        }
    }

    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a callback query on user decision on what to edit.
 * If the user chooses to go back, return to confirm page.
 */
pub async fn action_add_edit(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
    (mut messages, payment, edit): (Vec<MessageId>, AddPaymentParams, AddPaymentEdit),
) -> HandlerResult {
    match msg.text() {
        Some(text) => match edit {
            AddPaymentEdit::Description => {
                let new_payment = AddPaymentParams {
                    chat_id: payment.chat_id,
                    sender_id: payment.sender_id,
                    sender_username: payment.sender_username,
                    datetime: payment.datetime,
                    description: Some(text.to_string()),
                    creditor: payment.creditor,
                    currency: payment.currency,
                    total: payment.total,
                    debts: payment.debts,
                };
                display_add_overview(&bot, &dialogue, &msg, messages, new_payment).await?;
            }
            AddPaymentEdit::Creditor => {
                let username = parse_username(text);

                if let Err(err) = username {
                    let new_message = send_bot_message(&bot, &msg, err.to_string()).await?.id;
                    repeat_state(dialogue, state, new_message).await?;
                    return Ok(());
                }

                let new_payment = AddPaymentParams {
                    chat_id: payment.chat_id,
                    sender_id: payment.sender_id,
                    sender_username: payment.sender_username,
                    datetime: payment.datetime,
                    description: payment.description,
                    creditor: Some(username?),
                    currency: payment.currency,
                    total: payment.total,
                    debts: payment.debts,
                };
                display_add_overview(&bot, &dialogue, &msg, messages, new_payment).await?;
            }
            AddPaymentEdit::Total => {
                let currency_amount = parse_currency_amount(text);
                match currency_amount {
                    Ok((total, currency)) => {
                        let new_payment = AddPaymentParams {
                            chat_id: payment.chat_id,
                            sender_id: payment.sender_id,
                            sender_username: payment.sender_username,
                            datetime: payment.datetime,
                            description: payment.description,
                            creditor: payment.creditor,
                            currency: Some(currency),
                            total: Some(total),
                            debts: payment.debts,
                        };
                        let new_message = send_bot_message(&bot,
                            &msg,
                            format!("Great! How are we splitting this?\n\n{DEBT_EQUAL_DESCRIPTION_MESSAGE}{DEBT_EXACT_DESCRIPTION_MESSAGE}{DEBT_RATIO_DESCRIPTION_MESSAGE}",),
                            ).reply_markup(make_keyboard_debt_selection())
                            .await?.id;
                        messages.push(new_message);
                        dialogue
                            .update(State::AddDebtSelection {
                                messages,
                                payment: new_payment,
                            })
                            .await?;
                    }
                    Err(err) => {
                        let new_message = send_bot_message(
                            &bot,
                            &msg,
                            format!("{}\n\n{TOTAL_INSTRUCTIONS_MESSAGE}", err.to_string()),
                        )
                        .await?
                        .id;
                        repeat_state(dialogue, state, new_message).await?;

                        return Ok(());
                    }
                }
            }
            AddPaymentEdit::DebtsEqual => {
                handle_debts(
                    bot,
                    dialogue,
                    state,
                    msg,
                    messages,
                    payment,
                    AddDebtsFormat::Equal,
                )
                .await?;
            }
            AddPaymentEdit::DebtsExact => {
                handle_debts(
                    bot,
                    dialogue,
                    state,
                    msg,
                    messages,
                    payment,
                    AddDebtsFormat::Exact,
                )
                .await?;
            }
            AddPaymentEdit::DebtsRatio => {
                handle_debts(
                    bot,
                    dialogue,
                    state,
                    msg,
                    messages,
                    payment,
                    AddDebtsFormat::Ratio,
                )
                .await?;
            }
        },
        None => {
            let new_message = send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}"))
                .await?
                .id;
            repeat_state(dialogue, state, new_message).await?;
        }
    }

    Ok(())
}
