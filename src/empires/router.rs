pub mod router {
    use serde_json::{json, Value};
    use axum::{
        Router, http::StatusCode, Json, response::IntoResponse, extract::State, extract,
    };
    use http::HeaderMap;
    use crate::{
        common::db::ConnectionPool,
        empires::{
            service::service::EmpiresTable as empiresTable,
            model::UpsertEmpire
        },
        users::model::UserRole,
        common::security::authorize_with_role
    };

    // - - - - - - - - - - - [ROUTES] - - - - - - - - - - -

    pub fn empires_route(shared_connection_pool: ConnectionPool) -> Router {
        Router::new()
            .route("/empires", axum::routing::post(create_empire_handler))
            .route("/empires/:empire_id", axum::routing::get(read_empire_handler))
            .route("/empires/:empire_id", axum::routing::put(update_empire_handler))
            .route("/empires/:empire_id", axum::routing::delete(delete_empire_handler))
            .with_state(shared_connection_pool)
    }

    // - - - - - - - - - - - [HANDLERS] - - - - - - - - - - -

    pub async fn create_empire_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        Json(upsert_empire): Json<UpsertEmpire>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

        // Authorize user with WRITER role or higher
        match authorize_with_role(&headers, &shared_state, UserRole::WRITER).await {
            Ok(_authorized_user) => {
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
            Err(err) => Err(err)
        }
    }


    pub async fn read_empire_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;

        // Authorize user with READER role or higher
        match authorize_with_role(&headers, &shared_state, UserRole::READER).await {
            Ok(_authorized_user) => {
                let connection = shared_state.pool.get()
                    .expect("Failed to acquire connection from pool");

                match empiresTable::new(connection).get(empire_id) {
                    Ok(empire) => {
                        if let Some(empire) = empire {
                            Ok((StatusCode::OK, Json(empire)))
                        } else {
                            Err((StatusCode::NOT_FOUND, Json(json!({"error": "empire not found"}))))
                        }
                    },
                    Err(err) => {
                        eprintln!("Error reading empire: {:?}", err);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read empire"}))))
                    }
                }
            }
            Err(err) => Err(err)
        }
    }

    pub async fn update_empire_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
        Json(upsert_empire): Json<UpsertEmpire>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;

        // Authorize user with EDITOR role or higher
        match authorize_with_role(&headers, &shared_state, UserRole::EDITOR).await {
            Ok(_authorized_user) => {
                let connection = shared_state.pool.get()
                    .expect("Failed to acquire connection from pool");

                match empiresTable::new(connection).update(empire_id, upsert_empire) {
                    Ok(updated_empire) => Ok((StatusCode::OK, Json(updated_empire))),
                    Err(diesel::result::Error::NotFound) => {
                        Err((StatusCode::NOT_FOUND, Json(json!({"error": "empire not found"}))))
                    },
                    Err(err) => {
                        eprintln!("Error updating empire: {:?}", err);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update empire"}))))
                    }
                }
            }
            Err(err) => Err(err)
        }
    }

    pub async fn delete_empire_handler(
        headers: HeaderMap,
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (empire_id, ) = path.0;

        // Authorize user with ADMIN role
        match authorize_with_role(&headers, &shared_state, UserRole::ADMIN).await {
            Ok(_authorized_user) => {
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
            Err(err) => Err(err)
        }
    }
}
