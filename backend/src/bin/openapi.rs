use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Wedding API", version = "0.1.0"),
    paths(
        allmaptout_backend::health,
        allmaptout_backend::auth::validate_code,
        allmaptout_backend::auth::admin_login,
        allmaptout_backend::auth::logout,
        allmaptout_backend::auth::get_session,
        allmaptout_backend::events::list_events,
        allmaptout_backend::rsvp::get_rsvp_status,
        allmaptout_backend::rsvp::submit_rsvp,
    ),
    components(schemas(
        allmaptout_backend::Health,
        allmaptout_backend::schemas::ValidateCodeRequest,
        allmaptout_backend::schemas::ValidateCodeResponse,
        allmaptout_backend::schemas::AdminLoginRequest,
        allmaptout_backend::schemas::AdminLoginResponse,
        allmaptout_backend::schemas::SessionResponse,
        allmaptout_backend::schemas::EventResponse,
        allmaptout_backend::schemas::EventsListResponse,
        allmaptout_backend::schemas::AttendeeInput,
        allmaptout_backend::schemas::AttendeeResponse,
        allmaptout_backend::schemas::SubmitRsvpRequest,
        allmaptout_backend::schemas::RsvpResponse,
        allmaptout_backend::schemas::RsvpStatusResponse,
        allmaptout_backend::schemas::FieldError,
        allmaptout_backend::schemas::ValidationErrorResponse,
    ))
)]
struct ApiDoc;

fn main() {
    println!("{}", ApiDoc::openapi().to_json().unwrap());
}
