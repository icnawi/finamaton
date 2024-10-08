use super::{
    CHAT_CURRENCY_KEY, CHAT_KEY, CHAT_PAYMENT_KEY, CHAT_SETTING_KEY, SETTING_CURRENCY_CONVERSION,
    SETTING_DEFAULT_CURRENCY, SETTING_ERASE_MESSAGES, SETTING_TIME_ZONE,
};
use redis::{Commands, Connection, RedisResult};
use serde::{Deserialize, Serialize};

/* Chat CRUD Operations
 * Chat represents a chat, most likely a group chat on Telegram.
 * Chat comprises a list of usernames and a list of payments,
 * and the latest state of optimized debts.
 * Has add, exists, get, update, and delete operations.
 * Except for update chat payment operation, as there is no need to do so in application.
 * For debts, only set and get required, delete is purely for testing.
 */

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Debt {
    pub debtor: String,
    pub creditor: String,
    pub currency: String,
    pub amount: i64,
}

// Adds a new chat to Redis
pub fn add_chat(con: &mut Connection, chat_id: &str, username: &str) -> RedisResult<()> {
    con.rpush(format!("{CHAT_KEY}:{chat_id}"), username)
}

// Gets all users from a chat
// Returns a vector of usernames
pub fn get_chat_users(con: &mut Connection, chat_id: &str) -> RedisResult<Vec<String>> {
    con.lrange(format!("{CHAT_KEY}:{chat_id}"), 0, -1)
}

// Checks if chat exists
pub fn get_chat_exists(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    con.exists(format!("{CHAT_KEY}:{chat_id}"))
}

// Adds a single new user to the chat. Automatically checks if already added.
// Not in use in production, prefers add_chat_user_multiple
#[allow(dead_code)]
pub fn add_chat_user(con: &mut Connection, chat_id: &str, username: &str) -> RedisResult<()> {
    let current_users: Vec<String> = get_chat_users(con, chat_id)?;
    if current_users.contains(&username.to_string()) {
        return Ok(());
    }
    con.rpush(format!("{CHAT_KEY}:{chat_id}"), username)
}

// Adds more users to the chat. Automatically checks if already added.
pub fn add_chat_user_multiple(
    con: &mut Connection,
    chat_id: &str,
    users: Vec<String>,
) -> RedisResult<()> {
    let current_users: Vec<String> = get_chat_users(con, chat_id)?;
    for user in users {
        if !current_users.contains(&user) {
            con.rpush(format!("{CHAT_KEY}:{chat_id}"), user)?;
        }
    }

    Ok(())
}

// Deletes a chat from Redis
// Mainly for testing purposes
// In application, no real need to delete keys
#[allow(dead_code)]
pub fn delete_chat(con: &mut Connection, chat_id: &str) -> RedisResult<()> {
    con.del(format!("{CHAT_KEY}:{chat_id}"))
}

/* Chat Payment CRUD Operations */

// Adds a new payment to a chat
pub fn add_chat_payment(con: &mut Connection, chat_id: &str, payment_id: &str) -> RedisResult<()> {
    con.lpush(format!("{CHAT_PAYMENT_KEY}:{chat_id}"), payment_id)
}

// Checks if payments exist in a chat
pub fn get_chat_payment_exists(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    con.exists(format!("{CHAT_PAYMENT_KEY}:{chat_id}"))
}

// Gets all payments from a chat
pub fn get_chat_payments(con: &mut Connection, chat_id: &str) -> RedisResult<Vec<String>> {
    con.lrange(format!("{CHAT_PAYMENT_KEY}:{chat_id}"), 0, -1)
}

// Deletes a payment from a chat
pub fn delete_chat_payment(
    con: &mut Connection,
    chat_id: &str,
    payment_id: &str,
) -> RedisResult<()> {
    con.lrem(format!("{CHAT_PAYMENT_KEY}:{chat_id}"), 0, payment_id)
}

// Deletes all payments from a chat
// Mainly for testing purposes
// In application, no real need to delete keys
#[allow(dead_code)]
pub fn delete_all_chat_payment(con: &mut Connection, chat_id: &str) -> RedisResult<()> {
    con.del(format!("{CHAT_PAYMENT_KEY}:{chat_id}"))
}

