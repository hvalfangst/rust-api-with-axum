pub mod service {
    use diesel::{
        prelude::*,
        PgConnection,
        r2d2::{ConnectionManager, PooledConnection},
    };
    use crate::{
        locations::model::{Location, UpsertLocation},
        schema
    };

    type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

    pub struct LocationsTable {
        connection: PooledPg,
    }

    impl LocationsTable {
        pub fn new(connection: PooledPg) -> LocationsTable {
            LocationsTable { connection }
        }

        pub fn create(&mut self, upsert_location: UpsertLocation) -> Result<Location, diesel::result::Error> {
            use schema::locations;

            let new_location = diesel::insert_into(locations::table)
                .values((
                    locations::star_system.eq(&upsert_location.star_system),
                    locations::area.eq(&upsert_location.area),
                ))
                .get_result(&mut self.connection)
                .expect("Create location failed");

            Ok(new_location)
        }

        pub fn get_all(&mut self) -> Result<Vec<Location>, diesel::result::Error> {
            use schema::locations;

            let all_locations = locations::table
                .load::<Location>(&mut self.connection)?;

            Ok(all_locations)
        }

        pub fn get(&mut self, location_id: i32) -> Result<Option<Location>, diesel::result::Error> {
            use schema::locations;

            let location = locations::table.find(location_id)
                .get_result(&mut self.connection)
                .optional()?;

            Ok(location)
        }

        pub fn update(&mut self, location_id: i32, upsert_location: UpsertLocation) -> Result<Location, diesel::result::Error> {
            use schema::locations;

            // Check if the location exists before attempting to update
            let existing_location = locations::table.find(location_id)
                .get_result::<Location>(&mut self.connection);

            match existing_location {
                Ok(_) => {
                    let updated_location = diesel::update(locations::table.find(location_id))
                        .set((
                            locations::star_system.eq(&upsert_location.star_system),
                            locations::area.eq(&upsert_location.area),
                        ))
                        .get_result(&mut self.connection)
                        .expect("Update location failed");

                    Ok(updated_location)
                },
                Err(_) => Err(diesel::result::Error::NotFound)
            }
        }

        pub fn delete(&mut self, location_id: i32) -> Result<(), diesel::result::Error> {
            use schema::locations;

            // Check if the location exists before attempting to delete
            let existing_location = locations::table.find(location_id)
                .get_result::<Location>(&mut self.connection);

            match existing_location {
                Ok(_) => {
                    diesel::delete(locations::table.find(location_id))
                        .execute(&mut self.connection)?;
                    Ok(())
                },
                Err(_) => {
                    Err(diesel::result::Error::NotFound)
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            common::{
                db::create_shared_connection_pool,
                util::load_environment_variable
            },
            locations::{
                model::UpsertLocation,
                service::service::LocationsTable
            }
        };

        #[test]
        fn create_succeeds_on_valid_input() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);

            let new_location = UpsertLocation {
                star_system: "Test Star System".to_string(),
                area: "Test Area".to_string(),
            };

            let created_location = location_db.create(new_location.clone()).expect("Create location failed");

            assert_eq!(created_location.star_system, new_location.star_system);
            assert_eq!(created_location.area, new_location.area);
        }


        #[test]
        fn read_succeeds_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);

            let new_location = UpsertLocation {
                star_system: "Test Star System".to_string(),
                area: "Test Area".to_string(),
            };
            let created_location = location_db.create(new_location.clone()).expect("Create location failed");

            let retrieved_location = location_db.get(created_location.id).expect("Read location failed").unwrap();

            assert_eq!(retrieved_location.star_system, new_location.star_system);
            assert_eq!(retrieved_location.area, new_location.area);
        }

        #[test]
        fn read_returns_none_on_nonexistent_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);

            let retrieved_location = location_db.get(-666);  // Use a non-existent ID
            assert!(retrieved_location.is_ok());  // Expecting Ok(None)
            assert!(retrieved_location.unwrap().is_none());
        }


        #[test]
        fn update_succeeds_on_valid_input() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);

            let new_location = UpsertLocation {
                star_system: "Test Star System".to_string(),
                area: "Test Area".to_string(),
            };
            let created_location = location_db.create(new_location.clone()).expect("Create location failed");

            let updated_request = UpsertLocation {
                star_system: "Updated Star System".to_string(),
                area: "Updated Area".to_string(),
            };
            let updated_location = location_db.update(created_location.id, updated_request.clone()).expect("Update location failed");

            assert_eq!(updated_location.star_system, updated_request.star_system);
            assert_eq!(updated_location.area, updated_request.area);
        }

        #[test]
        fn update_fails_on_nonexistent_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);

            let request = UpsertLocation {
                star_system: "This test will fail".to_string(),
                area: "so write random skit here".to_string(),
            };

            let result = location_db.update(-1, request.clone());  // Use a non-existent ID
            assert!(result.is_err());  // Expecting an error as the ID is not present
        }


        #[test]
        fn delete_succeeds_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);

            let new_location = UpsertLocation {
                star_system: "Test Star System".to_string(),
                area: "Test Area".to_string(),
            };

            let created_location = location_db.create(new_location.clone()).expect("Create location failed");
            location_db.delete(created_location.id.clone()).expect("Delete location failed");
            let deleted_location = location_db.get(created_location.id).expect("Read location failed");
            assert!(deleted_location.is_none()); // Expecting lack of value as location has been deleted
        }

        #[test]
        fn delete_fails_on_nonexistent_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut location_db = LocationsTable::new(connection);

            let result = location_db.delete(-666);  // Use a non-existent ID
            assert!(result.is_err());  // Expecting an error as the ID is not present
        }
    }
}

