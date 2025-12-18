use crate::Error::ContainsNoMcapFileWithName;
use crate::dto::{McapFileOverview, McapMessagePage, McapOverview};
use crate::identifier::{ChannelId, FileName};
use crate::mcap_file::McapFile;
use crate::ros_messages::RosMessageType;
use crate::{ChannelTopic, ChunkId, Error, MCAP_EXTENSION, dto};
use chrono::{DateTime, Utc};
use ecoord::TransformTree;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Rosbag {
    pub directory_path: PathBuf,
    pub mcap_files: HashMap<FileName, McapFile>,
}

impl Rosbag {
    pub fn new(directory_path: impl AsRef<Path>) -> Result<Self, Error> {
        let mcap_files: Vec<McapFile> = std::fs::read_dir(&directory_path)?
            .filter_map(|x| x.ok())
            .filter(|x| x.path().is_file())
            .filter(|x| x.path().extension() == Some(std::ffi::OsStr::new(MCAP_EXTENSION)))
            .sorted_by_key(|x| x.path())
            .map(|x| {
                McapFile::new(
                    x.path().file_stem().unwrap().to_str().unwrap().into(),
                    x.path(),
                )
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(Self {
            directory_path: directory_path.as_ref().to_owned(),
            mcap_files: mcap_files
                .into_iter()
                .map(|x| (x.file_name.clone(), x))
                .collect(),
        })
    }

    pub fn get_file_ids(&self) -> HashSet<FileName> {
        self.mcap_files.keys().cloned().sorted().collect()
    }

    pub fn get_last_file_id(&self) -> Option<FileName> {
        self.mcap_files.keys().last().cloned()
    }

    pub fn get_start_date_time(&self) -> Result<Option<DateTime<Utc>>, Error> {
        Ok(self
            .mcap_files
            .values()
            .map(|x| x.get_start_date_time())
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .flatten()
            .min()
            .copied())
    }

    pub fn get_end_date_time(&self) -> Result<Option<DateTime<Utc>>, Error> {
        Ok(self
            .mcap_files
            .values()
            .map(|x| x.get_end_date_time())
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .flatten()
            .max()
            .copied())
    }

    pub fn contains_channel(&self, channel_topic: &ChannelTopic) -> Result<bool, Error> {
        let contains_channel = self
            .mcap_files
            .values()
            .map(|x| {
                let channel_id = x.get_channel_id(channel_topic)?;
                x.contains_channel(channel_id)
            })
            .collect::<Result<Vec<bool>, Error>>()?;

        Ok(contains_channel.into_iter().any(|x| x))
    }

    pub fn get_overview(&self) -> Result<dto::McapOverview, Error> {
        let file_overviews: BTreeMap<FileName, McapFileOverview> = self
            .mcap_files
            .iter()
            .map(|x| x.1.get_overview().map(|i| (x.0.clone(), i)))
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        let rosbag_overviews = McapOverview::new(file_overviews);
        Ok(rosbag_overviews)
    }

    pub fn get_start_date_time_of_channel(
        &self,
        channel_topic: &ChannelTopic,
    ) -> Result<Option<DateTime<Utc>>, Error> {
        let start_date_times: Vec<DateTime<Utc>> = self
            .mcap_files
            .values()
            .map(|x| {
                let channel_id = x.get_channel_id(channel_topic)?;
                x.get_start_date_time_of_channel(channel_id)
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .flatten()
            .collect();

        let total_start_date_time = start_date_times.into_iter().min();
        Ok(total_start_date_time)
    }

    pub fn get_start_date_time_of_channels(
        &self,
        channel_topics: &HashSet<ChannelTopic>,
    ) -> Result<Option<DateTime<Utc>>, Error> {
        let start_date_times: Vec<DateTime<Utc>> = channel_topics
            .iter()
            .flat_map(|topic| {
                self.mcap_files
                    .values()
                    .filter_map(|mcap_file| {
                        let channel_id = mcap_file.get_channel_id(topic).ok()?;
                        mcap_file
                            .get_start_date_time_of_channel(channel_id)
                            .ok()
                            .flatten()
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(start_date_times.into_iter().min())
    }

    pub fn get_end_date_time_of_channel(
        &self,
        channel_topic: &ChannelTopic,
    ) -> Result<Option<DateTime<Utc>>, Error> {
        let end_date_times: Vec<DateTime<Utc>> = self
            .mcap_files
            .values()
            .map(|x| {
                let channel_id = x.get_channel_id(channel_topic)?;
                x.get_end_date_time_of_channel(channel_id)
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .flatten()
            .collect();

        let total_end_date_time = end_date_times.into_iter().max();
        Ok(total_end_date_time)
    }

    pub fn get_end_date_time_of_channels(
        &self,
        channel_topics: &HashSet<ChannelTopic>,
    ) -> Result<Option<DateTime<Utc>>, Error> {
        let start_date_times: Vec<DateTime<Utc>> = channel_topics
            .iter()
            .flat_map(|topic| {
                self.mcap_files
                    .values()
                    .filter_map(|mcap_file| {
                        let channel_id = mcap_file.get_channel_id(topic).ok()?;
                        mcap_file
                            .get_end_date_time_of_channel(channel_id)
                            .ok()
                            .flatten()
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(start_date_times.into_iter().max())
    }

    fn get_file_within_date_times(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        end_date_time: &Option<DateTime<Utc>>,
    ) -> Vec<&McapFile> {
        self.mcap_files
            .values()
            .filter(|x| {
                let file_start_date_time: DateTime<Utc> = x
                    .get_start_date_time()
                    .expect("should work")
                    .expect("should contain start date time");
                let file_end_date_time: DateTime<Utc> = x
                    .get_end_date_time()
                    .expect("should work")
                    .expect("should contain end date time");

                let within_start = start_date_time.is_none_or(|start| file_end_date_time >= start);
                let within_end = end_date_time.is_none_or(|end| file_start_date_time <= end);

                within_start && within_end
            })
            .sorted_by_key(|x| &x.file_name)
            .collect()
    }

    pub fn get_message_page_with_chunk_ids(
        &self,
        file_name: &FileName,
        chunk_ids: &[ChunkId],
        channel_topics: &Option<HashSet<ChannelTopic>>,
    ) -> Result<McapMessagePage, Error> {
        let file = self
            .mcap_files
            .get(file_name)
            .ok_or(ContainsNoMcapFileWithName(file_name.clone()))?;

        let channel_ids: Option<HashSet<ChannelId>> = channel_topics
            .as_ref()
            .map(|x| file.get_channel_ids(x))
            .transpose()?;

        let pages = vec![file.read_chunks_with_ids(chunk_ids, &channel_ids)?];
        Ok(McapMessagePage::combine(pages))
    }

    pub fn get_transforms(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        end_date_time: &Option<DateTime<Utc>>,
        channel_topics: &Option<HashSet<ChannelTopic>>,
    ) -> Result<TransformTree, Error> {
        let combined_page = self.get_message_page_with_type_fallback(
            start_date_time,
            end_date_time,
            channel_topics,
            RosMessageType::Tf2MessagesTFMessage,
        )?;
        let transform_tree = combined_page.get_all_transform_tree()?;
        Ok(transform_tree)
    }

    /// Returns the point cloud of optionally selected channels for a time window between
    /// start_date_time (inclusive) and end_date_time (exclusive).
    pub fn get_point_clouds(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        end_date_time: &Option<DateTime<Utc>>,
        channel_topics: &Option<HashSet<ChannelTopic>>,
    ) -> Result<epoint::PointCloud, Error> {
        let combined_page = self.get_message_page_with_type_fallback(
            start_date_time,
            end_date_time,
            channel_topics,
            RosMessageType::SensorMessagesPointCloud2,
        )?;
        let point_cloud = combined_page.get_point_cloud_messages_combined(
            start_date_time,
            end_date_time,
            channel_topics,
        )?;
        Ok(point_cloud)
    }

    /// Returns the point cloud with transforms of optionally selected channels for a time window between
    /// start_date_time (inclusive) and end_date_time (exclusive).
    pub fn get_point_clouds_with_transforms(
        &self,
        point_cloud_start_date_time: &Option<DateTime<Utc>>,
        point_cloud_end_date_time: &Option<DateTime<Utc>>,
        point_cloud_channel_topics: &Option<HashSet<ChannelTopic>>,
        transforms_start_date_time: &Option<DateTime<Utc>>,
        transforms_end_date_time: &Option<DateTime<Utc>>,
        transforms_channel_topics: &Option<HashSet<ChannelTopic>>,
    ) -> Result<epoint::PointCloud, Error> {
        let mut point_cloud = self.get_point_clouds(
            point_cloud_start_date_time,
            point_cloud_end_date_time,
            point_cloud_channel_topics,
        )?;

        point_cloud.transform_tree = self.get_transforms(
            transforms_start_date_time,
            transforms_end_date_time,
            transforms_channel_topics,
        )?;

        Ok(point_cloud)
    }

    /// Returns the images of optionally selected channels for a time window between
    /// start_date_time (inclusive) and end_date_time (exclusive).
    pub fn get_images(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        end_date_time: &Option<DateTime<Utc>>,
        channel_topics: &Option<HashSet<ChannelTopic>>,
    ) -> Result<eimage::ImageCollection, Error> {
        let combined_page = self.get_message_page_with_type_fallback(
            start_date_time,
            end_date_time,
            channel_topics,
            RosMessageType::SensorMessagesImage,
        )?;
        let image_collection = combined_page.get_all_images()?;
        Ok(image_collection)
    }

    pub fn get_message_page(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        end_date_time: &Option<DateTime<Utc>>,
        channel_topics: &HashSet<ChannelTopic>,
    ) -> Result<McapMessagePage, Error> {
        let chunk_ids_to_read: BTreeMap<FileName, Vec<ChunkId>> = self
            .get_overview()?
            .get_chunk_ids_of_channel_topics(start_date_time, end_date_time, channel_topics);

        let mut pages: Vec<McapMessagePage> = Vec::new();
        for (current_file, current_chunk_ids) in chunk_ids_to_read {
            let current_page = self.get_message_page_with_chunk_ids(
                &current_file,
                &current_chunk_ids,
                &Some(channel_topics.clone()),
            )?;

            pages.push(current_page);
        }
        let combined_page = McapMessagePage::combine(pages);
        Ok(combined_page)
    }

    pub fn get_message_page_of_first_chunk_per_channel_topic(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        end_date_time: &Option<DateTime<Utc>>,
        channel_topics: &HashSet<ChannelTopic>,
    ) -> Result<McapMessagePage, Error> {
        let chunk_ids_to_read: BTreeMap<FileName, Vec<ChunkId>> = self
            .get_overview()?
            .get_first_chunk_ids_of_channel_topics(start_date_time, end_date_time, channel_topics);

        let mut pages: Vec<McapMessagePage> = Vec::new();
        for (current_file, current_chunk_ids) in chunk_ids_to_read {
            let current_page = self.get_message_page_with_chunk_ids(
                &current_file,
                &current_chunk_ids,
                &Some(channel_topics.clone()),
            )?;

            pages.push(current_page);
        }
        let combined_page = McapMessagePage::combine(pages);
        Ok(combined_page)
    }

    fn get_message_page_with_type_fallback(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        end_date_time: &Option<DateTime<Utc>>,
        channel_topics: &Option<HashSet<ChannelTopic>>,
        message_type_fallback: RosMessageType,
    ) -> Result<McapMessagePage, Error> {
        let relevant_channel_topics: HashSet<ChannelTopic> =
            if let Some(channel_topics) = channel_topics {
                channel_topics.clone()
            } else {
                self.get_overview()?
                    .get_channel_topics_of_message_type(message_type_fallback)
            };

        self.get_message_page(start_date_time, end_date_time, &relevant_channel_topics)
    }
}
