table! {
    devices (id) {
        id -> Integer,
        registration_id -> Text,
        name -> Text,
        user -> Integer,
        online -> Bool,
        registration_date -> Text,
        last_login -> Text,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        password -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    devices,
    users,
);
