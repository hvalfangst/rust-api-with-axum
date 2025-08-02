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
        locations::{
            service::service::LocationsTable as locationsDB,
            model::UpsertLocation
        },
    };

    // - - - - - - - - - - - [ROUTES] - - - - - - - - - - -

    pub fn locations_route(shared_connection_pool: ConnectionPool) -> Router {
        // Create route groups with appropriate middleware
        let create_routes = Router::new()
            .route("/locations", axum::routing::post(create_location_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_writer));
        
        let read_routes = Router::new()
            .route("/locations", axum::routing::get(get_all_locations_handler))
            .route("/locations/:location_id", axum::routing::get(read_location_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_reader));
        
        let update_routes = Router::new()
            .route("/locations/:location_id", axum::routing::put(update_location_handler))
            .layer(middleware::from_fn_with_state(shared_connection_pool.clone(), require_editor));
        
        let delete_routes = Router::new()
            .route("/locations/:location_id", axum::routing::delete(delete_location_handler))
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

    pub async fn get_all_locations_handler(
        State(shared_state): State<ConnectionPool>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match locationsDB::new(connection).get_all() {
            Ok(locations) => Ok((StatusCode::OK, Json(locations))),
            Err(err) => {
                eprintln!("Error fetching all locations: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to fetch locations"}))))
            }
        }
    }

    pub async fn create_location_handler(
        State(shared_state): State<ConnectionPool>,
        Json(upsert_location): Json<UpsertLocation>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match locationsDB::new(connection).create(upsert_location) {
            Ok(new_location) => Ok((StatusCode::CREATED, Json(new_location))),
            Err(err) => {
                eprintln!("Error creating location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create location"}))))
            }
        }
    }

    pub async fn read_location_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id, ) = path.0;
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match locationsDB::new(connection).get(location_id) {
            Ok(location) => {
                if let Some(location) = location {
                    Ok((StatusCode::OK, Json(location)))
                } else {
                    Err((StatusCode::NOT_FOUND, Json(json!({"error": "Location not found"}))))
                }
            },
            Err(err) => {
                eprintln!("Error reading location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to read location"}))))
            }
        }
    }

    pub async fn update_location_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
        Json(upsert_location): Json<UpsertLocation>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id, ) = path.0;
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match locationsDB::new(connection).update(location_id, upsert_location) {
            Ok(updated_location) => Ok((StatusCode::OK, Json(updated_location))),
            Err(diesel::result::Error::NotFound) => {
                Err((StatusCode::NOT_FOUND, Json(json!({"error": "Location not found"}))))
            },
            Err(err) => {
                eprintln!("Error updating location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update location"}))))
            }
        }
    }

    pub async fn delete_location_handler(
        State(shared_state): State<ConnectionPool>,
        path: extract::Path<(i32, )>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let (location_id, ) = path.0;
        let connection = shared_state.pool.get()
            .expect("Failed to acquire connection from pool");

        match locationsDB::new(connection).delete(location_id) {
            Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
            Err(err) => {
                eprintln!("Error deleting location: {:?}", err);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to delete location"}))))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use axum::{
            body::Body,
            http::{Request, StatusCode}
        };
        use serde_json::json;
        use tower::ServiceExt;
        use crate::{
            common::{
                db::create_shared_connection_pool,
                util::load_environment_variable,
                security::hash_password
            },
            locations::{
                model::UpsertLocation,
                service::service::LocationsTable
            },
            users::{
                model::UpsertUser,
                service::service::UsersTable
            },
            locations_route
        };
        use crate::common::db::ConnectionPool;
        use crate::common::security::generate_token;
        use crate::users::model::UserRole;

        // Helper method utilized to create user with a specific role and return the associated bearer token in one line of code
        pub fn create_user_and_generate_token(connection_pool: ConnectionPool, email: &str, user_role: UserRole) -> Result<String, jsonwebtoken::errors::Error> {

            // Only email and role are mutable as password and fullname has no constraints
            let mut new_user = UpsertUser {
                email: email.to_string(),
                role: user_role.to_string(),
                password: "StålGardinerFunkerFjell53".to_string(),
                fullname: "Josef Stålhard".to_string()
            };

            // Hash the password
            hash_password(&mut new_user).expect("Hash failed");

            // Perform the user creation
            let create_user_result = {
                let connection = connection_pool.pool.get().expect("Failed to get connection");
                UsersTable::new(connection).create(new_user.clone())
            };

            // Generate the bearer token
            generate_token(&create_user_result.unwrap())
        }

        #[tokio::test]
        async fn post_locations_returns_201_for_authorized_user_with_write_access() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let service = locations_route(connection_pool.clone());

            // Create user with role WRITER and generate associated bearer token
            let bearer_token = create_user_and_generate_token(connection_pool, "stål.hard.russer@ugreit.ru", UserRole::WRITER);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri("/locations")
                .method("POST")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
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
        async fn post_locations_returns_401_for_unauthorized_user_without_write_access() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let service = locations_route(connection_pool.clone());

            // Create user with role READER and generate associated bearer token
            let bearer_token = create_user_and_generate_token(connection_pool, "myk.og.ekkel.russer@put.in", UserRole::READER);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri("/locations")
                .method("POST")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 401
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn put_locations_returns_200_for_authorized_user_with_edit_access() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);
            let service = locations_route(connection_pool.clone());

            // Create user with role WRITER and generate associated bearer token
            let bearer_token = create_user_and_generate_token(connection_pool, "dagfinnkuk@blåfjelletsvenner.no", UserRole::EDITOR);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = location_db.create(request_body.clone()).expect("Create location failed");

            // Assert equality
            assert_eq!(request_body.star_system, created_location.star_system);
            assert_eq!(request_body.area, created_location.area);

            let updated_request_body = UpsertLocation {
                star_system: "Kador".to_string(),
                area: "The Crimson Expanse".to_string(),
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("PUT")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
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
                "id": created_location.id,
                "area": updated_request_body.area,
                "star_system": updated_request_body.star_system
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn put_locations_returns_401_for_unauthorized_user_without_edit_access() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);
            let service = locations_route(connection_pool.clone());

            // Create user with role WRITER and generate associated bearer token
            let bearer_token = create_user_and_generate_token(connection_pool, "necromancer@gpf.no", UserRole::WRITER);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = location_db.create(request_body.clone()).expect("Create location failed");

            // Assert equality
            assert_eq!(request_body.star_system, created_location.star_system);
            assert_eq!(request_body.area, created_location.area);

            let updated_request_body = UpsertLocation {
                star_system: "Kador".to_string(),
                area: "The Crimson Expanse".to_string(),
            };

            // Create a request with the above data as payload
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("PUT")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
                .body(Body::from(serde_json::to_string(&updated_request_body).unwrap()))
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 401
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn get_locations_returns_200_for_authorized_user_with_read_access() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);
            let service = locations_route(connection_pool.clone());

            let bearer_token = create_user_and_generate_token(connection_pool, "duvetdet@gjerrigknark.no", UserRole::READER);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = location_db.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("GET")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
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
                "id": created_location.id,
                "area": request_body.area,
                "star_system": request_body.star_system
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn get_locations_returns_200_for_authorized_user_with_write_access() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);
            let service = locations_route(connection_pool.clone());

            let bearer_token = create_user_and_generate_token(connection_pool, "kokefaktura@woodworm.org", UserRole::WRITER);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = location_db.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("GET")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
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
                "id": created_location.id,
                "area": request_body.area,
                "star_system": request_body.star_system
            });

            // Assert equality
            assert_eq!(response_json, expected_response);
        }

        #[tokio::test]
        async fn get_locations_returns_401_for_unauthorized_user_without_read_access() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);
            let service = locations_route(connection_pool.clone());

            let bearer_token = create_user_and_generate_token(connection_pool, "igor.invalidus@bogdanov.fr", UserRole::INVALID);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = location_db.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("GET")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 401
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
        }

        #[tokio::test]
        async fn get_locations_returns_404_on_non_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let service = locations_route(connection_pool.clone());

            let bearer_token = create_user_and_generate_token(connection_pool, "birdman@ifi.uio.no", UserRole::READER);

            // Create a request with the aforementioned id
            let request = Request::builder()
                .uri(format!("/locations/{}", -666)) // Use a non-existent ID
                .method("GET")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
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
        async fn delete_locations_returns_204_for_authorized_user_with_admin_role() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);
            let service = locations_route(connection_pool.clone());

            let bearer_token = create_user_and_generate_token(connection_pool,"you.know.your.judo.well@succulentmail.gb", UserRole::ADMIN);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = location_db.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("DELETE")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 204
            assert_eq!(response.status(), StatusCode::NO_CONTENT);

            // Attempt to retrieve the deleted location
            let deleted_location_result = location_db.get(created_location.id);

            // Assert that the Result is Ok (no error)
            assert!(deleted_location_result.is_ok());

            // Extract the Option<Location> from the Ok variant
            let deleted_location = deleted_location_result.unwrap();

            // Assert that the deleted location is None (i.e., it doesn't exist)
            assert!(deleted_location.is_none());
        }

        #[tokio::test]
        async fn delete_locations_returns_401_for_unauthorized_user_without_admin_role() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 2);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);
            let service = locations_route(connection_pool.clone());

            let bearer_token = create_user_and_generate_token(connection_pool,"donttouchmys@p.succulentor.gb", UserRole::EDITOR);

            let request_body = UpsertLocation {
                star_system: "Fountain".to_string(),
                area: "The Serpent's Lair".to_string(),
            };

            // Create a new location with the above data
            let created_location = location_db.create(request_body.clone()).expect("Create location failed");

            // Create a request with the ID associated with our newly inserted row
            let request = Request::builder()
                .uri(format!("/locations/{}", created_location.id))
                .method("DELETE")
                .header("Authorization", format!("Bearer {}", bearer_token.unwrap())) // Add the bearer token
                .body(Body::empty())
                .unwrap();

            // Send the request through the service
            let response = service
                .oneshot(request)
                .await
                .unwrap();

            // Assert that the response status is 401
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }
}
