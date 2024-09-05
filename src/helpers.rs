use cosmwasm_std::{Binary, Env};
use cosmwasm_std::{Coin, Deps, Uint128};
use omniflix_std::types::omniflix::onft::v1beta1::Onft;
use omniflix_std::types::omniflix::onft::v1beta1::OnftQuerier;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use thiserror::Error;

pub fn get_collection_creation_fee(deps: Deps) -> Coin {
    let onft_querier = OnftQuerier::new(&deps.querier);
    let collection_creation_fee = onft_querier
        .params()
        .unwrap()
        .params
        .unwrap()
        .denom_creation_fee
        .unwrap();
    let collection_creation_fee = Coin {
        denom: collection_creation_fee.denom,
        amount: Uint128::from_str(&collection_creation_fee.amount).unwrap(),
    };
    collection_creation_fee
}

#[derive(Error, Debug, PartialEq)]
pub enum OnftQuerierError {
    #[error("Asset not found")]
    AssetNotFound {},
}
pub fn get_onft(
    deps: Deps,
    collection_id: String,
    onft_id: String,
) -> Result<Option<Onft>, OnftQuerierError> {
    let onft_querier = OnftQuerier::new(&deps.querier);
    let onft = onft_querier
        .onft(collection_id, onft_id)
        .map_err(|_| OnftQuerierError::AssetNotFound {})?;
    Ok(onft.onft)
}

pub fn generate_random_id_with_prefix(
    onft_collection_id: &str,
    entropy: &str,
    salt: &Binary,
    env: &Env,
    prefix: &str,
) -> String {
    // Concatenate inputs
    let input = format!(
        "{}{}{}{}{}",
        onft_collection_id,
        entropy,
        salt,
        env.block.height,
        env.block.time.nanos()
    );

    // Create a SHA-256 hash of the concatenated string
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();

    // Truncate the hash to 32 characters
    let truncated = hex::encode(result).chars().take(32).collect::<String>();

    format!("{}{}", prefix, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_env;

    #[test]
    fn test_generate_publish_id() {
        let onft_collection_id = "onft_collection_id";
        let onft_id = "onft_id";
        let salt = Binary::from("salt".as_bytes());
        let env = mock_env();

        let publish_id =
            generate_random_id_with_prefix(onft_collection_id, onft_id, &salt, &env, "publish");
        println!("{}", publish_id);
    }
}
