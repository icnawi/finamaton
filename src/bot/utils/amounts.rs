use std::collections::HashSet;

use crate::bot::{
    constants::misc::MAX_VALUE,
    currency::{get_default_currency, Currency},
    handlers::AddDebtsFormat,
    processor::is_username_equal,
};

use super::{
    format::{get_currency, parse_username},
    BotError,
};

// Parse an amount. Reads a string, returns i64 based on currency.
pub fn parse_amount(text: &str, decimal_places: i32) -> Result<i64, BotError> {
    let factor = 10.0_f64.powi(decimal_places);
    let amount = match text.parse::<i64>() {
        Ok(val) => (val as f64 * factor).round() as i64,
        Err(_) => match text.parse::<f64>() {
            Ok(val) => (val * factor).round() as i64,
            Err(_) => {
                return Err(BotError::UserError(
                    "Uh-oh! ❌ Please give me a valid number!".to_string(),
                ))
            }
        },
    };

    if amount > MAX_VALUE {
        Err(BotError::UserError(
            "Uh-oh! 🥺 This number is too large for me to handle!".to_string(),
        ))
    } else if amount <= 0 {
        Err(BotError::UserError(
            "Uh-oh! ❌ Please give me a positive number!".to_string(),
        ))
    } else {
        Ok(amount)
    }
}

// Parse a float. Reads a string, returns f64.
pub fn parse_float(text: &str) -> Result<f64, BotError> {
    let amount = match text.parse::<f64>() {
        Ok(val) => val,
        Err(_) => match text.parse::<i32>() {
            Ok(val) => val as f64,
            Err(_) => {
                return Err(BotError::UserError(
                    "Uh-oh! ❌ Please give me a valid number!".to_string(),
                ))
            }
        },
    };

    if amount > MAX_VALUE as f64 {
        Err(BotError::UserError(
            "Uh-oh! 🥺 This number is too large for me to handle!".to_string(),
        ))
    } else if amount <= 0.0 {
        Err(BotError::UserError(
            "Uh-oh! ❌ Please give me a positive number!".to_string(),
        ))
    } else {
        Ok(amount)
    }
}
// Parse a string representing an amount and a currency
pub fn parse_currency_amount(text: &str) -> Result<(i64, Currency), BotError> {
    let items = text.split_whitespace().collect::<Vec<&str>>();
    if items.len() > 2 {
        return Err(BotError::UserError(
            "Uh-oh! ❌ I don't understand... Please use the following format!".to_string(),
        ));
    } else if items.len() == 1 {
        let currency = get_default_currency();
        let amount = parse_amount(items[0], currency.1)?;
        Ok((amount, currency))
    } else {
        let currency = get_currency(&items[1])?;
        let amount = parse_amount(items[0], currency.1)?;
        Ok((amount, currency))
    }
}

// Parse and process a string to retrieve a list of debts, for split by equal amount.
pub fn process_debts_equal(text: &str, total: Option<i64>) -> Result<Vec<(String, i64)>, BotError> {
    let mut users = text.split_whitespace().collect::<Vec<&str>>();
    if users.len() == 0 {
        return Err(BotError::UserError(
            "Uh-oh! ❌ Please give me at least one username!".to_string(),
        ));
    }

    let total = match total {
        Some(val) => val,
        None => {
            return Err(BotError::UserError(
                "Uh-oh! ❌ The total amount isn't provided.".to_string(),
            ));
        }
    };

    let mut accounted_users: HashSet<String> = HashSet::new();
    let mut i = 0;
    while i < users.len() {
        let user = users[i];
        if accounted_users.contains(&user.to_lowercase()) {
            users.remove(i);
        } else {
            accounted_users.insert(user.to_lowercase());
            i += 1;
        }
    }

    let amount = (total as f64 / users.len() as f64).round() as i64;
    let diff = total - amount * users.len() as i64;

    let mut debts: Vec<(String, i64)> = Vec::new();
    for user in &users {
        let username = parse_username(user)?;
        let debt = (username.clone(), amount);
        debts.push(debt);
    }

    // Distribute the difference in amount to as many users as required through smallest denomination
    for i in 0..(diff).abs() {
        debts[i as usize].1 += if diff > 0 { 1 } else { -1 };
    }

    Ok(debts)
}

