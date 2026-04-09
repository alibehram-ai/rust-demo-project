mod auth;
mod db;
mod error;
mod handlers;
mod models;
mod pdf;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::register,
        handlers::auth::login,
        handlers::invoices::list_invoices,
        handlers::invoices::create_invoice,
        handlers::invoices::get_invoice,
        handlers::invoices::update_invoice,
        handlers::invoices::delete_invoice,
        handlers::invoices::get_invoice_pdf,
    ),
    components(schemas(
        models::user::RegisterRequest,
        models::user::LoginRequest,
        models::user::AuthResponse,
        models::user::UserResponse,
        models::invoice::InvoiceResponse,
        models::invoice::CreateInvoiceRequest,
        models::invoice::UpdateInvoiceRequest,
        models::invoice::InvoiceItem,
        models::invoice::InvoiceStatus,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "invoices", description = "Invoice management endpoints"),
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:invoices.db".to_string());
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Starting server at http://{}:{}", host, port);
    tracing::info!(
        "Swagger UI available at http://{}:{}/swagger-ui/",
        host,
        port
    );

    let pool = web::Data::new(pool);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(pool.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .service(
                web::scope("/api/auth")
                    .service(handlers::auth::register)
                    .service(handlers::auth::login),
            )
            .service(
                web::scope("/api/invoices")
                    .service(handlers::invoices::list_invoices)
                    .service(handlers::invoices::create_invoice)
                    .service(handlers::invoices::get_invoice)
                    .service(handlers::invoices::update_invoice)
                    .service(handlers::invoices::delete_invoice)
                    .service(handlers::invoices::get_invoice_pdf),
            )
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
