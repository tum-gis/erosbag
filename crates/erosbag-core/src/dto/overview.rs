use crate::ros_messages::RosMessageType;
use crate::{ChannelId, ChannelTopic, ChunkId, FileName};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct McapOverview {
    pub files: BTreeMap<FileName, McapFileOverview>,
}

impl McapOverview {
    pub fn new(files: BTreeMap<FileName, McapFileOverview>) -> Self {
        Self { files }
    }

    pub fn get_channel_topics_of_message_type(
        &self,
        ros_message_type: RosMessageType,
    ) -> HashSet<ChannelTopic> {
        self.files
            .values()
            .flat_map(|current_file| {
                current_file
                    .get_channel_ids_of_message_type(ros_message_type)
                    .into_iter()
                    .map(|x| current_file.get_channel_topic(x))
            })
            .flatten()
            .collect()
    }

    pub fn get_channel_topics_of_message_types(
        &self,
        ros_message_types: HashSet<RosMessageType>,
    ) -> HashMap<RosMessageType, HashSet<ChannelTopic>> {
        ros_message_types
            .into_iter()
            .map(|x| (x, self.get_channel_topics_of_message_type(x)))
            .collect()
    }

    pub fn get_chunk_ids_of_channel_topics(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        stop_date_time: &Option<DateTime<Utc>>,
        channel_topics: &HashSet<ChannelTopic>,
    ) -> BTreeMap<FileName, Vec<ChunkId>> {
        self.files
            .iter()
            .map(|(i, x)| {
                let chunk_ids = x.get_chunk_ids_containing_channel_ids(
                    start_date_time,
                    stop_date_time,
                    &x.get_channel_ids_from_topics(channel_topics),
                );

                (i.clone(), chunk_ids)
            })
            .collect()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct McapFileOverview {
    pub channels: HashMap<ChannelId, ChannelOverview>,
    pub chunks: HashMap<ChunkId, ChunkOverview>,
}

impl McapFileOverview {
    pub fn new(channels: Vec<ChannelOverview>, chunks: Vec<ChunkOverview>) -> Self {
        Self {
            channels: channels.into_iter().map(|x| (x.id, x)).collect(),
            chunks: chunks.into_iter().map(|x| (x.id, x)).collect(),
        }
    }

    /// Returns the total number of topics in this page.
    pub fn len(&self) -> usize {
        self.channels.len()
    }

    /// Checks if the page is empty.
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }

    pub fn get_channel_ids(&self) -> HashSet<ChannelId> {
        self.channels.keys().cloned().collect()
    }

    pub fn get_channel_id_of_topic(&self, channel_topic: &ChannelTopic) -> Option<ChannelId> {
        self.channels
            .values()
            .find(|x| &x.topic == channel_topic)
            .map(|x| x.id)
    }

    pub fn get_channel_ids_from_topics(
        &self,
        channel_topics: &HashSet<ChannelTopic>,
    ) -> HashSet<ChannelId> {
        channel_topics
            .iter()
            .flat_map(|x| self.get_channel_id_of_topic(x))
            .collect()
    }

    pub fn get_channel_topic(&self, channel_id: ChannelId) -> Option<ChannelTopic> {
        self.channels.get(&channel_id).map(|x| x.topic.clone())
    }

    pub fn get_channel_ids_of_message_type(
        &self,
        ros_message_type: RosMessageType,
    ) -> HashSet<ChannelId> {
        self.channels
            .values()
            .filter(|x| x.ros_message_type.is_some_and(|y| y == ros_message_type))
            .map(|x| x.id)
            .collect()
    }

    pub fn get_message_count_for_channel(&self, channel_id: ChannelId) -> usize {
        self.chunks
            .values()
            .filter(|x| x.contained_channel.contains(&channel_id))
            .count()
    }

    pub fn total_messages(&self) -> usize {
        self.chunks
            .values()
            .map(|x| x.contained_channel.len())
            .sum()
    }

    pub fn get_chunk_ids_containing_channel_ids(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        stop_date_time: &Option<DateTime<Utc>>,
        channel_ids: &HashSet<ChannelId>,
    ) -> Vec<ChunkId> {
        self.chunks
            .values()
            .filter(|chunk| {
                start_date_time.is_none_or(|start| chunk.stop_date_time >= start)
                    && stop_date_time.is_none_or(|stop| chunk.start_date_time <= stop)
                    && !chunk.contained_channel.is_disjoint(channel_ids)
            })
            .map(|chunk| chunk.id)
            .collect()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChannelOverview {
    id: ChannelId,
    pub topic: ChannelTopic,
    pub ros_message_type: Option<RosMessageType>,
}

impl ChannelOverview {
    pub fn new(
        id: ChannelId,
        topic: ChannelTopic,
        ros_message_type: Option<RosMessageType>,
    ) -> Self {
        Self {
            id,
            topic,
            ros_message_type,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChunkOverview {
    pub id: ChunkId,
    pub start_date_time: DateTime<Utc>,
    pub stop_date_time: DateTime<Utc>,
    pub contained_channel: HashSet<ChannelId>,
}

impl ChunkOverview {
    pub fn new(
        id: ChunkId,
        start_date_time: DateTime<Utc>,
        stop_date_time: DateTime<Utc>,
        contained_channel: HashSet<ChannelId>,
    ) -> Self {
        Self {
            id,
            start_date_time,
            stop_date_time,
            contained_channel,
        }
    }
}
