// Error messages
pub const UNKNOWN_ERROR_MESSAGE: &str =
    "Oops! Something went wrong! I can't do that right now. Please try again later!\n\n";
pub const NO_TEXT_MESSAGE: &str =
    "❓ I'm having a bit of trouble understanding that! Please reply to me in text!\n\n";

// Instruction messages
pub const TOTAL_INSTRUCTIONS_MESSAGE: &str =
    "Enter the amount and optionally, the 3-letter currency code. For example: 100.00 USD, 200 JPY, 300.00, etc.\n\n⭐️ If you're unsure of the currency code, you can always check out my User Guide with /help!";
pub const CURRENCY_INSTRUCTIONS_MESSAGE: &str =
    "Enter the 3-letter currency code. For example: USD, EUR, JPY, etc.\n\n⭐️ If you're unsure of the currency code, you can always check out my User Guide with /help!";
pub const TIME_ZONE_INSTRUCTIONS_MESSAGE: &str =
    "⭐️ Check out my User Guide with /help for all my supported time zones!";
pub const DEBT_EQUAL_INSTRUCTIONS_MESSAGE: &str =
    "Enter the Telegram usernames of everyone sharing like this:\n\n@username__1\n@username__2\n@username__3\n...\n\n⭐️ Remember to include the payer if they're chipping in too!";
pub const DEBT_EXACT_INSTRUCTIONS_MESSAGE: &str =
    "Enter the Telegram usernames and exact amounts like this: \n\n@username__1 amount1\n@username__2 amount2\n@username__3 amount3\n...\n\n⭐️ If there are any leftover amounts, I'll assume it's the payer's!";
pub const DEBT_RATIO_INSTRUCTIONS_MESSAGE: &str =
    "Enter the Telegram usernames and portions like this: \n\n@username__1 portion1\n@username__2 portion2\n@username__3 portion3\n...\n\n⭐️ I can work with any positive number, whole or decimal!";
pub const PAY_BACK_INSTRUCTIONS_MESSAGE: &str =
    "Enter the Telegram usernames and exact amounts like this: \n\n@username__1 amount1\n@username__2 amount2\n@username__3 amount3\n...\n\n";
pub const STATEMENT_INSTRUCTIONS_MESSAGE: &str =
    "⭐️ I can also present the other currencies/formats below!";

// Description messages
pub const DEBT_EQUAL_DESCRIPTION_MESSAGE: &str =
    "Equal — Divide the total amount equally among users\n";
pub const DEBT_EXACT_DESCRIPTION_MESSAGE: &str =
    "Exact — Split the total cost by exact amounts for each user\n";
pub const DEBT_RATIO_DESCRIPTION_MESSAGE: &str =
    "Proportion — Share the total cost by relative proportions for each user\n";
pub const TIME_ZONE_DESCRIPTION: &str = "*🕔 Time Zone* — Time zone for displaying date and time";
pub const DEFAULT_CURRENCY_DESCRIPTION: &str =
    "💵 *Default Currency* — Currency used if left blank";
pub const CURRENCY_CONVERSION_DESCRIPTION: &str =
    "↔️ *Currency Conversion* — Convert currencies when calculating balances and spendings";
pub const ERASE_MESSAGES_DESCRIPTION: &str =
    "🚮 *Erase Messages* — Keep only the final updates and automatically delete my other messages";

// Action messages
pub const CANCEL_ADD_MESSAGE: &str =
    "Okay! I've cancelled adding the payment. No changes have been made! 🌟";
pub const CANCEL_EDIT_MESSAGE: &str =
    "Okay! I've cancelled the edit. No changes have been made! 🌟";
pub const CANCEL_DELETE_MESSAGE: &str =
    "Okay! I've cancelled deleting the payment. No changes have been made! 🌟";
pub const CANCEL_SETTINGS_MESSAGE: &str = "Okay! No changes to my settings have been made! 🌟";

// Misc
pub const HEADER_MSG_HEAD: &str = "Anytime! ☺️\nI've recorded ";
pub const HEADER_MSG_TAIL: &str = " payments. Here are the latest entries!\n\n";
