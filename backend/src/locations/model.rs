use diesel::prelude::*;
use serde_derive::{Serialize, Deserialize};
use crate::schema::locations;

#[derive(Serialize, Debug, Clone, Queryable)]
#[diesel(table_name = locations)]
pub struct Location {
    pub id: i32,
    pub star_system: String,
    pub area: String,
}

#[derive(Debug, Clone, Insertable, Deserialize, Serialize)]
#[diesel(table_name = locations)]
pub struct UpsertLocation {
    pub star_system: String,
    pub area: String,
}