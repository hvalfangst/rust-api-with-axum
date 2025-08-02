pub mod service {
    use diesel::{
        prelude::*,
        PgConnection,
        r2d2::{ConnectionManager, PooledConnection},
    };
    use crate::{
        empires::model::{Empire, UpsertEmpire},
        schema
    };

    type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

    pub struct EmpiresTable {
        connection: PooledPg,
    }

    impl EmpiresTable {
        pub fn new(connection: PooledPg) -> EmpiresTable {
            EmpiresTable { connection }
        }

        pub fn create(&mut self, upsert_empire: UpsertEmpire) -> Result<Empire, diesel::result::Error> {
            use schema::empires;

            let new_empire = diesel::insert_into(empires::table)
                .values((
                    empires::name.eq(&upsert_empire.name),
                    empires::slogan.eq(&upsert_empire.slogan),
                    empires::location_id.eq(&upsert_empire.location_id),
                    empires::description.eq(&upsert_empire.description)
                ))
                .get_result(&mut self.connection)
                .expect("Create empire failed");

            Ok(new_empire)
        }

        pub fn get_all(&mut self) -> Result<Vec<Empire>, diesel::result::Error> {
            use schema::empires;

            let all_empires = empires::table
                .load::<Empire>(&mut self.connection)?;

            Ok(all_empires)
        }

        pub fn get(&mut self, empire_id: i32) -> Result<Option<Empire>, diesel::result::Error> {
            use schema::empires;

            let empire = empires::table
                .find(empire_id)
                .get_result(&mut self.connection)
                .optional()?;

            Ok(empire)
        }

        pub fn update(&mut self, empire_id: i32, upsert_empire: UpsertEmpire,
        ) -> Result<Empire, diesel::result::Error> {
            use schema::empires;

            // Check if the empire exists before attempting to update
            let existing_empire = empires::table
                .find(empire_id)
                .get_result::<Empire>(&mut self.connection);

            match existing_empire {
                Ok(_) => {
                    let updated_empire = diesel::update(empires::table.find(empire_id))
                        .set((
                            empires::name.eq(&upsert_empire.name),
                            empires::slogan.eq(&upsert_empire.slogan),
                            empires::location_id.eq(upsert_empire.location_id),
                            empires::description.eq(&upsert_empire.description)
                        ))
                        .get_result(&mut self.connection)
                        .expect("Update empire failed");

                    Ok(updated_empire)
                }
                Err(_) => Err(diesel::result::Error::NotFound),
            }
        }

        pub fn delete(&mut self, empire_id: i32) -> Result<(), diesel::result::Error> {
            use schema::empires;

            // Check if the empire exists before attempting to delete
            let existing_empire = empires::table
                .find(empire_id)
                .get_result::<Empire>(&mut self.connection);

            match existing_empire {
                Ok(_) => {
                    diesel::delete(empires::table.find(empire_id))
                        .execute(&mut self.connection)?;
                    Ok(())
                }
                Err(_) => {
                    Err(diesel::result::Error::NotFound)
                }
            }
        }
    }
}

