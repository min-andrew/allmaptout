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
        allmaptout_backend::admin::get_dashboard_stats,
        allmaptout_backend::admin::list_guests,
        allmaptout_backend::admin::create_guest,
        allmaptout_backend::admin::update_guest,
        allmaptout_backend::admin::delete_guest,
        allmaptout_backend::admin::regenerate_code,
        allmaptout_backend::admin::list_admin_events,
        allmaptout_backend::admin::create_event,
        allmaptout_backend::admin::update_event,
        allmaptout_backend::admin::delete_event,
        allmaptout_backend::admin::change_password,
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
        allmaptout_backend::schemas::CreateGuestRequest,
        allmaptout_backend::schemas::CreateGuestResponse,
        allmaptout_backend::schemas::UpdateGuestRequest,
        allmaptout_backend::schemas::AdminGuestResponse,
        allmaptout_backend::schemas::AdminGuestsListResponse,
        allmaptout_backend::schemas::AdminRsvpSummary,
        allmaptout_backend::schemas::GenerateCodeResponse,
        allmaptout_backend::schemas::RecentRsvp,
        allmaptout_backend::schemas::DashboardStatsResponse,
        allmaptout_backend::schemas::CreateEventRequest,
        allmaptout_backend::schemas::UpdateEventRequest,
        allmaptout_backend::schemas::AdminEventResponse,
        allmaptout_backend::schemas::AdminEventsListResponse,
        allmaptout_backend::schemas::ChangePasswordRequest,
        allmaptout_backend::schemas::ChangePasswordResponse,
    ))
)]
struct ApiDoc;

fn main() {
    println!("{}", ApiDoc::openapi().to_json().unwrap());
}
