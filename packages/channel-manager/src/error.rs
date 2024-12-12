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
}
