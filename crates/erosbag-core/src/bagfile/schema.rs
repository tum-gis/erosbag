table! {
    messages (id) {
        id -> Integer,
        topic_id -> Integer,
        timestamp -> BigInt,
        data -> Binary,
    }
}

table! {
    topics (id) {
        id -> Integer,
        name -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        serialization_format -> Text,
        offered_qos_profiles -> Text,
    }
}

allow_tables_to_appear_in_same_query!(messages, topics,);
