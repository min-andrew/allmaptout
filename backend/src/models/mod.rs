use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Guest {
    pub id: Uuid,
    pub name: String,
    pub party_size: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Admin {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodeType {
    Guest,
    Admin,
}

impl CodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeType::Guest => "guest",
            CodeType::Admin => "admin",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "guest" => Some(CodeType::Guest),
            "admin" => Some(CodeType::Admin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InviteCode {
    pub id: Uuid,
    pub code: String,
    pub code_type: String,
    pub guest_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl InviteCode {
    pub fn get_code_type(&self) -> Option<CodeType> {
        CodeType::parse(&self.code_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionType {
    Guest,
    AdminPending,
    Admin,
}

impl SessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionType::Guest => "guest",
            SessionType::AdminPending => "admin_pending",
            SessionType::Admin => "admin",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "guest" => Some(SessionType::Guest),
            "admin_pending" => Some(SessionType::AdminPending),
            "admin" => Some(SessionType::Admin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub token: String,
    pub session_type: String,
    pub guest_id: Option<Uuid>,
    pub admin_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn get_session_type(&self) -> Option<SessionType> {
        SessionType::parse(&self.session_type)
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}
