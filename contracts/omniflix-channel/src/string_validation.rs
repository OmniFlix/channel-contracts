use thiserror::Error;

use crate::ContractError;

// Configuration for string validation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StringValidationConfig {
    min_length: usize,
    max_length: usize,
    allow_numbers: bool,
    allow_uppercase: bool,
    allow_spaces: bool,
    allow_special_chars: bool,
    required_prefixes: Vec<String>, // Required prefixes for the string
    required_suffixes: Vec<String>, // Required suffixes for the string
    must_contain: Vec<String>,      // Substrings that must be present
}

// Define string validation errors
#[derive(Error, Debug, PartialEq, Eq)]
pub enum StringValidationError {
    #[error("Invalid length sent: {sent} min_length: {min_length} max_length: {max_length}")]
    InvalidLength {
        sent: String,
        min_length: usize,
        max_length: usize,
    },
    #[error("Invalid prefix sent: {sent} required: {required:?}")]
    InvalidPrefix { sent: String, required: Vec<String> },
    #[error("Invalid suffix sent: {sent} required: {required:?}")]
    InvalidSuffix { sent: String, required: Vec<String> },
    #[error("Invalid must contain sent: {sent} required: {required:?}")]
    InvalidMustContain { sent: String, required: Vec<String> },
    #[error("Uppercase characters not allowed sent: {sent}")]
    UppercaseNotAllowed { sent: String },
    #[error("Numbers not allowed sent: {sent}")]
    NumbersNotAllowed { sent: String },
    #[error("Spaces not allowed sent: {sent}")]
    SpacesNotAllowed { sent: String },
    #[error("Special characters not allowed sent: {sent}")]
    SpecialCharsNotAllowed { sent: String },
}

// Default configuration for string validation
impl Default for StringValidationConfig {
    fn default() -> Self {
        StringValidationConfig {
            min_length: 3,
            max_length: 32,
            allow_numbers: false,
            allow_uppercase: false,
            allow_spaces: false,
            allow_special_chars: false,
            required_prefixes: vec![],
            required_suffixes: vec![],
            must_contain: vec![],
        }
    }
}

// Types of string validation
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringValidationType {
    Username,
    ChannelName,
    Description,
    Link,
    AssetName,
}
// Get configuration based on validation type
impl StringValidationType {
    fn get_config(&self) -> StringValidationConfig {
        match self {
            StringValidationType::Username => StringValidationConfig {
                min_length: 3,
                max_length: 32,
                allow_numbers: false,
                allow_uppercase: false,
                allow_spaces: false,
                allow_special_chars: false,
                ..Default::default()
            },
            StringValidationType::ChannelName => StringValidationConfig {
                min_length: 3,
                max_length: 32,
                allow_numbers: true,
                allow_uppercase: true,
                allow_spaces: false,
                allow_special_chars: false,
                ..Default::default()
            },
            StringValidationType::Description => StringValidationConfig {
                min_length: 3,
                max_length: 256,
                allow_numbers: true,
                allow_uppercase: true,
                allow_spaces: true,
                allow_special_chars: true,
                ..Default::default()
            },
            StringValidationType::Link => StringValidationConfig {
                min_length: 3,
                max_length: 256,
                allow_numbers: true,
                allow_uppercase: true,
                allow_spaces: false,
                allow_special_chars: true,
                required_prefixes: vec![
                    "http://".to_string(),
                    "https://".to_string(),
                    "ipfs://".to_string(),
                ],
                must_contain: vec![".".to_string()], // Ensure there's a dot in the link
                ..Default::default()
            },
            StringValidationType::AssetName => StringValidationConfig {
                min_length: 3,
                max_length: 64,
                allow_numbers: true,
                allow_uppercase: true,
                allow_spaces: true,
                allow_special_chars: false,
                ..Default::default()
            },
        }
    }
}

