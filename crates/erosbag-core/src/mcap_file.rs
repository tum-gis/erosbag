use crate::dto::{McapFileOverview, McapMessagePage};
use crate::identifier::{ChannelId, FileName};
use crate::ros_messages::RosMessageType;
use crate::{ChannelTopic, ChunkId, Error, MessageId, dto};
use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;
use mcap::records::Statistics;
use mcap::{Summary, records};
use memmap::Mmap;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;

fn map_mcap(file_path: impl AsRef<Path>) -> Mmap {
    let fd = std::fs::File::open(file_path).expect("Couldn't open MCAP file");
    unsafe { Mmap::map(&fd) }.expect("Couldn't mmap MCAP file")
}

#[derive(Debug)]
pub struct McapFile {
    pub(crate) file_name: FileName,
    pub(crate) mapped: Mmap,
}

impl McapFile {
    pub fn new(id: FileName, file_path: impl AsRef<Path>) -> Result<Self, Error> {
        let mapped = map_mcap(&file_path);

        Ok(Self {
            file_name: id,
            mapped,
        })
    }

    pub(crate) fn summary(&self) -> Result<Option<Summary>, Error> {
        let summary = mcap::Summary::read(&self.mapped)?;
        Ok(summary)
    }

    pub(crate) fn stats(&self) -> Result<Option<Statistics>, Error> {
        let summary = self.summary()?;
        let stats = summary.and_then(|summary| summary.stats);
        Ok(stats)
    }

    pub fn get_overview(&self) -> Result<McapFileOverview, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let channels: Vec<dto::ChannelOverview> = summary
            .channels
            .values()
            .map(|x| {
                let id: ChannelId = x.id.into();
                let topic: ChannelTopic = x.topic.clone().into();
                let ros_message_type =
                    RosMessageType::from_str(x.schema.clone().unwrap().name.as_str()).ok();

                dto::ChannelOverview::new(id, topic, ros_message_type)
            })
            .collect();

        let chunks: Vec<dto::ChunkOverview> = summary
            .chunk_indexes
            .into_iter()
            .enumerate()
            .map(|(current_index, current_chunk_index)| {
                let id: ChunkId = current_index.into();
                let start_date_time: DateTime<Utc> =
                    Utc.timestamp_nanos(current_chunk_index.message_start_time as i64);
                let end_date_time: DateTime<Utc> =
                    Utc.timestamp_nanos(current_chunk_index.message_end_time as i64);
                let contained_channel: HashSet<ChannelId> = current_chunk_index
                    .message_index_offsets
                    .keys()
                    .map(|x| x.into())
                    .collect();

                dto::ChunkOverview::new(id, start_date_time, end_date_time, contained_channel)
            })
            .collect();

