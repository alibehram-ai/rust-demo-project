# Rust Invoice API

A RESTful API for invoice management built with Rust, featuring JWT authentication, PDF generation, and interactive API documentation.

## Features

- 🔐 **JWT Authentication** - Secure user registration and login
- 📄 **Invoice Management** - Full CRUD operations for invoices
- 📊 **PDF Generation** - Export invoices as PDF documents
- 📖 **Swagger UI** - Interactive API documentation
- 🗃️ **SQLite Database** - Lightweight persistent storage
- ✅ **Input Validation** - Request validation with detailed error messages

## Tech Stack

- **Framework**: [Actix-web](https://actix.rs/) - High-performance web framework
- **Database**: SQLite with [SQLx](https://github.com/launchbadge/sqlx)
- **Authentication**: JWT tokens with [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
- **Documentation**: [Utoipa](https://github.com/juhaku/utoipa) with Swagger UI
- **PDF**: [printpdf](https://github.com/fschutt/printpdf)

## Getting Started

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/YOUR_USERNAME/rust-invoice-api.git
   cd rust-invoice-api
   ```

2. Create environment file:
   ```bash
   cp .env.example .env
   ```

3. Configure your `.env` file:
   ```env
   DATABASE_URL=sqlite:invoices.db
   JWT_SECRET=your-super-secret-key-change-in-production
   JWT_EXPIRATION_HOURS=24
   SERVER_HOST=127.0.0.1
   SERVER_PORT=8080
   ```

4. Run the application:
   ```bash
   cargo run
   ```

The server will start at `http://127.0.0.1:8080`

## API Documentation

Once running, access the interactive Swagger UI at:
```
http://127.0.0.1:8080/swagger-ui/
```

## API Endpoints

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | Register a new user |
| POST | `/api/auth/login` | Login and get JWT token |

### Invoices (Protected)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/invoices` | List all invoices |
| POST | `/api/invoices` | Create a new invoice |
| GET | `/api/invoices/{id}` | Get invoice by ID |
| PUT | `/api/invoices/{id}` | Update an invoice |
| DELETE | `/api/invoices/{id}` | Delete an invoice |
| GET | `/api/invoices/{id}/pdf` | Download invoice as PDF |

## Invoice Status

Invoices can have one of the following statuses:
- `draft` - Initial state
- `sent` - Invoice sent to client
- `paid` - Payment received
- `overdue` - Past due date

## Example Usage

### Register a User
```bash
curl -X POST http://127.0.0.1:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "securepassword", "name": "John Doe"}'
```

### Login
```bash
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "securepassword"}'
```

### Create Invoice
```bash
curl -X POST http://127.0.0.1:8080/api/invoices \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "client_name": "Acme Corp",
    "client_email": "billing@acme.com",
    "issue_date": "2024-01-15",
    "due_date": "2024-02-15",
    "items": [
      {"description": "Web Development", "quantity": 10, "unit_price": 100, "amount": 1000}
    ],
    "tax_rate": 10
  }'
```

## Project Structure

```
src/
├── main.rs          # Application entry point
├── db.rs            # Database connection pool
├── error.rs         # Error handling
├── pdf.rs           # PDF generation
├── auth/
│   ├── mod.rs
│   ├── jwt.rs       # JWT token creation/validation
│   └── middleware.rs # Authentication middleware
├── handlers/
│   ├── mod.rs
│   ├── auth.rs      # Auth endpoints
│   └── invoices.rs  # Invoice endpoints
└── models/
    ├── mod.rs
    ├── user.rs      # User models
    └── invoice.rs   # Invoice models
```

## License

This project is licensed under the MIT License.
