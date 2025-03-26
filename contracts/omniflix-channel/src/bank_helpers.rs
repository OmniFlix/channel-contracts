use cosmwasm_std::{Addr, Attribute, Coin, CosmosMsg, Decimal, Uint128};
use cw_utils::NativeBalance;

use crate::ContractError;

/// Validates that the received payment matches the expected amount
pub fn check_payment(expected: Vec<Coin>, received: Vec<Coin>) -> Result<(), ContractError> {
    let mut expected_balance = NativeBalance::default();
    for coin in expected.clone() {
        expected_balance += coin;
    }

    let mut received_balance = NativeBalance::default();
    for coin in received.clone() {
        received_balance += coin;
    }

    expected_balance.normalize();
    received_balance.normalize();

    if expected_balance != received_balance {
        return Err(ContractError::PaymentError { expected, received });
    }

    Ok(())
}

/// Creates a bank message for sending coins to a recipient, handling zero amounts gracefully
///
/// This helper function wraps the creation of a bank send message with validation:
/// - If the input amount contains zero coins, returns an empty vector to avoid transaction failures
/// - If the amount is non-zero, returns a vector containing the bank send message
///
/// # Arguments
/// * `recipient` - The address that will receive the coins
/// * `amount` - Vector of coins to send
///
/// # Returns
/// * `Vec<CosmosMsg>` - Empty vector if amount is zero, otherwise vector containing the bank message
pub fn bank_msg_wrapper(recipient: Addr, amount: Vec<Coin>) -> Vec<CosmosMsg> {
    let mut final_amount = NativeBalance::default();
    for coin in amount.clone() {
        final_amount += coin;
    }
    // Remove any zero coins
    final_amount.normalize();
    // If the final amount is empty, return an empty vec
    if final_amount.is_empty() {
        return vec![];
    }
    let bank_msg: CosmosMsg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: recipient.into_string(),
        amount: final_amount.0,
    });
    vec![bank_msg]
}

/// Distributes funds among collaborators according to their shares
/// and sends any remaining amount to the channel payment address
pub fn distribute_funds_with_shares(
    collaborators: Vec<(Addr, Decimal)>,
    amount: Coin,
    channel_payment_address: Addr,
) -> Result<(Vec<CosmosMsg>, Vec<Attribute>), ContractError> {
    let mut bank_msgs: Vec<CosmosMsg> = vec![];
    let mut remaining_amount = amount.clone().amount;
    let mut attributes: Vec<Attribute> = vec![];

    for (collaborator, share) in collaborators.clone() {
        // Create a decimal from the share
        let share_amount = Decimal::from_ratio(remaining_amount, Uint128::one()) * share;
        let uint_share_amount = share_amount.to_uint_floor();
        let share_amount_coin = Coin {
            denom: amount.denom.clone(),
            amount: uint_share_amount,
        };
        bank_msgs.extend(bank_msg_wrapper(
            collaborator.clone(),
            vec![share_amount_coin.clone()],
        ));
        remaining_amount -= uint_share_amount;
        attributes.push(Attribute::new(
            collaborator.to_string(),
            share_amount_coin.to_string(),
        ));
    }

    if !remaining_amount.is_zero() {
        let remaining_amount_coin = Coin {
            denom: amount.denom.clone(),
            amount: remaining_amount,
        };
        bank_msgs.extend(bank_msg_wrapper(
            channel_payment_address.clone(),
            vec![remaining_amount_coin.clone()],
        ));
        attributes.push(Attribute::new(
            channel_payment_address.to_string(),
            remaining_amount_coin.to_string(),
        ));
    }

    Ok((bank_msgs, attributes))
}
