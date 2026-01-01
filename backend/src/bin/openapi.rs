use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Wedding API", version = "0.1.0"),
    paths(allmaptout_backend::health),
    components(schemas(allmaptout_backend::Health))
)]
struct ApiDoc;

fn main() {
    println!("{}", ApiDoc::openapi().to_json().unwrap());
}
