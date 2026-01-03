//! Request/response schemas with validation.
//!
//! This module contains DTOs (Data Transfer Objects) for API requests and responses.
//! Schemas use the `validator` crate for input validation and `serde` for serialization.
//!
//! For database models, see the `models` module (when created).
//!
//! # Example
//!
//! ```rust
//! use allmaptout_backend::schemas::CreateUser;
//! use validator::Validate;
//!
//! let user = CreateUser {
//!     email: "test@example.com".into(),
//!     name: "Test User".into(),
//! };
//!
//! assert!(user.validate().is_ok());
//! ```

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Trait for validating request payloads.
/// Implemented automatically for types that derive `Validate`.
pub trait ValidatedRequest: Validate {
    /// Validate the request and return a list of field errors if invalid.
    fn validate_request(&self) -> Result<(), Vec<FieldError>> {
        self.validate().map_err(|errors| {
            errors
                .field_errors()
                .iter()
                .flat_map(|(field, errs)| {
                    errs.iter().map(move |e| FieldError {
                        field: field.to_string(),
                        message: e
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| format!("Invalid value for {}", field)),
                    })
                })
                .collect()
        })
    }
}

// Blanket implementation for all types that implement Validate
impl<T: Validate> ValidatedRequest for T {}

/// A validation error for a specific field.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FieldError {
    /// The field that failed validation.
    pub field: String,
    /// A human-readable error message.
    pub message: String,
}

/// Response body for validation errors.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationErrorResponse {
    /// The error type.
    pub error: String,
    /// List of field-level validation errors.
    pub fields: Vec<FieldError>,
}

impl ValidationErrorResponse {
    /// Create a new validation error response from field errors.
    pub fn new(fields: Vec<FieldError>) -> Self {
        Self {
            error: "Validation failed".to_string(),
            fields,
        }
    }
}

// ============================================================================
// Example schemas - delete or modify these when building your features
// ============================================================================

/// Example: User creation request with validation.
///
/// ```rust
/// use allmaptout_backend::schemas::CreateUser;
/// use validator::Validate;
///
/// let valid = CreateUser {
///     email: "user@example.com".into(),
///     name: "John Doe".into(),
/// };
/// assert!(valid.validate().is_ok());
///
/// let invalid = CreateUser {
///     email: "not-an-email".into(),
///     name: "".into(),
/// };
/// assert!(invalid.validate().is_err());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUser {
    /// User's email address.
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    /// User's display name.
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
}

/// Example: Pagination parameters with validation.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct PaginationParams {
    /// Page number (1-indexed).
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    #[serde(default = "default_page")]
    pub page: u32,

    /// Number of items per page.
    #[validate(range(min = 1, max = 100, message = "Limit must be 1-100"))]
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}
