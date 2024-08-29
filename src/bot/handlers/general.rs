use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

use crate::bot::{
    constants::{
        commands::{
            COMMAND_ADD_PAYMENT, COMMAND_BALANCES, COMMAND_DELETE_PAYMENT, COMMAND_EDIT_PAYMENT,
            /* COMMAND_HELP, */ COMMAND_PAY_BACK, COMMAND_SPENDINGS, COMMAND_VIEW_PAYMENTS,
        },
        messages::BLANK_CANCEL,
        // urls::{FEEDBACK_URL, USER_GUIDE_URL},
    },
    dispatcher::Command,
    processor::init_chat_config,
    utils::{
        bot_actions::{assert_handle_request_limit, send_bot_message},
        HandlerResult,
    },
};

/* Invalid state.
 * This action is invoked when the bot is in start state, and there is a non-command message
 * addressed to it.
 * Currently, simply does not respond to anything. Reduces spam.
 */
pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    // Checks if msg is a service message, ignores it if so
    let is_service_msg = msg.from().is_none();

    if is_service_msg {
        // Check if the message is SPECIFICALLY about the bot itself being added to a group
        let new_members = msg.new_chat_members();
        if let Some(new_members) = new_members {
            let bot_id = bot.get_me().send().await?.id;
            if new_members.iter().any(|member| member.id == bot_id) {
                action_start(bot, msg).await?;
            }
        }

        Ok(())
    } else {
        // send_bot_message(&bot, &msg, format!("Sorry, I'm not intelligent enough to process that! 🤖\nPlease refer to {COMMAND_HELP} on how to use me!")).await?;
        Ok(())
    }
}

/* Invalid message during callback expected.
 * Currently, simply does not respond to anything. Reduces spam.
 */
pub async fn callback_invalid_message(_bot: Bot, _msg: Message) -> HandlerResult {
    /*
    send_bot_message(
        &bot,
        &msg
        "Hey, you don't have to text me...\nJust click on any of the buttons above 👆 to continue!",
    )
    .await?;
    */
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    // Inits chat configs
    init_chat_config(&msg.chat.id.to_string())?;

    // TODO: Add to messages constant
    let intro = format!("Hello! I'm Finamaton!\n\nI'm tracking both individual and group expenses to simplify finance management");

    let add_info = &format!("Start with {COMMAND_ADD_PAYMENT}. You can {COMMAND_VIEW_PAYMENTS} anytime, and I'll help to {COMMAND_EDIT_PAYMENT} or {COMMAND_DELETE_PAYMENT}.");
    let view_info = &format!("Check out {COMMAND_SPENDINGS} to see overall spendings. Track {COMMAND_BALANCES} of those who owes what. To repay, use {COMMAND_PAY_BACK}");
    send_bot_message(
        &bot,
        &msg,
        format!("{intro}\n\n{add_info}\n\n{view_info}\n\n"),
    )
    .await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let mut commands = Command::descriptions().to_string();
    commands = commands.replace("–", "\\—");

    // TODO: Add to messages constant

    // let user_guide_info = &format!("🆘 For all the nitty\\-gritty details on supported 🕔 time zones, 💵 currencies, and more, check out my [User Guide]({USER_GUIDE_URL})\\!");

    send_bot_message(&bot, &msg, format!("*Commands*\n\n{}", commands))
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

/* Cancel command.
 * Called when state is at start, thus nothing to cancel.
 */
pub async fn action_cancel(bot: Bot, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    send_bot_message(
        &bot,
        &msg,
        format!("{BLANK_CANCEL}"), // TODO: Add to messages constant
    )
    .await?;
    Ok(())
}
