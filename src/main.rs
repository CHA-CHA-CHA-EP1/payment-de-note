use actix_web::{
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
    error::JsonPayloadError, http::StatusCode, web,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
    code: String,
}

#[derive(Debug)]
enum PaymentHttpError {
    BadRequest,
}

impl PaymentHttpError {
    fn as_response_data(&self) -> (StatusCode, ErrorResponse) {
        match self {
            Self::BadRequest => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    message: "Bad Request".to_string(),
                    code: "P001".to_string(),
                },
            ),
        }
    }
}

impl std::fmt::Display for PaymentHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (_, body) = self.as_response_data();
        write!(f, "{}", body.message)
    }
}

impl ResponseError for PaymentHttpError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let (status, body) = self.as_response_data();
        HttpResponse::build(status).json(body)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PaymentMethod {
    Bank,
    Truemoney,
}

#[derive(Debug, Deserialize, Validate)]
struct PaymentRequest {
    #[validate(range(min = 1.0, max = 1000000.0))]
    amount: f64,
    payment_method: PaymentMethod,
}

#[derive(Debug, Serialize)]
struct PayemntResponse {
    status: String,
    amount: f64,
}

async fn payment_handler(
    request: web::Json<PaymentRequest>,
) -> Result<HttpResponse, PaymentHttpError> {
    request
        .validate()
        .map_err(|_| PaymentHttpError::BadRequest)?;

    Ok(HttpResponse::Ok().body("I'm alive"))
}

fn custom_json_error_handler(_: JsonPayloadError, _req: &HttpRequest) -> Error {
    PaymentHttpError::BadRequest.into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let json_config = web::JsonConfig::default().error_handler(custom_json_error_handler);

    HttpServer::new(move || {
        App::new()
            .app_data(json_config.clone())
            .route("/health-check", web::get().to(health_check))
            .route("/payment/initiate", web::post().to(payment_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
