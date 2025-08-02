pub mod router {
    use serde_json::{json, Value};
    use bcrypt::verify;
    use axum::{extract, extract::State, http::StatusCode, Json, response::IntoResponse, Router, middleware};
    use crate::{
        common::{
            db::ConnectionPool,
            security::{hash_password, generate_token},
            middleware::{require_reader, require_editor, require_admin}
        },
        users::{
            service::service::UsersTable,
            model::{
                UpsertUser,
                LoginUser,
            },
        },
    };

    // - - - - - - - - - - - [ROUTES] - - - - - - - - - - -

    pub fn users_route(shared_connection_pool: ConnectionPool) -> Router {
        // Public routes (no authentication required)
        let public_routes = Router::new()
            .route("/users", axum::routing::post(create_user_handler))  // Registration
            .route("/users/login", axum::routing::post(login_user_handler));  // Login
        
        // Protected routes requiring authentication
        let read_routes = Router::new()
            .route("/users", axum::routing::get(list_users_handler))
            .route("/users/:user_id", axum::routing::get(get_user_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_reader));
        
        let update_routes = Router::new()
            .route("/users/:user_id", axum::routing::put(update_user_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_editor));
        
        let delete_routes = Router::new()
            .route("/users/:user_id", axum::routing::delete(delete_user_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_admin));

        // Merge all route groups
        Router::new()
            .merge(public_routes)
            .merge(read_routes)
            .merge(update_routes)
            .merge(delete_routes)
            .with_state(shared_connection_pool)
    }


    // - - - - - - - - - - - [HANDLERS] - - - - - - - - - - -

    pub async fn list_users_handler(
        State(shared_state): State<ConnectionPool>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = UsersTable::new(connection);

        match users.list() {
            Ok(users_list) => Ok((StatusCode::OK, Json(users_list))),
            Err(err) => {
                eprintln!("Error listing users: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to list users"}))))
            }
        }
    }

    pub async fn create_user_handler(
        State(shared_state): State<ConnectionPool>,
        Json(mut body): Json<UpsertUser>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        if !validate_email(&body) {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error": "Invalid input for field 'email'"}))));
        }

        hash_password(&mut body)?;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match UsersTable::new(connection).create(body) {
            Ok(created_user) => Ok((StatusCode::CREATED, Json(created_user))),
            Err(err) => {
                eprintln!("Create user failed: {:?}", err);
                Err((StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error": "Failed to create user"}))))
            }
        }
    }

    fn validate_email(body: &UpsertUser) -> bool {
        body.is_valid_email()
    }

    pub async fn get_user_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32,)>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (user_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = UsersTable::new(connection);

        match users.get(user_id) {
            Ok(user) => {
                if let Some(user) = user {
                    Ok((StatusCode::OK, Json(user)))
                } else {
                    Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"}))))
                }
            },
            Err(err) => {
                eprintln!("Error reading user: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read user"}))))
            }
        }
    }

    pub async fn update_user_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32,)>,
        Json(update_user): Json<UpsertUser>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (user_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = UsersTable::new(connection);

        match users.update(user_id, update_user) {
            Ok(updated_user) => Ok((StatusCode::OK, Json(updated_user))),
            Err(diesel::result::Error::NotFound) => {
                Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"}))))
            },
            Err(err) => {
                eprintln!("Error updating user: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update user"}))))
            }
        }
    }

    pub async fn delete_user_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32,)>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (user_id,) = path.0;

        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        let mut users = UsersTable::new(connection);

        match users.delete(user_id) {
            Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
            Err(err) => {
                eprintln!("Error deleting user: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to delete user"}))))
            }
        }
    }

    pub async fn login_user_handler(
        State(shared_state): State<ConnectionPool>,
        Json(body): Json<LoginUser>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match UsersTable::new(connection).get_by_email(body.email.clone()) {
            Ok(Some(user)) if body.email == user.email => {
                return if verify(&body.password, &user.password).unwrap_or(false) {
                    if let Some(token) = generate_token(&user).ok() {
                        Ok((StatusCode::OK, Json(token)))
                    } else {
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to generate token"}))))
                    }
                } else {
                    Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "Wrong password"}))))
                }
            }
            Ok(Some(_)) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"})))),
            Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "User not found"})))),
            Err(err) => {
                eprintln!("Error reading user: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read user"}))))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use serde_json::json;
        use tower::ServiceExt;
        use crate::{create_shared_connection_pool, load_environment_variable, users_route};
        use crate::users::model::UpsertUser;
        use crate::users::service::service::UsersTable;

        #[tokio::test]
        async fn post_users_returns_201_on_valid_data() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let service = users_route(connection_pool);

            let request_body = UpsertUser {
                email: "valid@email.com".to_string(),
                password: "Big100".to_string(),
                fullname: "Kenneth Molasses".to_string(),
                role: "READER".to_string()
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri("/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 201
            assert_eq!(response.status(), StatusCode::CREATED);
        }

        #[tokio::test]
        async fn post_users_returns_422_on_invalid_email() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let service = users_route(connection_pool);

            let request_body = UpsertUser {
                email: "eg-klare-meg".to_string(),
                password: "Big100".to_string(),
                fullname: "Kenneth Molasses".to_string(),
                role: "READER".to_string()
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri("/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 422
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        }

        #[tokio::test]
        async fn put_users_returns_200_on_valid_data() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);
            let service = users_route(connection_pool);

            // Data
            let request_body = UpsertUser {
                email: "ernst@snowmail.com".to_string(),
                password: "feltedsnowmen".to_string(),
                fullname: "Ernst van Schnee".to_string(),
                role: "READER".to_string()
            };

            // Create a new location with the above data
            let created_user = user_db.create(request_body.clone()).expect("Create location failed");

            // Assert equality
            assert_eq!(request_body.email, created_user.email);
            assert_eq!(request_body.password, created_user.password);
            assert_eq!(request_body.fullname, created_user.fullname);
            assert_eq!(request_body.role, created_user.role);

            // Data
            let updated_request_body = UpsertUser {
                email: "ernst@snowmail.com".to_string(),
                password: "feltseng?".to_string(),
                fullname: "Ernst van Schnee".to_string(),
                role: "READER".to_string()
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri(format!("/users/{}", created_user.id))
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&updated_request_body).unwrap()))
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 200
            assert_eq!(response.status(), StatusCode::OK);

            // Extract body from response
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Construct JSON consisting of expected payload
            let expected_response = json!({
                "id": created_user.id,
                "email": updated_request_body.email,
                "password": updated_request_body.password,
                "fullname": updated_request_body.fullname,
                "role": updated_request_body.role
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn get_users_returns_200_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);
            let service = users_route(connection_pool);

            let request_body = UpsertUser {
                email: "glossy@ringdue.no".to_string(),
                password: "LillePostBudMin".to_string(),
                fullname: "Glossy Garnished".to_string(),
                role: "READER".to_string()
            };

            // Create a new location with the above data
            let created_user = user_db.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/users/{}", created_user.id))
                .method("GET")
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 200
            assert_eq!(response.status(), StatusCode::OK);

            // Extract body from response
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Construct JSON consisting of expected payload
            let expected_response = json!({
                "id": created_user.id,
                "email": request_body.email,
                "password": request_body.password,
                "fullname": request_body.fullname,
                "role": request_body.role
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn get_users_returns_404_on_non_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let service = users_route(connection_pool);

            // Create a request with the aforementioned id
            let request = Request::builder()
                .uri(format!("/users/{}", -666)) // Use a non-existent ID
                .method("GET")
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 404 as there are no locations associated with the id
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn delete_users_returns_204() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);
            let service = users_route(connection_pool);

            let request_body = UpsertUser {
                email: "josek@ifi.uio.no".to_string(),
                password: "TurboPascalLife".to_string(),
                fullname: "Jose Kernelio".to_string(),
                role: "READER".to_string()
            };

            // Create a new user with the above data
            let created_user = user_db.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/users/{}", created_user.id))
                .method("DELETE")
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 204
            assert_eq!(response.status(), StatusCode::NO_CONTENT);

            // Attempt to retrieve the deleted user
            let deleted_user_result = user_db.get(created_user.id);

            // Assert that the Result is Ok (no error)
            assert!(deleted_user_result.is_ok());

            // Extract the Option<User> from the Ok variant
            let deleted_user = deleted_user_result.unwrap();

            // Assert that the deleted user is None (i.e., it doesn't exist)
            assert!(deleted_user.is_none());
        }

        #[tokio::test]
        async fn get_users_returns_200_with_users_list() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let service = users_route(connection_pool);

            // Create a request to list all users
            let request = Request::builder()
                .uri("/users")
                .method("GET")
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 200
            assert_eq!(response.status(), StatusCode::OK);

            // Extract body from response
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            // Assert that the response is an array
            assert!(response_json.is_array());
        }
    }
}