/* Chat Currency CRUD Operations */
// Adds a currency to a chat
pub fn add_chat_currency(con: &mut Connection, chat_id: &str, currency: &str) -> RedisResult<()> {
    con.rpush(format!("{CHAT_CURRENCY_KEY}:{chat_id}"), currency)
}

// Gets all currencies from a chat
pub fn get_chat_currencies(con: &mut Connection, chat_id: &str) -> RedisResult<Vec<String>> {
    con.lrange(format!("{CHAT_CURRENCY_KEY}:{chat_id}"), 0, -1)
}

// Deletes all currencies from a chat
// Mainly for testing purposes
// In application, no real need to delete keys
#[allow(dead_code)]
pub fn delete_chat_currencies(con: &mut Connection, chat_id: &str) -> RedisResult<()> {
    con.del(format!("{CHAT_CURRENCY_KEY}:{chat_id}"))
}

/* Chat Setting CRUD Operations */
// Sets time zone for a chat
pub fn set_chat_time_zone(con: &mut Connection, chat_id: &str, time_zone: &str) -> RedisResult<()> {
    con.hset(
        format!("{CHAT_SETTING_KEY}:{chat_id}"),
        SETTING_TIME_ZONE,
        time_zone,
    )
}

// Sets default currency for a chat
pub fn set_chat_default_currency(
    con: &mut Connection,
    chat_id: &str,
    currency: &str,
) -> RedisResult<()> {
    con.hset(
        format!("{CHAT_SETTING_KEY}:{chat_id}"),
        SETTING_DEFAULT_CURRENCY,
        currency,
    )
}

// Sets currency conversion for a chat
pub fn set_chat_currency_conversion(
    con: &mut Connection,
    chat_id: &str,
    currency_conversion: bool,
) -> RedisResult<()> {
    con.hset(
        format!("{CHAT_SETTING_KEY}:{chat_id}"),
        SETTING_CURRENCY_CONVERSION,
        currency_conversion,
    )
}

// Sets erase messages for a chat
pub fn set_chat_erase_messages(
    con: &mut Connection,
    chat_id: &str,
    erase_messages: bool,
) -> RedisResult<()> {
    con.hset(
        format!("{CHAT_SETTING_KEY}:{chat_id}"),
        SETTING_ERASE_MESSAGES,
        erase_messages,
    )
}