// Validate a string based on the specified validation type
pub fn validate_string(
    input: &str,
    validation_type: StringValidationType,
) -> Result<(), ContractError> {
    let config = validation_type.get_config();

    // Check length
    if !(config.min_length..=config.max_length).contains(&input.len()) {
        return Err(ContractError::StringValidationError(
            StringValidationError::InvalidLength {
                sent: input.to_string(),
                min_length: config.min_length,
                max_length: config.max_length,
            },
        ));
    }

    // Check required prefixes
    if !config.required_prefixes.is_empty() {
        let has_valid_prefix = config
            .required_prefixes
            .iter()
            .any(|prefix| input.starts_with(prefix));
        if !has_valid_prefix {
            return Err(ContractError::StringValidationError(
                StringValidationError::InvalidPrefix {
                    sent: input.to_string(),
                    required: config.required_prefixes.clone(),
                },
            ));
        }
    }

    // Check required suffixes
    if !config.required_suffixes.is_empty() {
        let has_valid_suffix = config
            .required_suffixes
            .iter()
            .any(|suffix| input.ends_with(suffix));
        if !has_valid_suffix {
            return Err(ContractError::StringValidationError(
                StringValidationError::InvalidSuffix {
                    sent: input.to_string(),
                    required: config.required_suffixes.clone(),
                },
            ));
        }
    }

    // Check must contain substrings
    for required_substring in &config.must_contain {
        if !input.contains(required_substring) {
            return Err(ContractError::StringValidationError(
                StringValidationError::InvalidMustContain {
                    sent: input.to_string(),
                    required: config.must_contain.clone(),
                },
            ));
        }
    }

    // Check character restrictions
    for c in input.chars() {
        match c {
            c if c.is_ascii_lowercase() => continue,
            c if c.is_ascii_uppercase() => {
                if !config.allow_uppercase {
                    return Err(ContractError::StringValidationError(
                        StringValidationError::UppercaseNotAllowed {
                            sent: input.to_string(),
                        },
                    ));
                }
            }
            c if c.is_ascii_digit() => {
                if !config.allow_numbers {
                    return Err(ContractError::StringValidationError(
                        StringValidationError::NumbersNotAllowed {
                            sent: input.to_string(),
                        },
                    ));
                }
            }
            ' ' => {
                if !config.allow_spaces {
                    return Err(ContractError::StringValidationError(
                        StringValidationError::SpacesNotAllowed {
                            sent: input.to_string(),
                        },
                    ));
                }
            }
            c if !c.is_ascii_alphanumeric() && c != ' ' => {
                if !config.allow_special_chars {
                    return Err(ContractError::StringValidationError(
                        StringValidationError::SpecialCharsNotAllowed {
                            sent: input.to_string(),
                        },
                    ));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        // Test valid username
        assert!(validate_string("validname", StringValidationType::Username).is_ok());

        // Test invalid cases
        assert!(validate_string("ab", StringValidationType::Username).is_err()); // too short
        assert!(validate_string("a".repeat(33).as_str(), StringValidationType::Username).is_err()); // too long
        assert!(validate_string("Invalid123", StringValidationType::Username).is_err()); // numbers not allowed
        assert!(validate_string("Invalid!", StringValidationType::Username).is_err()); // special chars not allowed
        assert!(validate_string("invalid name", StringValidationType::Username).is_err());
        // spaces not allowed
    }

    #[test]
    fn test_validate_channel_name() {
        // Test valid channel names
        assert!(validate_string("Channel123", StringValidationType::ChannelName).is_ok());
        assert!(validate_string("channelname", StringValidationType::ChannelName).is_ok());

        // Test invalid cases
        assert!(validate_string("ch", StringValidationType::ChannelName).is_err()); // too short
        assert!(
            validate_string("a".repeat(33).as_str(), StringValidationType::ChannelName).is_err()
        ); // too long
        assert!(validate_string("channel!", StringValidationType::ChannelName).is_err()); // special chars not allowed
        assert!(validate_string("channel name", StringValidationType::ChannelName).is_err());
        // spaces not allowed
    }

    #[test]
    fn test_validate_description() {
        // Test valid descriptions
        assert!(
            validate_string("Valid description 123!", StringValidationType::Description).is_ok()
        );
        assert!(validate_string("Short desc", StringValidationType::Description).is_ok());

        // Test invalid cases
        assert!(validate_string("ab", StringValidationType::Description).is_err()); // too short
        assert!(
            validate_string("a".repeat(257).as_str(), StringValidationType::Description).is_err()
        ); // too long
    }

    #[test]
    fn test_validate_link() {
        // Test valid links
        assert!(validate_string("https://example.com", StringValidationType::Link).is_ok());
        assert!(validate_string("http://test.org/path", StringValidationType::Link).is_ok());
        assert!(validate_string(
            "https://sub.domain.com/path?query=123",
            StringValidationType::Link
        )
        .is_ok());

        // Test invalid links
        assert!(validate_string("not-a-url", StringValidationType::Link).is_err());
        assert!(validate_string("ftp://invalid.com", StringValidationType::Link).is_err());
        assert!(validate_string("https://", StringValidationType::Link).is_err());
        assert!(validate_string("https://nodot", StringValidationType::Link).is_err());
    }
}
