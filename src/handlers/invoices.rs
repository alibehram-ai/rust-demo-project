use crate::auth::middleware::AuthUser;
use crate::db::DbPool;
use crate::error::AppError;
use crate::models::invoice::{
    CreateInvoiceRequest, Invoice, InvoiceItem, InvoiceResponse, UpdateInvoiceRequest,
};
use crate::models::user::User;
use crate::pdf::generate_pdf;
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

fn calculate_totals(items: &[InvoiceItem], tax_rate: f64) -> (f64, f64, f64) {
    let subtotal: f64 = items.iter().map(|i| i.amount).sum();
    let tax_amount = subtotal * (tax_rate / 100.0);
    let total = subtotal + tax_amount;
    (subtotal, tax_amount, total)
}

#[utoipa::path(
    get,
    path = "/api/invoices",
    responses(
        (status = 200, description = "List of invoices", body = Vec<InvoiceResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = [])),
    tag = "invoices"
)]
#[get("")]
pub async fn list_invoices(
    pool: web::Data<DbPool>,
    auth: AuthUser,
) -> Result<HttpResponse, AppError> {
    let invoices =
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE user_id = ? ORDER BY created_at DESC")
            .bind(&auth.0.sub)
            .fetch_all(pool.get_ref())
            .await?;

    let responses: Vec<InvoiceResponse> = invoices.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(responses))
}

#[utoipa::path(
    post,
    path = "/api/invoices",
    request_body = CreateInvoiceRequest,
    responses(
        (status = 201, description = "Invoice created", body = InvoiceResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = [])),
    tag = "invoices"
)]
#[post("")]
pub async fn create_invoice(
    pool: web::Data<DbPool>,
    auth: AuthUser,
    body: web::Json<CreateInvoiceRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let tax_rate = body.tax_rate.unwrap_or(0.0);
    let invoice_number = body
        .invoice_number
        .clone()
        .unwrap_or_else(|| format!("INV-{}", id[..8].to_uppercase()));

    let (subtotal, tax_amount, total) = calculate_totals(&body.items, tax_rate);
    let items_json =
        serde_json::to_string(&body.items).map_err(|e| AppError::InternalError(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO invoices
        (id, user_id, invoice_number, client_name, client_email, client_address,
         issue_date, due_date, status, items, notes, tax_rate, subtotal, tax_amount, total, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'draft', ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&auth.0.sub)
    .bind(&invoice_number)
    .bind(&body.client_name)
    .bind(&body.client_email)
    .bind(&body.client_address)
    .bind(&body.issue_date)
    .bind(&body.due_date)
    .bind(&items_json)
    .bind(&body.notes)
    .bind(tax_rate)
    .bind(subtotal)
    .bind(tax_amount)
    .bind(total)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await?;

    let invoice = sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.get_ref())
        .await?;

    Ok(HttpResponse::Created().json(InvoiceResponse::from(invoice)))
}

#[utoipa::path(
    get,
    path = "/api/invoices/{id}",
    params(("id" = String, Path, description = "Invoice ID")),
    responses(
        (status = 200, description = "Invoice found", body = InvoiceResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invoice not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "invoices"
)]
#[get("/{id}")]
pub async fn get_invoice(
    pool: web::Data<DbPool>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let invoice =
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = ? AND user_id = ?")
            .bind(&id)
            .bind(&auth.0.sub)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or(AppError::InvoiceNotFound)?;

    Ok(HttpResponse::Ok().json(InvoiceResponse::from(invoice)))
}

#[utoipa::path(
    put,
    path = "/api/invoices/{id}",
    params(("id" = String, Path, description = "Invoice ID")),
    request_body = UpdateInvoiceRequest,
    responses(
        (status = 200, description = "Invoice updated", body = InvoiceResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invoice not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "invoices"
)]
#[put("/{id}")]
pub async fn update_invoice(
    pool: web::Data<DbPool>,
    auth: AuthUser,
    path: web::Path<String>,
    body: web::Json<UpdateInvoiceRequest>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let invoice =
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = ? AND user_id = ?")
            .bind(&id)
            .bind(&auth.0.sub)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or(AppError::InvoiceNotFound)?;

    let client_name = body.client_name.clone().unwrap_or(invoice.client_name);
    let client_email = body.client_email.clone().unwrap_or(invoice.client_email);
    let client_address = body.client_address.clone().or(invoice.client_address);
    let issue_date = body.issue_date.clone().unwrap_or(invoice.issue_date);
    let due_date = body.due_date.clone().unwrap_or(invoice.due_date);
    let status = body.status.clone().unwrap_or(invoice.status);
    let notes = body.notes.clone().or(invoice.notes);
    let tax_rate = body.tax_rate.unwrap_or(invoice.tax_rate);

    let items: Vec<InvoiceItem> = if let Some(new_items) = &body.items {
        new_items.clone()
    } else {
        serde_json::from_str(&invoice.items).unwrap_or_default()
    };

    let (subtotal, tax_amount, total) = calculate_totals(&items, tax_rate);
    let items_json =
        serde_json::to_string(&items).map_err(|e| AppError::InternalError(e.to_string()))?;
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"UPDATE invoices SET
        client_name = ?, client_email = ?, client_address = ?,
        issue_date = ?, due_date = ?, status = ?, items = ?, notes = ?,
        tax_rate = ?, subtotal = ?, tax_amount = ?, total = ?, updated_at = ?
        WHERE id = ? AND user_id = ?"#,
    )
    .bind(&client_name)
    .bind(&client_email)
    .bind(&client_address)
    .bind(&issue_date)
    .bind(&due_date)
    .bind(&status)
    .bind(&items_json)
    .bind(&notes)
    .bind(tax_rate)
    .bind(subtotal)
    .bind(tax_amount)
    .bind(total)
    .bind(&now)
    .bind(&id)
    .bind(&auth.0.sub)
    .execute(pool.get_ref())
    .await?;

    let updated = sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(InvoiceResponse::from(updated)))
}

#[utoipa::path(
    delete,
    path = "/api/invoices/{id}",
    params(("id" = String, Path, description = "Invoice ID")),
    responses(
        (status = 204, description = "Invoice deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invoice not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "invoices"
)]
#[delete("/{id}")]
pub async fn delete_invoice(
    pool: web::Data<DbPool>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let result = sqlx::query("DELETE FROM invoices WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&auth.0.sub)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::InvoiceNotFound);
    }

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/api/invoices/{id}/pdf",
    params(("id" = String, Path, description = "Invoice ID")),
    responses(
        (status = 200, description = "Invoice PDF as binary"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invoice not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "invoices"
)]
#[get("/{id}/pdf")]
pub async fn get_invoice_pdf(
    pool: web::Data<DbPool>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let invoice =
        sqlx::query_as::<_, Invoice>("SELECT * FROM invoices WHERE id = ? AND user_id = ?")
            .bind(&id)
            .bind(&auth.0.sub)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or(AppError::InvoiceNotFound)?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&auth.0.sub)
        .fetch_one(pool.get_ref())
        .await?;

    let invoice_response = InvoiceResponse::from(invoice);
    let pdf_bytes = generate_pdf(&invoice_response, &user.name)?;

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .append_header((
            "Content-Disposition",
            format!(
                "attachment; filename=\"invoice-{}.pdf\"",
                invoice_response.invoice_number
            ),
        ))
        .body(pdf_bytes))
}
