table! {
    messages (id) {
        id -> Int4,
        from -> Int4,
        to -> Int4,
        content -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        salt -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    messages,
    users,
);
