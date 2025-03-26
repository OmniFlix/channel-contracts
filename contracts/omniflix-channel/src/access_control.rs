use crate::ContractError;
use channel_manager::channel::ChannelsManager;
use cosmwasm_std::{Addr, Deps};
use omniflix_channel_types::channel::{ChannelId, Role};
use omniflix_std::types::omniflix::onft::v1beta1::{Onft, OnftQuerier};

/// Validates if the sender has the required permissions to perform an action on a channel
///
/// # Arguments
/// * `deps` - Dependencies for accessing storage and querier
/// * `channel_id` - Identifier of the channel
/// * `sender` - Address of the account attempting the action
/// * `channels_collection_id` - Collection ID containing the channel NFTs
/// * `required_role` - Minimum role required to perform the action
///
/// # Returns
/// * `Ok(())` if the sender either:
///   - Owns the channel NFT (has admin privileges)
///   - Is a collaborator with sufficient role permissions
/// * `Err(ContractError::Unauthorized)` if the sender lacks required permissions
pub fn validate_permissions(
    deps: Deps,
    channel_id: ChannelId,
    sender: Addr,
    channels_collection_id: String,
    required_role: Role,
) -> Result<(), ContractError> {
    let channels = ChannelsManager::new();
    let channel_details = channels.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id;

    // First check if they own the channel NFT
    if let Ok(_channel_onft) = get_onft_with_owner(
        deps,
        channels_collection_id,
        channel_onft_id,
        sender.to_string(),
    ) {
        // Channel owner (NFT holder) has admin privileges
        return Ok(());
    }

    // If not the owner, check if user is a collaborator with sufficient privileges
    if let Ok(collaborator) = channels.get_collaborator(deps.storage, channel_id, sender) {
        if has_sufficient_privileges(collaborator.role, required_role) {
            return Ok(());
        }
    }

    // If neither owner nor collaborator with sufficient privileges, return error
    Err(ContractError::Unauthorized {})
}

// Helper function to check role hierarchy
fn has_sufficient_privileges(actual_role: Role, required_role: Role) -> bool {
    match (actual_role, required_role) {
        // Admin can do everything
        (Role::Admin, _) => true,

        // Moderator can do moderator and publisher tasks
        (Role::Moderator, Role::Moderator | Role::Publisher) => true,

        // Publisher can only do publisher tasks
        (Role::Publisher, Role::Publisher) => true,

        // All other combinations are insufficient privileges
        _ => false,
    }
}

pub fn get_onft_with_owner(
    deps: Deps,
    collection_id: String,
    onft_id: String,
    owner: String,
) -> Result<Onft, ContractError> {
    let onft_querier = OnftQuerier::new(&deps.querier);
    let onft_response = onft_querier
        .onft(collection_id.clone(), onft_id.clone())
        .map_err(|_| ContractError::OnftNotFound {
            collection_id: collection_id.clone(),
            onft_id: onft_id.clone(),
        })?;

    let onft = onft_response
        .onft
        .ok_or_else(|| ContractError::OnftNotFound {
            collection_id: collection_id.clone(),
            onft_id: onft_id.clone(),
        })?;

    if onft.owner != owner {
        return Err(ContractError::OnftNotOwned {
            collection_id,
            onft_id,
        });
    }

    Ok(onft)
}
