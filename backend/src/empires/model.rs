use diesel::prelude::*;
use serde_derive::{Serialize, Deserialize};
use crate::schema::empires;

#[derive(Serialize, Debug, Clone, Queryable)]
#[diesel(table_name = empires)]
pub struct Empire {
    pub id: i32,
    pub name: String,
    pub slogan: String,
    pub location_id: i32,
    pub description: String
}

#[derive(Debug, Clone, Insertable, Deserialize, Serialize)]
#[diesel(table_name = empires)]
pub struct UpsertEmpire {
    pub name: String,
    pub slogan: String,
    pub location_id: i32,
    pub description: String
}