// Parse and process a string to retrieve a list of debts, for split by exact amount.
pub fn process_debts_exact(
    text: &str,
    creditor: &Option<String>,
    currency: Option<Currency>,
    total: Option<i64>,
) -> Result<Vec<(String, i64)>, BotError> {
    if let Some(creditor) = creditor {
        if let Some(total) = total {
            if let Some(currency) = currency {
                let mut debts: Vec<(String, i64)> = Vec::new();
                let mut sum: i64 = 0;
                let items: Vec<&str> = text.split_whitespace().collect();
                if items.len() % 2 != 0 {
                    return Err(BotError::UserError(
                        "Uh-oh! ❌ I don't understand... Please use the following format!"
                            .to_string(),
                    ));
                }

                for i in (0..items.len()).step_by(2) {
                    let username = parse_username(items[i])?;
                    let amount = parse_amount(items[i + 1], currency.1)?;
                    sum += amount;

                    let mut found = false;
                    for debt in &mut debts {
                        if debt.0 == username {
                            debt.1 += amount;
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        debts.push((username, amount));
                    }
                }

                if sum > total {
                    Err(BotError::UserError(
                        "Uh-oh! ❌ The amounts you gave me are more than the total paid!"
                            .to_string(),
                    ))
                } else if sum < total {
                    for debt in &mut debts {
                        if debt.0 == creditor.to_string() {
                            debt.1 += total - sum;
                            return Ok(debts);
                        }
                    }

                    debts.push((creditor.to_string(), total - sum));
                    Ok(debts)
                } else {
                    Ok(debts)
                }
            } else {
                Err(BotError::UserError(
                    "Uh-oh! ❌ The currency isn't provided.".to_string(),
                ))
            }
        } else {
            Err(BotError::UserError(
                "Uh-oh! ❌ The total amount isn't provided.".to_string(),
            ))
        }
    } else {
        Err(BotError::UserError(
            "Uh-oh! ❌ The payer isn't provided.".to_string(),
        ))
    }
}

// Parse and process a string to retrieve a list of debts, returns Vec<Debt>.
pub fn process_debts(
    debts_format: AddDebtsFormat,
    text: &str,
    creditor: &Option<String>,
    currency: Option<Currency>,
    total: Option<i64>,
) -> Result<Vec<(String, i64)>, BotError> {
    match debts_format {
        AddDebtsFormat::Equal => process_debts_equal(text, total),
        AddDebtsFormat::Exact => process_debts_exact(text, creditor, currency, total),
        AddDebtsFormat::Ratio => process_debts_ratio(text, total),
    }
}

// Parse and process a string to retrieve a list of debts, for split by ratio.
pub fn process_debts_ratio(text: &str, total: Option<i64>) -> Result<Vec<(String, i64)>, BotError> {
    let items: Vec<&str> = text.split_whitespace().collect();
    let mut debts_ratioed: Vec<(String, f64)> = Vec::new();
    let mut debts: Vec<(String, i64)> = Vec::new();
    let mut sum: f64 = 0.0;

    if items.len() % 2 != 0 {
        return Err(BotError::UserError(
            "Uh-oh! ❌ I don't understand... Please use the following format!".to_string(),
        ));
    }

    let mut users: Vec<String> = Vec::new();
    let mut ratios: Vec<f64> = Vec::new();

    for i in (0..items.len()).step_by(2) {
        let curr = parse_username(items[i])?;
        let pos = users.iter().position(|u| is_username_equal(u, &curr));
        match pos {
            Some(pos) => {
                ratios[pos] += parse_float(items[i + 1])?;
            }
            None => {
                users.push(curr.to_string());
                ratios.push(parse_float(items[i + 1])?);
            }
        }
    }

    for i in 0..users.len() {
        let username = &users[i];
        sum += ratios[i];
        debts_ratioed.push((username.to_string(), ratios[i]));
    }

    let total = match total {
        Some(val) => val,
        None => {
            return Err(BotError::UserError(
                "Uh-oh! ❌ The total amount isn't provided.".to_string(),
            ));
        }
    };

    let mut exact_sum: i64 = 0;
    for debt in &mut debts_ratioed {
        let amount = ((debt.1 / sum) * total as f64).round() as i64;
        debts.push((debt.0.clone(), amount));
        exact_sum += amount;
    }

    // Distribute the difference in amount to as many users as required through smallest denomination
    let diff = total - exact_sum;
    for i in 0..(diff).abs() {
        debts[i as usize].1 += if diff > 0 { 1 } else { -1 };
    }

    Ok(debts)
}

pub fn parse_debts_payback(
    text: &str,
    currency: Currency,
    sender: &str,
) -> Result<Vec<(String, i64)>, BotError> {
    let mut debts: Vec<(String, i64)> = Vec::new();
    let items: Vec<&str> = text.split_whitespace().collect();
    if items.len() % 2 != 0 {
        return Err(BotError::UserError(
            "Uh-oh! ❌ I don't understand... Please use the following format!".to_string(),
        ));
    }

    for i in (0..items.len()).step_by(2) {
        let username = parse_username(items[i])?;
        let amount = parse_amount(items[i + 1], currency.1)?;
        if is_username_equal(&username, sender) {
            return Err(BotError::UserError(
                "Uh-oh! ❌ You can't pay back yourself!".to_string(),
            ));
        }
        let mut found = false;
        for debt in &mut debts {
            if debt.0 == username {
                debt.1 += amount;
                found = true;
                break;
            }
        }
        if !found {
            debts.push((username, amount));
        }
    }

    Ok(debts)
}
