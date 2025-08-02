use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response}
    ,
};

use crate::{
    common::{db::ConnectionPool, security::authorize_with_role},
    users::model::{User, UserRole},
};

// Extension to store authorized user in request
pub struct AuthorizedUser {
    pub user: Option<User>,
}

// Middleware function for requiring specific roles
pub async fn require_admin(
    State(pool): State<ConnectionPool>,
    req: Request<Body>,
    next: Next<Body>,
) -> Response {
    authorize_and_continue(req, next, pool, UserRole::ADMIN).await
}

pub async fn require_editor(
    State(pool): State<ConnectionPool>,
    req: Request<Body>,
    next: Next<Body>,
) -> Response {
    authorize_and_continue(req, next, pool, UserRole::EDITOR).await
}

pub async fn require_writer(
    State(pool): State<ConnectionPool>,
    req: Request<Body>,
    next: Next<Body>,
) -> Response {
    authorize_and_continue(req, next, pool, UserRole::WRITER).await
}

pub async fn require_reader(
    State(pool): State<ConnectionPool>,
    mut req: Request<Body>,
    next: Next<Body>,
) -> Response {
    authorize_and_continue(req, next, pool, UserRole::READER).await
}

// Helper function to authorize and continue
async fn authorize_and_continue(
    mut req: Request<Body>,
    next: Next<Body>,
    pool: ConnectionPool,
    required_role: UserRole,
) -> Response {
    let headers = req.headers();
    
    // Authorize user
    match authorize_with_role(headers, &pool, required_role).await {
        Ok(user) => {
            // Add user to request extensions
            req.extensions_mut().insert(AuthorizedUser { user });
            
            // Continue to the handler
            next.run(req).await
        },
        Err((status, json_error)) => {
            // Convert error to response
            (status, json_error).into_response()
        }
    }
}