use crate::ros_messages::RosMessageType;
use crate::topics::qos_profile::QualityOfServiceProfile;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TopicMetadata {
    pub message_type: RosMessageType,
    pub serialization_format: TopicSerializationFormat,
    pub offered_qos_profiles: Vec<QualityOfServiceProfile>,
}

impl TopicMetadata {
    pub fn new(
        message_type: RosMessageType,
        serialization_format: TopicSerializationFormat,
        offered_qos_profiles: Vec<QualityOfServiceProfile>,
    ) -> Self {
        Self {
            message_type,
            serialization_format,
            offered_qos_profiles,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TopicMetadataWithName {
    pub name: String,
    pub topic: TopicMetadata,
}

impl TopicMetadataWithName {
    pub fn new(name: String, topic: TopicMetadata) -> Self {
        Self { name, topic }
    }

    pub fn from(
        name: String,
        message_type: RosMessageType,
        serialization_format: TopicSerializationFormat,
        offered_qos_profiles: Vec<QualityOfServiceProfile>,
    ) -> Self {
        let topic = TopicMetadata::new(message_type, serialization_format, offered_qos_profiles);

        Self { name, topic }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TopicSerializationFormat {
    CDR,
}

impl FromStr for TopicSerializationFormat {
    type Err = ();

    fn from_str(input: &str) -> Result<TopicSerializationFormat, Self::Err> {
        match input {
            "cdr" => Ok(TopicSerializationFormat::CDR),
            _ => Err(()),
        }
    }
}

impl TopicSerializationFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            TopicSerializationFormat::CDR => "cdr",
        }
    }
}
