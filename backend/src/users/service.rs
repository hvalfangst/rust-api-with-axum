pub mod service {

    use diesel::{
        prelude::*,
        PgConnection,
        result::Error,
        r2d2::{ConnectionManager, PooledConnection},
    };

    use crate::{
        users::model::{User, UpsertUser},
        schema,
        common::error::CustomError
    };

    type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

    pub struct UsersTable {
        connection: PooledPg,
    }

    impl UsersTable {
        pub fn new(connection: PooledPg) -> UsersTable {
            UsersTable { connection }
        }

        pub fn create(&mut self, create_user: UpsertUser) -> Result<User, CustomError> {
            use schema::users;

            diesel::insert_into(users::table)
                .values((
                    users::email.eq(&create_user.email),
                    users::password.eq(&create_user.password),
                    users::fullname.eq(&create_user.fullname),
                    users::role.eq(&create_user.role),
                ))
                .get_result::<User>(&mut self.connection)
                .map_err(|err| {
                    CustomError::from_diesel_err(err, "while creating user")
                })
        }

        pub fn get(&mut self, user_id: i32) -> Result<Option<User>, diesel::result::Error> {
            use schema::users;

            let user = users::table.find(user_id)
                .get_result(&mut self.connection)
                .optional()?;

            Ok(user)
        }

        pub fn get_by_email(&mut self, email: String) -> Result<Option<User>, Error> {
            use schema::users;

            let user = users::table
                .filter(users::email.eq(email))
                .get_result(&mut self.connection)
                .optional()?;

            Ok(user)
        }

        pub fn list(&mut self) -> Result<Vec<User>, Error> {
            use schema::users;

            let all_users = users::table
                .load::<User>(&mut self.connection)?;

            Ok(all_users)
        }

        pub fn update(&mut self, user_id: i32, update_user: UpsertUser) -> Result<User, Error> {
            use schema::users;

            // Check if the user exists before attempting to update
            let existing_user = users::table.find(user_id)
                .get_result::<User>(&mut self.connection);

            match existing_user {
                Ok(_) => {
                    let updated_user = diesel::update(users::table.find(user_id))
                        .set((
                            users::email.eq(&update_user.email),
                            users::password.eq(&update_user.password),
                            users::fullname.eq(&update_user.fullname),
                            users::role.eq(&update_user.role),
                        ))
                        .get_result(&mut self.connection)
                        .expect("Update user failed");

                    Ok(updated_user)
                },
                Err(_) => Err(Error::NotFound)
            }
        }


        pub fn delete(&mut self, user_id: i32) -> Result<(), diesel::result::Error> {
            use schema::users;

            // Check if the location exists before attempting to delete
            let existing_location = users::table.find(user_id)
                .get_result::<User>(&mut self.connection);

            match existing_location {
                Ok(_) => {
                    diesel::delete(users::table.find(user_id))
                        .execute(&mut self.connection)?;
                    Ok(())
                },
                Err(_) => {
                    Err(Error::NotFound)
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            common::{
                db::create_shared_connection_pool,
                util::load_environment_variable,
                error::ErrorType
            },
            users::{
                model::UpsertUser,
                service::service::UsersTable
            }
        };

        #[test]
        fn create_succeeds_on_valid_input() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);

            let new_user = UpsertUser {
                email: "obelisksx@ifi.uio.no".to_string(),
                password: "EatSleepRepeat".to_string(),
                fullname: "Obelix fra IFI".to_string(),
                role: "READER".to_string()
            };

            let created_user = user_db.create(new_user.clone()).expect("Create user failed");

            assert_eq!(created_user.email, new_user.email);
            assert_eq!(created_user.password, new_user.password);
            assert_eq!(created_user.fullname, new_user.fullname);
            assert_eq!(created_user.role, new_user.role);
        }

        #[test]
        fn create_fails_on_duplicate_mail() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);

            let dupe_user = UpsertUser {
                email: "duperdave@blizzard.com".to_string(),
                password: "GullDagger69".to_string(),
                fullname: "Mule Duperino".to_string(),
                role: "READER".to_string()
            };

            // First create should succeed
            let first_create_result = user_db.create(dupe_user.clone());
            assert!(first_create_result.is_ok()); // Check if it's Ok

            let first_create = match first_create_result {
                Ok(user) => user, // Extract the User from the Result
                Err(err) => panic!("First create failed with error: {:?}", err),
            };

            assert_eq!(first_create.email, dupe_user.email);
            assert_eq!(first_create.password, dupe_user.password);
            assert_eq!(first_create.fullname, dupe_user.fullname);
            assert_eq!(first_create.role, dupe_user.role);

            // Second create should fail due to violation of unique constraint on 'email'
            let second_create = user_db.create(dupe_user.clone());
            assert!(second_create.is_err());

            // Check the specific error type (BadRequest) and message
            if let Err(err) = second_create {
                assert_eq!(err.err_type, ErrorType::UniqueViolation);
                assert_eq!(
                    err.message,
                    "while creating user: duplicate key value violates unique constraint \"users_email_key\""
                );
            } else {
                panic!("Expected an error, but got Ok");
            }
        }

        #[test]
        fn read_succeeds_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);

            let new_user = UpsertUser {
                email: "kokemakken@tremakk.no".to_string(),
                password: "huuuuuman".to_string(),
                fullname: "Woodwormius".to_string(),
                role: "READER".to_string()
            };

            let created_user = user_db.create(new_user.clone()).expect("Create user failed");
            let retrieved_user = user_db.get(created_user.id).expect("Read user failed").unwrap();

            assert_eq!(retrieved_user.email, new_user.email);
            assert_eq!(retrieved_user.password, new_user.password);
            assert_eq!(retrieved_user.fullname, new_user.fullname);
            assert_eq!(retrieved_user.role, new_user.role);
        }

        #[test]
        fn read_returns_none_on_non_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);

            let retrieved_user = user_db.get(-666); // Use a non-existing ID

            assert!(retrieved_user.is_ok()); // Expecting Ok(none)
            assert!((retrieved_user.unwrap().is_none()));
        }

        #[test]
        fn update_succeeds_on_valid_input() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);

            let original_request = UpsertUser {
                email: "pondi@wwf.com".to_string(),
                password: "SnorkSnorkSnork".to_string(),
                fullname: "Panda Pondi".to_string(),
                role: "READER".to_string()
            };

            let original_user = user_db.create(original_request.clone()).expect("Create user failed");

            let updated_request = UpsertUser {
                email: "uhi@wwf.com".to_string(),
                password: "SlafsSlafsSlaf".to_string(),
                fullname: "Panda Pondi".to_string(),
                role: "READER".to_string()
            };

            let updated_user = user_db.update(original_user.id, updated_request.clone()).expect("Update user failed");

            assert_eq!(updated_user.email, updated_request.email);
            assert_eq!(updated_user.password, updated_request.password);
            assert_eq!(updated_user.fullname, updated_request.fullname);
            assert_eq!(updated_user.role, updated_request.role);
        }

        #[test]
        fn update_fails_on_nonexistent_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);

            let request = UpsertUser {
                email: "lukewarm@manlet.com".to_string(),
                password: "realfrogeyes".to_string(),
                fullname: "Lukas Parrot".to_string(),
                role: "READER".to_string()
            };

            let result = user_db.update(-666, request.clone());  // Use a non-existent ID

            assert!(result.is_err());  // Expecting an error as the ID is not present
        }

        #[test]
        fn delete_succeeds_on_existing_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);

            let request = UpsertUser {
                email: "world.according.to.jesse@mongols.com".to_string(),
                password: "bunchofslackjawedfgets".to_string(),
                fullname: "Jesse Ventura".to_string(),
                role: "READER".to_string()
            };

            let user = user_db.create(request.clone()).expect("Create user failed");
            user_db.delete(user.id.clone()).expect("Delete user failed");
            let deleted_user = user_db.get(user.id).expect("Read user failed");

            assert!(deleted_user.is_none()); // Expecting lack of value as user has been deleted
        }

        #[test]
        fn delete_fails_on_nonexistent_id() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);
            let result = user_db.delete(-666);  // Use a non-existent ID

            assert!(result.is_err());  // Expecting an error as the ID is not present
        }

        #[test]
        fn list_returns_all_users() {
            let database_url = load_environment_variable("TEST_DB");
            let connection_pool = create_shared_connection_pool(database_url, 1);
            let connection = connection_pool.pool.get().expect("Failed to get connection");
            let mut user_db = UsersTable::new(connection);

            // Create a few test users
            let user1 = UpsertUser {
                email: "user1@test.com".to_string(),
                password: "password1".to_string(),
                fullname: "User One".to_string(),
                role: "READER".to_string()
            };

            let user2 = UpsertUser {
                email: "user2@test.com".to_string(),
                password: "password2".to_string(),
                fullname: "User Two".to_string(),
                role: "WRITER".to_string()
            };

            user_db.create(user1).expect("Create user1 failed");
            user_db.create(user2).expect("Create user2 failed");

            // List all users
            let users = user_db.list().expect("List users failed");

            // Assert that we have at least 2 users (could be more from other tests)
            assert!(users.len() >= 2);

            // Check that our test users are in the list
            let emails: Vec<&String> = users.iter().map(|u| &u.email).collect();
            assert!(emails.contains(&&"user1@test.com".to_string()));
            assert!(emails.contains(&&"user2@test.com".to_string()));
        }
    }
}

