// @generated automatically by Diesel CLI.

diesel::table! {
    empires (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 100]
        slogan -> Varchar,
        location_id -> Int4,
        description -> Text,
    }
}

diesel::table! {
    locations (id) {
        id -> Int4,
        #[max_length = 100]
        star_system -> Varchar,
        #[max_length = 100]
        area -> Varchar,
    }
}

diesel::table! {
    players (id) {
        id -> Int4,
        user_id -> Int4,
        active_ship_id -> Int4,
        location_id -> Int4,
    }
}

diesel::table! {
    ships (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 50]
        category -> Nullable<Varchar>,
        description -> Nullable<Text>,
        empire_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 100]
        password -> Varchar,
        #[max_length = 100]
        fullname -> Varchar,
        #[max_length = 10]
        role -> Varchar,
    }
}

diesel::joinable!(empires -> locations (location_id));
diesel::joinable!(players -> locations (location_id));
diesel::joinable!(players -> ships (active_ship_id));
diesel::joinable!(players -> users (user_id));
diesel::joinable!(ships -> empires (empire_id));

diesel::allow_tables_to_appear_in_same_query!(
    empires,
    locations,
    players,
    ships,
    users,
);
