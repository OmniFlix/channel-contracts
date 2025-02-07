use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ChannelError {
    #[error("Channel ID not found")]
    ChannelIdNotFound {},

    #[error("Channel ID already exists")]
    ChannelIdAlreadyExists {},

    #[error("User name not found")]
    UserNameNotFound {},

    #[error("User name already taken")]
    UserNameAlreadyTaken {},

    #[error("Saving channel details failed")]
    SaveChannelDetailsFailed {},

    #[error("Saving reserved usernames failed")]
    SaveReservedUsernamesFailed {},

    #[error("Reserved username not found")]
    UsernameNotReserved {},

    #[error("Collaborator already exists")]
    CollaboratorExists {},

    #[error("Invalid share percentage")]
    InvalidSharePercentage {},

    #[error("Collaborator not found")]
    CollaboratorNotFound {},

    #[error("Collaborator expired")]
    CollaboratorExpired {},

    #[error("Total unique collaborators limit exceeded")]
    TotalUniqueCollaboratorsLimitExceeded {},

    #[error("Follower not found")]
    FollowerNotFound {},

    #[error("Already following")]
    AlreadyFollowing {},
}
