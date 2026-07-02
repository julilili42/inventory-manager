# Inventory Manager

A web-based inventory management tool written in Rust and React. It links
articles, customers, and orders, tracks stock levels, imports records, and
generates PDF invoices through a REST API and browser dashboard.

![Inventory Manager article and customer dashboard](https://github.com/user-attachments/assets/71478ac1-54ac-4c2a-bbb3-d66764ae643e)

![Inventory Manager order workflow](docs/orders.jpg)

## Get Started

Start the backend and frontend in two terminals:

```sh
cd backend
cargo run
```

```sh
cd frontend
npm install
npm run dev
```

Open the dashboard at `http://localhost:5173`. The API listens on
`http://127.0.0.1:8080/api`; Swagger UI is available at
`http://127.0.0.1:8080/api/docs`. The backend creates `data/database.db`
automatically.

## Why this project exists

This project explores full-stack Rust services, structured inventory workflows,
SQLite-backed persistence, API documentation, and PDF invoice generation for
small business-style operations.

## Features

- Article and customer management
- CSV import for inventory and customer records
- Order processing with automated PDF invoice generation
- Stock-level tracking
- REST API with Swagger UI documentation
- React/Vite dashboard for articles, customers, and orders

## Tech Stack

- Rust, Tokio, Axum
- SQLite, rusqlite, r2d2
- utoipa, Swagger UI
- headless_chrome, printpdf
- React, TypeScript, Vite
- Tailwind CSS, shadcn/ui