        Ok(dto::McapFileOverview::new(channels, chunks))
    }

    pub fn get_start_date_time(&self) -> Result<Option<DateTime<Utc>>, Error> {
        let stats = self.stats()?;
        let timestamp = stats.map(|x| Utc.timestamp_nanos(x.message_start_time as i64));
        Ok(timestamp)
    }

    pub fn get_stop_date_time(&self) -> Result<Option<DateTime<Utc>>, Error> {
        let stats = self.stats()?;
        let timestamp = stats.map(|x| Utc.timestamp_nanos(x.message_end_time as i64));
        Ok(timestamp)
    }

    pub fn get_start_date_time_of_channel(
        &self,
        channel_id: ChannelId,
    ) -> Result<Option<DateTime<Utc>>, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let date_time = summary
            .chunk_indexes
            .into_iter()
            .filter(|x| x.message_index_offsets.contains_key(&channel_id.into()))
            .min_by_key(|x| x.message_start_time)
            .map(|x| Utc.timestamp_nanos(x.message_start_time as i64));

        Ok(date_time)
    }

    pub fn get_stop_date_time_of_channel(
        &self,
        channel_id: ChannelId,
    ) -> Result<Option<DateTime<Utc>>, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let date_time = summary
            .chunk_indexes
            .into_iter()
            .filter(|x| x.message_index_offsets.contains_key(&channel_id.into()))
            .max_by_key(|x| x.message_end_time)
            .map(|x| Utc.timestamp_nanos(x.message_end_time as i64));

        Ok(date_time)
    }

    pub fn contains_channel(&self, channel_id: ChannelId) -> Result<bool, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let contains_channel = summary.channels.contains_key(&channel_id.into());
        Ok(contains_channel)
    }

    pub fn get_message_count_per_channel(&self) -> Result<HashMap<ChannelId, u64>, Error> {
        let counts_per_channel: HashMap<ChannelId, u64> = self
            .stats()?
            .expect("should be present")
            .channel_message_counts
            .into_iter()
            .map(|x| (x.0.into(), x.1))
            .collect();

        Ok(counts_per_channel)
    }

    pub fn get_channel_topic(&self, channel_id: ChannelId) -> Result<ChannelTopic, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let channel = summary
            .channels
            .get(&channel_id.into())
            .ok_or(Error::ChannelWithIdDoesNotExist(channel_id))?;

        Ok(channel.topic.clone().into())
    }

    pub fn get_channel_id(&self, channel_topic: &ChannelTopic) -> Result<ChannelId, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let found_channel = summary
            .channels
            .iter()
            .find(|(i, x)| x.topic == channel_topic.to_string())
            .ok_or(Error::ChannelWithTopicDoesNotExist(channel_topic.clone()))?;

        Ok(ChannelId::from(*found_channel.0))
    }

    pub fn get_channel_ids(
        &self,
        channel_topics: &HashSet<ChannelTopic>,
    ) -> Result<HashSet<ChannelId>, Error> {
        channel_topics
            .iter()
            .map(|x| self.get_channel_id(x))
            .collect()
    }

    pub fn get_all_channel_ids(&self) -> Result<HashSet<ChannelId>, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let all_channel_ids: HashSet<ChannelId> =
            summary.channels.keys().map(|x| x.into()).collect();
        Ok(all_channel_ids)
    }

    pub fn get_channel_ids_of_message_type(
        &self,
        message_type: &RosMessageType,
    ) -> Result<HashSet<ChannelId>, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let channel_ids = summary
            .channels
            .iter()
            .filter(|(i, x)| x.schema.clone().unwrap().name.as_str() == message_type.as_str())
            .map(|(i, x)| ChannelId::from(*i))
            .collect::<HashSet<ChannelId>>();
        Ok(channel_ids)
    }

    pub fn get_channel_ids_of_message_types(
        &self,
        message_types: &HashSet<RosMessageType>,
    ) -> Result<HashSet<ChannelId>, Error> {
        let summary = self.summary()?.expect("summary should be present");

        let message_types_str: HashSet<&str> = message_types.iter().map(|x| x.as_str()).collect();

        let channel_ids = summary
            .channels
            .iter()
            .filter(|(i, x)| message_types_str.contains(x.schema.clone().unwrap().name.as_str()))
            .map(|(i, x)| ChannelId::from(*i))
            .collect::<HashSet<ChannelId>>();
        Ok(channel_ids)
    }

    fn get_chunk_indices(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        stop_date_time: &Option<DateTime<Utc>>,
        channel_id_selection: &Option<HashSet<ChannelId>>,
    ) -> Result<Vec<ChunkId>, Error> {
        let summary = self.summary()?.unwrap();

        let mut chunk_ids: Vec<(ChunkId, records::ChunkIndex)> = summary
            .chunk_indexes
            .into_iter()
            .enumerate()
            .map(|x| (x.0.into(), x.1))
            .collect();

        if let Some(start_date_time) = start_date_time {
            chunk_ids
                .retain(|x| *start_date_time <= Utc.timestamp_nanos(x.1.message_start_time as i64));
        }

        if let Some(stop_date_time) = stop_date_time {
            chunk_ids
                .retain(|x| Utc.timestamp_nanos(x.1.message_end_time as i64) <= *stop_date_time);
        }

        if let Some(channel_id_selection) = channel_id_selection {
            let channel_id_selection: HashSet<u16> = channel_id_selection
                .iter()
                .map(|x| u16::from(*x))
                .collect::<HashSet<_>>();
            chunk_ids.retain(|x| {
                channel_id_selection
                    .iter()
                    .any(|k| x.1.message_index_offsets.keys().contains(k))
            });
        }

        Ok(chunk_ids.iter().map(|x| x.0).collect())
    }

    pub fn read_chunks_with_ids(
        &self,
        chunk_ids: &[ChunkId],
        channel_ids: &Option<HashSet<ChannelId>>,
    ) -> Result<McapMessagePage, Error> {
        let pages = chunk_ids
            .into_par_iter()
            .map(|x| self.read_chunk(*x, channel_ids))
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(McapMessagePage::combine(pages))
    }

    fn read_chunk(
        &self,
        chunk_id: ChunkId,
        channel_topics: &Option<HashSet<ChannelId>>,
    ) -> Result<McapMessagePage, Error> {
        let summary = self.summary().expect("should work").unwrap();
        let chunk_index = summary.chunk_indexes.get::<usize>(chunk_id.into()).unwrap();
        let mut messages: Vec<(MessageId, mcap::Message)> = summary
            .stream_chunk(&self.mapped, chunk_index)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .enumerate()
            .map(|(i, x)| (i.into(), x))
            .collect::<Vec<_>>();

        if let Some(channel_ids) = channel_topics {
            messages.retain(|x| channel_ids.contains(&x.1.channel.id.into()))
        }

        let messages: Vec<dto::McapMessageMeta<mcap::Message>> = messages
            .into_iter()
            .map(|(i, x)| {
                dto::McapMessageMeta::new(
                    self.file_name.clone(),
                    x.channel.topic.clone().into(),
                    chunk_id,
                    i,
                    Utc.timestamp_nanos(x.log_time as i64),
                    Utc.timestamp_nanos(x.publish_time as i64),
                    x,
                )
            })
            .collect::<Vec<_>>();

        let message_page = McapMessagePage::from(messages)?;
        Ok(message_page)
    }
}
