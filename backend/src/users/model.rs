use std::fmt;
use diesel::prelude::*;
use regex::Regex;
use serde_derive::{Serialize, Deserialize};
use crate::schema::users;

#[derive(Debug, Clone, Serialize, Queryable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub fullname: String,
    pub role: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    READER,
    WRITER,
    EDITOR,
    ADMIN,
    INVALID
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserRole::READER => write!(f, "READER"),
            UserRole::WRITER => write!(f, "WRITER"),
            UserRole::EDITOR => write!(f, "EDITOR"),
            UserRole::ADMIN => write!(f, "ADMIN"),
            UserRole::INVALID => write!(f, "INVALID"),
        }
    }
}

pub fn string_to_user_role(role: String) -> UserRole {
    match role.as_str() {
        "READER" => UserRole::READER,
        "WRITER" => UserRole::WRITER,
        "EDITOR" => UserRole::EDITOR,
        "ADMIN" => UserRole::ADMIN,
        _ => UserRole::INVALID
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct UpsertUser {
    pub email: String,
    pub password: String,
    pub fullname: String,
    pub role: String,
}

impl UpsertUser {
    pub fn is_valid_email(&self) -> bool {
        let email_pattern = Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").unwrap();
        email_pattern.is_match(&self.email)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub role: UserRole
}