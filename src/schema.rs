use diesel::table;

table! {
    todo {
        id -> Integer,
        content -> Text,
        expire_time -> Nullable<Timestamp>,
    }
}