// Checks if time zone exists for a chat
pub fn is_exists_chat_time_zone(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    let keys: Vec<String> = con.hkeys(format!("{CHAT_SETTING_KEY}:{chat_id}"))?;
    if keys.contains(&SETTING_TIME_ZONE.to_string()) {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Checks if default currency exists for a chat
pub fn is_exists_chat_default_currency(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    let keys: Vec<String> = con.hkeys(format!("{CHAT_SETTING_KEY}:{chat_id}"))?;
    if keys.contains(&SETTING_DEFAULT_CURRENCY.to_string()) {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Checks if currency conversion exists for a chat
pub fn is_exists_chat_currency_conversion(
    con: &mut Connection,
    chat_id: &str,
) -> RedisResult<bool> {
    let keys: Vec<String> = con.hkeys(format!("{CHAT_SETTING_KEY}:{chat_id}"))?;
    if keys.contains(&SETTING_CURRENCY_CONVERSION.to_string()) {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Checks if erase messages exists for a chat
pub fn is_exists_chat_erase_messages(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    let keys: Vec<String> = con.hkeys(format!("{CHAT_SETTING_KEY}:{chat_id}"))?;
    if keys.contains(&SETTING_ERASE_MESSAGES.to_string()) {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Gets time zone for a chat
pub fn get_chat_time_zone(con: &mut Connection, chat_id: &str) -> RedisResult<String> {
    con.hget(format!("{CHAT_SETTING_KEY}:{chat_id}"), SETTING_TIME_ZONE)
}

// Gets default currency for a chat
pub fn get_chat_default_currency(con: &mut Connection, chat_id: &str) -> RedisResult<String> {
    con.hget(
        format!("{CHAT_SETTING_KEY}:{chat_id}"),
        SETTING_DEFAULT_CURRENCY,
    )
}

// Gets currency conversion for a chat
pub fn get_chat_currency_conversion(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    con.hget(
        format!("{CHAT_SETTING_KEY}:{chat_id}"),
        SETTING_CURRENCY_CONVERSION,
    )
}

// Gets erase messages for a chat
pub fn get_chat_erase_messages(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    con.hget(
        format!("{CHAT_SETTING_KEY}:{chat_id}"),
        SETTING_ERASE_MESSAGES,
    )
}

// Deletes chat settings
// Mainly for testing purposes
// In application, no real need to delete keys
#[allow(dead_code)]
pub fn delete_chat_settings(con: &mut Connection, chat_id: &str) -> RedisResult<()> {
    con.del(format!("{CHAT_SETTING_KEY}:{chat_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::redis::connect::connect;

    #[test]
    fn test_add_chat() {
        let mut con = connect().unwrap();

        let chat_id = "123456789";
        let username = "987654321";
        assert!(add_chat(&mut con, chat_id, username).is_ok());

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_get_chat_exists() {
        let mut con = connect().unwrap();

        let chat_id = "1234567891";
        let username = "9876543211";
        add_chat(&mut con, chat_id, username).unwrap();
        assert!(get_chat_exists(&mut con, chat_id).unwrap());

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_get_chat_users() {
        let mut con = connect().unwrap();

        let chat_id = "1234567890";
        let username = "9876543210";
        add_chat(&mut con, chat_id, username).unwrap();
        let users = get_chat_users(&mut con, chat_id);
        assert!(users.is_ok());
        assert_eq!(users.unwrap(), vec![username.to_string()]);

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_add_user_to_chat() {
        let mut con = connect().unwrap();

        let chat_id = "1234567892";
        let username = "9876543212";
        let new_username = "9876543213";
        add_chat(&mut con, chat_id, username).unwrap();
        assert!(add_chat_user(&mut con, chat_id, new_username).is_ok());

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_add_users_to_chat() {
        let mut con = connect().unwrap();

        let chat_id = "1234567893";
        let first_user = "987654321";
        let users = vec![
            "987654322".to_string(),
            "987654323".to_string(),
            "987654324".to_string(),
        ];
        add_chat(&mut con, chat_id, first_user).unwrap();
        assert!(add_chat_user_multiple(&mut con, chat_id, users).is_ok());
        assert_eq!(
            get_chat_users(&mut con, chat_id).unwrap(),
            vec![
                "987654321".to_string(),
                "987654322".to_string(),
                "987654323".to_string(),
                "987654324".to_string(),
            ]
        );

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_delete_chat() {
        let mut con = connect().unwrap();

        let chat_id = "1234567894";
        let username = "9876543216";
        add_chat(&mut con, chat_id, username).unwrap();
        assert!(get_chat_exists(&mut con, chat_id).unwrap());
        delete_chat(&mut con, chat_id).unwrap();
        assert!(!get_chat_exists(&mut con, chat_id).unwrap());
    }

    #[test]
    fn test_add_get_chat_payment() {
        let mut con = connect().unwrap();

        let chat_id = "1234567895";
        let payment_id = "payment_id_1";
        assert!(add_chat_payment(&mut con, chat_id, payment_id).is_ok());
        assert!(get_chat_payment_exists(&mut con, chat_id).is_ok());
        assert!(get_chat_payments(&mut con, chat_id).unwrap() == vec![payment_id]);

        let second_payment_id = "payment_id_2";
        assert!(add_chat_payment(&mut con, chat_id, second_payment_id).is_ok());
        assert!(
            get_chat_payments(&mut con, chat_id).unwrap() == vec![second_payment_id, payment_id]
        );

        delete_all_chat_payment(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_delete_chat_payment() {
        let mut con = connect().unwrap();

        let chat_id = "1234567896";
        let payment_id = "payment_id_2";
        add_chat_payment(&mut con, chat_id, payment_id).unwrap();
        let payment_id_second = "payment_id_3";
        add_chat_payment(&mut con, chat_id, payment_id_second).unwrap();
        let payment_id_third = "payment_id_4";
        add_chat_payment(&mut con, chat_id, payment_id_third).unwrap();
        delete_chat_payment(&mut con, chat_id, payment_id_second).unwrap();

        assert_eq!(
            get_chat_payments(&mut con, chat_id).unwrap(),
            vec![payment_id_third, payment_id]
        );
        delete_all_chat_payment(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_delete_all_chat_payment() {
        let mut con = connect().unwrap();

        let chat_id = "1234567897";
        let payment_id = "payment_id_5";
        add_chat_payment(&mut con, chat_id, payment_id).unwrap();
        delete_all_chat_payment(&mut con, chat_id).unwrap();
        assert!(!get_chat_payment_exists(&mut con, chat_id).unwrap());
    }

    #[test]
    fn test_add_get_chat_currency() {
        let mut con = connect().unwrap();

        let chat_id = "1234567899";
        let currency = "USD";
        assert!(add_chat_currency(&mut con, chat_id, currency).is_ok());
        assert_eq!(
            get_chat_currencies(&mut con, chat_id).unwrap(),
            vec![currency]
        );

        let second_currency = "EUR";
        assert!(add_chat_currency(&mut con, chat_id, second_currency).is_ok());
        assert_eq!(
            get_chat_currencies(&mut con, chat_id).unwrap(),
            vec![currency, second_currency]
        );
        assert!(delete_chat_currencies(&mut con, chat_id).is_ok());
    }

    #[test]
    fn test_set_get_chat_time_zone() {
        let mut con = connect().unwrap();

        let chat_id = "12345678900";
        let time_zone = "SST";

        assert!(!is_exists_chat_time_zone(&mut con, chat_id).unwrap());
        assert!(set_chat_time_zone(&mut con, chat_id, time_zone).is_ok());
        assert_eq!(
            get_chat_time_zone(&mut con, chat_id).unwrap(),
            time_zone.to_string()
        );
        assert!(is_exists_chat_time_zone(&mut con, chat_id).unwrap());

        let second_time_zone = "PST";
        assert!(set_chat_time_zone(&mut con, chat_id, second_time_zone).is_ok());
        assert_eq!(
            get_chat_time_zone(&mut con, chat_id).unwrap(),
            second_time_zone.to_string()
        );

        assert!(delete_chat_settings(&mut con, chat_id).is_ok());
    }

    #[test]
    fn test_set_get_chat_default_currency() {
        let mut con = connect().unwrap();

        let chat_id = "12345678901";
        let currency = "USD";

        assert!(!is_exists_chat_default_currency(&mut con, chat_id).unwrap());
        assert!(set_chat_default_currency(&mut con, chat_id, currency).is_ok());
        assert_eq!(
            get_chat_default_currency(&mut con, chat_id).unwrap(),
            currency.to_string()
        );
        assert!(is_exists_chat_default_currency(&mut con, chat_id).unwrap());

        let second_currency = "EUR";
        assert!(set_chat_default_currency(&mut con, chat_id, second_currency).is_ok());
        assert_eq!(
            get_chat_default_currency(&mut con, chat_id).unwrap(),
            second_currency.to_string()
        );

        assert!(delete_chat_settings(&mut con, chat_id).is_ok());
    }

    #[test]
    fn test_set_get_chat_currency_conversion() {
        let mut con = connect().unwrap();

        let chat_id = "12345678902";
        let currency_conversion = true;

        assert!(!is_exists_chat_currency_conversion(&mut con, chat_id).unwrap());
        assert!(set_chat_currency_conversion(&mut con, chat_id, currency_conversion).is_ok());
        assert_eq!(
            get_chat_currency_conversion(&mut con, chat_id).unwrap(),
            currency_conversion
        );
        assert!(is_exists_chat_currency_conversion(&mut con, chat_id).unwrap());

        let second_currency_conversion = false;
        assert!(
            set_chat_currency_conversion(&mut con, chat_id, second_currency_conversion).is_ok()
        );
        assert_eq!(
            get_chat_currency_conversion(&mut con, chat_id).unwrap(),
            second_currency_conversion
        );

        assert!(delete_chat_settings(&mut con, chat_id).is_ok());
    }

    #[test]
    fn test_set_get_chat_erase_messages() {
        let mut con = connect().unwrap();

        let chat_id = "12345678903";
        let erase_messages = true;

        assert!(!is_exists_chat_erase_messages(&mut con, chat_id).unwrap());
        assert!(set_chat_erase_messages(&mut con, chat_id, erase_messages).is_ok());
        assert_eq!(
            get_chat_erase_messages(&mut con, chat_id).unwrap(),
            erase_messages
        );
        assert!(is_exists_chat_erase_messages(&mut con, chat_id).unwrap());

        let second_erase_messages = false;
        assert!(set_chat_erase_messages(&mut con, chat_id, second_erase_messages).is_ok());
        assert_eq!(
            get_chat_erase_messages(&mut con, chat_id).unwrap(),
            second_erase_messages
        );

        assert!(delete_chat_settings(&mut con, chat_id).is_ok());
    }
}
