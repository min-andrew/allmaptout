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
// Auth schemas
// ============================================================================

use uuid::Uuid;

/// Request to validate an invite code.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct ValidateCodeRequest {
    /// The invite code to validate.
    #[validate(length(min = 1, max = 50, message = "Code is required"))]
    pub code: String,
}

/// Response after validating an invite code.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidateCodeResponse {
    /// The type of session created: "guest" or "admin_pending".
    pub session_type: String,
    /// Guest name (only for guest sessions).
    pub guest_name: Option<String>,
}

/// Request for admin login (username/password).
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct AdminLoginRequest {
    /// Admin username.
    #[validate(length(min = 1, max = 100, message = "Username is required"))]
    pub username: String,
    /// Admin password.
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// Response after successful admin login.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminLoginResponse {
    /// The admin's username.
    pub username: String,
}

/// Response for current session info.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionResponse {
    /// Session type: "guest", "admin_pending", or "admin".
    pub session_type: String,
    /// Guest ID (if guest session).
    pub guest_id: Option<Uuid>,
    /// Guest name (if guest session).
    pub guest_name: Option<String>,
    /// Admin ID (if admin session).
    pub admin_id: Option<Uuid>,
    /// Admin username (if admin session).
    pub admin_username: Option<String>,
}

// ============================================================================
// Events schemas
// ============================================================================

/// Response for a single event.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventResponse {
    pub id: Uuid,
    pub name: String,
    pub event_type: String,
    pub event_date: String,
    pub event_time: String,
    pub location_name: String,
    pub location_address: String,
    pub description: Option<String>,
}

/// Response containing a list of events.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventsListResponse {
    pub events: Vec<EventResponse>,
}

// ============================================================================
// RSVP schemas
// ============================================================================

/// Input for a single attendee in an RSVP submission.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct AttendeeInput {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub is_attending: bool,
    pub meal_preference: Option<String>,
    #[validate(length(
        max = 500,
        message = "Dietary restrictions must be under 500 characters"
    ))]
    pub dietary_restrictions: Option<String>,
    pub is_primary: bool,
}

/// Request to submit or update an RSVP.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct SubmitRsvpRequest {
    #[validate(length(min = 1, message = "At least one attendee required"))]
    pub attendees: Vec<AttendeeInput>,
}

/// Response for a single attendee.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttendeeResponse {
    pub id: Uuid,
    pub name: String,
    pub is_attending: bool,
    pub meal_preference: Option<String>,
    pub dietary_restrictions: Option<String>,
    pub is_primary: bool,
}

/// Response for an RSVP with its attendees.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RsvpResponse {
    pub id: Uuid,
    pub guest_id: Uuid,
    pub responded_at: String,
    pub attendees: Vec<AttendeeResponse>,
}

/// Response for RSVP status check.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RsvpStatusResponse {
    pub has_responded: bool,
    pub party_size: i32,
    pub guest_name: String,
    pub rsvp: Option<RsvpResponse>,
}
