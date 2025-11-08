use std::sync::Arc;

use actix_web::{
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
    error::JsonPayloadError, http::StatusCode, web,
};
use mongodb::{
    Client, Collection,
    bson::{Document, doc},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
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

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Bank => write!(f, "bank"),
            PaymentMethod::Truemoney => write!(f, "truemoney"),
        }
    }
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
    mongodb_client: web::Data<Arc<Client>>,
) -> Result<HttpResponse, PaymentHttpError> {
    request
        .validate()
        .map_err(|_| PaymentHttpError::BadRequest)?;

    let body = request.into_inner();
    let database = mongodb_client.database("payment");
    let payment: Collection<Document> = database.collection("payment");

    let insert_doc = doc! {
        "transaction_id": Uuid::new_v4().to_string(),
        "amount": body.amount,
        "payment_method": body.payment_method.to_string(),
        "status": "pending"
    };

    let res = payment
        .insert_one(insert_doc)
        .await
        .map_err(|_| PaymentHttpError::BadRequest)?;

    println!("{:?}", res);

    Ok(HttpResponse::Ok().body("I'm alive"))
}

fn custom_json_error_handler(_: JsonPayloadError, _req: &HttpRequest) -> Error {
    PaymentHttpError::BadRequest.into()
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // database init
    let client = Arc::new(Client::with_uri_str("mongodb://127.0.0.1:27017").await?);

    let json_config = web::JsonConfig::default().error_handler(custom_json_error_handler);

    println!("server is starting http://127.0.0.1:8080");

    HttpServer::new(move || {
        let client = client.clone();
        let client_data = web::Data::new(client);

        App::new()
            .app_data(json_config.clone())
            .app_data(client_data)
            .route("/health-check", web::get().to(health_check))
            .route("/payment/initiate", web::post().to(payment_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
