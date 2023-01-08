use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct RosbagMetaDataDocument {
    pub rosbag2_bagfile_information: Rosbag2BagfileInformationElement,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Rosbag2BagfileInformationElement {
    pub version: u32,
    pub storage_identifier: String,
    pub duration: DurationElement,
    pub starting_time: StartingTimeElement,
    pub message_count: u64,
    pub topics_with_message_count: Vec<TopicWithMessageCountElement>,
    pub compression_format: String,
    pub compression_mode: String,
    pub relative_file_paths: Vec<String>,
    pub files: Vec<FileElement>,
    pub custom_data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct DurationElement {
    pub nanoseconds: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct StartingTimeElement {
    pub nanoseconds_since_epoch: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TopicWithMessageCountElement {
    pub topic_metadata: TopicMetaDataElement,
    pub message_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TopicMetaDataElement {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub serialization_format: String,
    pub offered_qos_profiles: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct FileElement {
    pub path: String,
    pub starting_time: StartingTimeElement,
    pub duration: DurationElement,
    pub message_count: u64,
}
