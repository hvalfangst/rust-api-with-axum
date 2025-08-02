pub mod router {
    use serde_json::{json, Value};
    use axum::{
        Router, http::StatusCode, Json, response::IntoResponse, extract::State, extract, middleware,
    };
    use crate::{
        common::{
            db::ConnectionPool,
            middleware::{require_writer, require_reader, require_editor, require_admin}
        },
        empires::{
            service::service::EmpiresTable as empiresTable,
            model::UpsertEmpire
        }
    };

    // - - - - - - - - - - - [ROUTES] - - - - - - - - - - -

    pub fn empires_route(shared_connection_pool: ConnectionPool) -> Router {
        // Create route groups with appropriate middleware
        let create_routes = Router::new()
            .route("/empires", axum::routing::post(create_empire_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_writer));
        
        let read_routes = Router::new()
            .route("/empires", axum::routing::get(get_all_empires_handler))
            .route("/empires/:empire_id", axum::routing::get(read_empire_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_reader));
        
        let update_routes = Router::new()
            .route("/empires/:empire_id", axum::routing::put(update_empire_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_editor));
        
        let delete_routes = Router::new()
            .route("/empires/:empire_id", axum::routing::delete(delete_empire_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_admin));

        // Merge all route groups
        Router::new()
            .merge(create_routes)
            .merge(read_routes)
            .merge(update_routes)
            .merge(delete_routes)
            .with_state(shared_connection_pool)
    }

    // - - - - - - - - - - - [HANDLERS] - - - - - - - - - - -

    pub async fn get_all_empires_handler(
        State(shared_state): State<ConnectionPool>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match empiresTable::new(connection).get_all() {
            Ok(empires) => Ok((StatusCode::OK, Json(empires))),
            Err(err) => {
                eprintln!("Error fetching all empires: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to fetch empires"}))))
            }
        }
    }

    pub async fn create_empire_handler(
        State(shared_state): State<ConnectionPool>,
        Json(upsert_empire): Json<UpsertEmpire>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match empiresTable::new(connection).create(upsert_empire) {
            Ok(new_empire) => Ok((StatusCode::CREATED, Json(new_empire))),
            Err(err) => {
                eprintln!("Error creating empire: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create empire"}))))
            }
        }
    }


    pub async fn read_empire_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match empiresTable::new(connection).get(empire_id) {
            Ok(empire) => {
                if let Some(empire) = empire {
                    Ok((StatusCode::OK, Json(empire)))
                } else {
                    Err((StatusCode::NOT_FOUND, Json(json!({"error": "Empire not found"}))))
                }
            },
            Err(err) => {
                eprintln!("Error reading empire: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read empire"}))))
            }
        }
    }

    pub async fn update_empire_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
        Json(upsert_empire): Json<UpsertEmpire>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match empiresTable::new(connection).update(empire_id, upsert_empire) {
            Ok(updated_empire) => Ok((StatusCode::OK, Json(updated_empire))),
            Err(diesel::result::Error::NotFound) => {
                Err((StatusCode::NOT_FOUND, Json(json!({"error": "Empire not found"}))))
            },
            Err(err) => {
                eprintln!("Error updating empire: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update empire"}))))
            }
        }
    }

    pub async fn delete_empire_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match empiresTable::new(connection).delete(empire_id) {
            Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
            Err(err) => {
                eprintln!("Error deleting empire: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to delete empire"}))))
            }
        }
    }
}
