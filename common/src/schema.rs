table! {
    channels (id) {
        id -> Int4,
        uuid -> Uuid,
        twitch_channel_id -> Varchar,
        channel_name -> Varchar,
    }
}

table! {
    messages (id) {
        id -> Int8,
        uuid -> Uuid,
        msg -> Varchar,
        msg_type -> Varchar,
        user_id -> Int4,
        channel_id -> Int4,
        resub_id -> Nullable<Int4>,
        send_time -> Timestamptz,
        bits -> Nullable<Int8>,
    }
}

table! {
    resubs (id) {
        id -> Int4,
        uuid -> Uuid,
        cumulative_month -> Int2,
        tier -> Int2,
    }
}

table! {
    users (id) {
        id -> Int4,
        uuid -> Uuid,
        username -> Varchar,
        twitch_user_id -> Varchar,
    }
}

table! {
    users_old_names (id) {
        id -> Int4,
        user_id -> Int4,
        username -> Varchar,
        first_time_with_new_name -> Timestamptz,
    }
}

joinable!(messages -> channels (channel_id));
joinable!(messages -> resubs (resub_id));
joinable!(messages -> users (user_id));
joinable!(users_old_names -> users (user_id));

allow_tables_to_appear_in_same_query!(
    channels,
    messages,
    resubs,
    users,
    users_old_names,
);
