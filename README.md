# Full-Stack Application with Rust Backend and WebAssembly Frontend

A complete web application written in Rust featuring an API built with Axum and frontend pages created with Leptos WASM. Endpoints are protected by
JWT authentication, and the application supports role-based access control for managing users, star system locations, and empires in a fictional MMO universe inspired by EVE Online.

## Architecture

### Backend
- **Framework**: Axum web framework with Hyper
- **Database**: PostgreSQL with Diesel ORM
- **Authentication**: JWT tokens with bcrypt password hashing
- **Authorization**: Role-based access control (READER, WRITER, EDITOR, ADMIN)
- **API**: RESTful endpoints with proper HTTP status codes

### Frontend
- **Framework**: Leptos (Rust → WebAssembly)
- **Styling**: CSS with responsive design
- **State Management**: Leptos signals and effects
- **HTTP Client**: gloo-net for API communication
- **Routing**: Client-side routing with leptos-router

## Requirements

* x86-64 architecture
* Linux/Unix environment
* [Rust](https://www.rust-lang.org/tools/install) (latest stable)
* [Docker](https://www.docker.com/products/docker-desktop/) and Docker Compose
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) for frontend builds



## Development Setup

### Backend Development

Start the backend server with automatic database setup:

```bash
./serve_backend.sh
```

This script will:
- Launch PostgreSQL containers for development and testing
- Install Diesel CLI for database migrations
- Run database migrations automatically
- Compile and start the Axum server on `http://localhost:3000`

### Frontend Development

In a separate terminal, start the frontend development server:

```bash
./serve_frontend.sh
```

This script will:
- Build the WebAssembly package using wasm-pack
- Start a Python HTTP server on `http://localhost:8000`
- Serve the frontend with hot-reload capabilities

## API Reference

The API domain is inspired by the MMO game 'EVE Online', providing [endpoints](backend/src/empires/router.rs) to manage users, star system locations, and empires within the game's universe.

### Authentication Endpoints

| Method | Endpoint         | Description          | Auth Required |
|--------|------------------|----------------------|---------------|
| POST   | `/users/login`   | User authentication  | No            |
| POST   | `/users`         | User registration    | No            |

### CRUD Endpoints

| Resource   | Method | Endpoint              | Description         | Required Role |
|------------|--------|-----------------------|---------------------|---------------|
| Users      | GET    | `/users`             | List all users      | READER        |
| Users      | GET    | `/users/:id`         | Get user by ID      | READER        |
| Users      | PUT    | `/users/:id`         | Update user         | EDITOR        |
| Users      | DELETE | `/users/:id`         | Delete user         | ADMIN         |
| Locations  | GET    | `/locations`         | List all locations  | READER        |
| Locations  | POST   | `/locations`         | Create location     | WRITER        |
| Locations  | GET    | `/locations/:id`     | Get location by ID  | READER        |
| Locations  | PUT    | `/locations/:id`     | Update location     | EDITOR        |
| Locations  | DELETE | `/locations/:id`     | Delete location     | ADMIN         |
| Empires    | GET    | `/empires`           | List all empires    | READER        |
| Empires    | POST   | `/empires`           | Create empire       | WRITER        |
| Empires    | GET    | `/empires/:id`       | Get empire by ID    | READER        |
| Empires    | PUT    | `/empires/:id`       | Update empire       | EDITOR        |
| Empires    | DELETE | `/empires/:id`       | Delete empire       | ADMIN         |

## User Roles

The system implements a hierarchical role-based access control:

- **READER**: Can view resources (locations, empires, users)
- **WRITER**: READER permissions + can create new resources
- **EDITOR**: WRITER permissions + can modify existing resources
- **ADMIN**: EDITOR permissions + can delete resources and manage users

Higher roles inherit all permissions from lower roles.

## Database Schema

The application uses PostgreSQL with the following main entities:

- **users**: User accounts with authentication and role information
- **locations**: Star system and area data
- **empires**: Empire information with location associations

Database [migrations](backend/migrations) are managed through Diesel and executed automatically during development setup.