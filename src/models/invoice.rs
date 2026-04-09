use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct InvoiceItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Draft,
    Sent,
    Paid,
    Overdue,
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceStatus::Draft => write!(f, "draft"),
            InvoiceStatus::Sent => write!(f, "sent"),
            InvoiceStatus::Paid => write!(f, "paid"),
            InvoiceStatus::Overdue => write!(f, "overdue"),
        }
    }
}

impl From<String> for InvoiceStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "sent" => InvoiceStatus::Sent,
            "paid" => InvoiceStatus::Paid,
            "overdue" => InvoiceStatus::Overdue,
            _ => InvoiceStatus::Draft,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Invoice {
    pub id: String,
    pub user_id: String,
    pub invoice_number: String,
    pub client_name: String,
    pub client_email: String,
    pub client_address: Option<String>,
    pub issue_date: String,
    pub due_date: String,
    pub status: String,
    pub items: String,
    pub notes: Option<String>,
    pub tax_rate: f64,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InvoiceResponse {
    pub id: String,
    pub user_id: String,
    pub invoice_number: String,
    pub client_name: String,
    pub client_email: String,
    pub client_address: Option<String>,
    pub issue_date: String,
    pub due_date: String,
    pub status: InvoiceStatus,
    pub items: Vec<InvoiceItem>,
    pub notes: Option<String>,
    pub tax_rate: f64,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Invoice> for InvoiceResponse {
    fn from(inv: Invoice) -> Self {
        let items: Vec<InvoiceItem> = serde_json::from_str(&inv.items).unwrap_or_default();
        InvoiceResponse {
            id: inv.id,
            user_id: inv.user_id,
            invoice_number: inv.invoice_number,
            client_name: inv.client_name,
            client_email: inv.client_email,
            client_address: inv.client_address,
            issue_date: inv.issue_date,
            due_date: inv.due_date,
            status: InvoiceStatus::from(inv.status),
            items,
            notes: inv.notes,
            tax_rate: inv.tax_rate,
            subtotal: inv.subtotal,
            tax_amount: inv.tax_amount,
            total: inv.total,
            created_at: inv.created_at,
            updated_at: inv.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateInvoiceRequest {
    pub invoice_number: Option<String>,
    #[validate(length(min = 1))]
    pub client_name: String,
    #[validate(email)]
    pub client_email: String,
    pub client_address: Option<String>,
    pub issue_date: String,
    pub due_date: String,
    pub items: Vec<InvoiceItem>,
    pub notes: Option<String>,
    pub tax_rate: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateInvoiceRequest {
    pub client_name: Option<String>,
    pub client_email: Option<String>,
    pub client_address: Option<String>,
    pub issue_date: Option<String>,
    pub due_date: Option<String>,
    pub status: Option<String>,
    pub items: Option<Vec<InvoiceItem>>,
    pub notes: Option<String>,
    pub tax_rate: Option<f64>,
}
