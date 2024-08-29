// Error messages
pub const UNKNOWN_ERROR_MESSAGE: &str =
    "🤷 Oops! Something went wrong! I can't do that right now. Please try again later!\n\n";
pub const NO_TEXT_MESSAGE: &str = "❓ No text? Please reply to me in text!\n\n";

// Instruction messages
pub const TOTAL_INSTRUCTIONS_MESSAGE: &str =
    "Type the amount and currency (optional). For example: 100.00 USD, 200 MXN, 300.00, etc.\n\n";
pub const CURRENCY_INSTRUCTIONS_MESSAGE: &str =
    "Type the currency code. For example: USD, EUR, UAH, etc.\n\n";
pub const TIME_ZONE_INSTRUCTIONS_MESSAGE: &str =
    "Check out my User Guide with /help for all my supported time zones!"; //TODO
pub const DEBT_EQUAL_INSTRUCTIONS_MESSAGE: &str =
    "Share memeber's usernames, for example:\n\n@username_1\n@username_2\n@username_3\n...\n\n Don't forget to add the payer!";
pub const DEBT_EXACT_INSTRUCTIONS_MESSAGE: &str =
    "Share memeber's usernames and their amount stakes: \n\n@username_1 amount1\n@username_2 amount2\n@username_3 amount3\n...\n\n⭐️ If balance is positive, it's the payer's!";
pub const DEBT_RATIO_INSTRUCTIONS_MESSAGE: &str =
    "Share memeber's usernames and their portion stakes: \n\n@username_1 portion1\n@username_2 portion2\n@username_3 portion3\n...\n\n⭐️ It can be 100, 50, 33 etc";
pub const PAY_BACK_INSTRUCTIONS_MESSAGE: &str =
    "Enter the Telegram usernames and exact amounts like this: \n\n@username_1 amount1\n@username_2 amount2\n@username_3 amount3\n...\n\n";
pub const STATEMENT_INSTRUCTIONS_MESSAGE: &str = "I provide other currencies/formats below!";

// Description messages
pub const DEBT_EQUAL_DESCRIPTION_MESSAGE: &str = "Equal — Even amount for each user\n";
pub const DEBT_EXACT_DESCRIPTION_MESSAGE: &str = "Exact — Precise amount for each user\n";
pub const DEBT_RATIO_DESCRIPTION_MESSAGE: &str =
    "Share — Total amount is based on shares from 100% for each user\n";
pub const TIME_ZONE_DESCRIPTION: &str = "*Time Zone* — Your time zone";
pub const DEFAULT_CURRENCY_DESCRIPTION: &str = "*Default Currency* — System currency";
pub const CURRENCY_CONVERSION_DESCRIPTION: &str =
    "*Currency Conversion* — Convert currencies when calculating balances and spendings";
pub const ERASE_MESSAGES_DESCRIPTION: &str =
    "*Erase Messages* — I keep only the latest updates, the rest is deleted";

// Action messages

pub const CANCEL_ADD_MESSAGE: &str = "Okay! I cancelled <b>Add</b> payment action.";
pub const CANCEL_EDIT_MESSAGE: &str = "Okay! I cancelled <b>Edit</b> payment action.";
pub const CANCEL_DELETE_MESSAGE: &str = "Okay! I cancelled <b>Delete</b> payment action.";
pub const CANCEL_SETTINGS_MESSAGE: &str = "Okay! No settings were harmed!";
pub const BLANK_CANCEL: &str = "There's nothing to cancel!";

// Misc
pub const HEADER_MSG_HEAD: &str = "Anytime! \nI store ";
pub const HEADER_MSG_TAIL: &str = " payment records. Here are the latest entries!\n\n";
