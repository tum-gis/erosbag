use super::schema::messages;
use super::schema::topics;
use crate::ros_messages::RosMessageType;
use crate::topics::topic;
use crate::topics::topic::TopicSerializationFormat;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Insertable, Identifiable, Queryable, AsChangeset)]
pub struct Topic {
    pub id: i32,
    pub name: String,
    pub type_: String,
    pub serialization_format: String,
    pub offered_qos_profiles: String,
}

impl From<&Topic> for topic::TopicMetadata {
    fn from(item: &Topic) -> Self {
        Self::new(
            RosMessageType::from_str(item.type_.as_str()).unwrap(),
            TopicSerializationFormat::from_str(item.serialization_format.as_str()).unwrap(),
            serde_yaml::from_str(&item.offered_qos_profiles).unwrap(), // TODO: handle
        )
    }
}

impl From<&Topic> for topic::TopicMetadataWithName {
    fn from(item: &Topic) -> Self {
        Self::from(
            item.name.clone(),
            RosMessageType::from_str(item.type_.as_str()).unwrap(),
            TopicSerializationFormat::from_str(item.serialization_format.as_str()).unwrap(),
            serde_yaml::from_str(&item.offered_qos_profiles).unwrap(), // TODO: handle
        )
    }
}

#[derive(Insertable)]
#[diesel(table_name = topics)]
pub struct NewTopic {
    pub name: String,
    pub type_: String,
    pub serialization_format: String,
    pub offered_qos_profiles: String,
}

impl From<&topic::TopicMetadataWithName> for NewTopic {
    fn from(item: &topic::TopicMetadataWithName) -> Self {
        Self {
            name: item.name.clone(),
            type_: item.topic.message_type.clone().as_str().to_string(),
            serialization_format: item.topic.serialization_format.as_str().to_string(),
            offered_qos_profiles: serde_yaml::to_string(&item.topic.offered_qos_profiles).unwrap(),
        }
    }
}

/*NewTopic {
name: name.clone(),
type_: metadata.message_type.as_str().to_string(),
serialization_format: metadata.serialization_format.as_str().to_string(),
offered_qos_profiles: serde_yaml::to_string(&metadata.offered_qos_profiles).unwrap(),
};*/

#[derive(Debug, Clone, Eq, PartialEq, Insertable, Identifiable, Queryable)]
pub struct Message {
    pub id: i32,
    pub topic_id: i32,
    pub timestamp: i64,
    pub data: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = messages)]
pub struct NewMessage {
    pub topic_id: i32,
    pub timestamp: i64,
    pub data: Vec<u8>,
}
