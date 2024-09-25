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

    #[error("Invalid user name")]
    InvalidUserName {},

    #[error("Invalid description")]
    InvalidDescription {},

    #[error("Saving channel details failed")]
    SaveChannelDetailsFailed {},
}
