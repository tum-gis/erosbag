use crate::Error::{ChannelDoesNotHold, ChannelWithoutSchema};
use crate::ros_messages::RosMessageType;
use crate::ros_messages::{geometry_msgs, sensor_msgs};
use crate::{ChannelTopic, ChunkId, Error, FileName, MessageId, ros_messages};
use chrono::{DateTime, Utc};
use ecoord::{FrameId, Transform, TransformId, TransformInfo};
use eimage::ImageSeries;
use itertools::Itertools;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tracing::warn;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct McapMessageMeta<T> {
    pub file_name: FileName,
    pub channel_topic: ChannelTopic,
    pub chunk_id: ChunkId,
    pub message_id: MessageId,
    pub log_date_time: DateTime<Utc>,
    pub publish_date_time: DateTime<Utc>,
    pub message: T,
}

impl<T> McapMessageMeta<T> {
    pub fn new(
        file_name: FileName,
        channel_topic: ChannelTopic,
        chunk_id: ChunkId,
        message_id: MessageId,
        log_date_time: DateTime<Utc>,
        publish_date_time: DateTime<Utc>,
        message: T,
    ) -> Self {
        Self {
            file_name,
            channel_topic,
            chunk_id,
            message_id,
            log_date_time,
            publish_date_time,
            message,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct McapMessagePage {
    pub imu_messages: HashMap<ChannelTopic, Vec<McapMessageMeta<sensor_msgs::Imu>>>,
    pub nav_sat_fix_messages: HashMap<ChannelTopic, Vec<McapMessageMeta<sensor_msgs::NavSatFix>>>,
    pub point_cloud_messages: HashMap<ChannelTopic, Vec<McapMessageMeta<sensor_msgs::PointCloud2>>>,
    pub image_messages: HashMap<ChannelTopic, Vec<McapMessageMeta<sensor_msgs::Image>>>,
    pub tf_messages: HashMap<ChannelTopic, Vec<McapMessageMeta<ros_messages::tf2_msgs::TFMessage>>>,
    pub odometry_messages:
        HashMap<ChannelTopic, Vec<McapMessageMeta<ros_messages::nav_msgs::Odometry>>>,
    pub visualization_marker_messages:
        HashMap<ChannelTopic, Vec<McapMessageMeta<ros_messages::visualization_msgs::Marker>>>,
    pub visualization_marker_array_messages:
        HashMap<ChannelTopic, Vec<McapMessageMeta<ros_messages::visualization_msgs::MarkerArray>>>,
}

impl McapMessagePage {
    pub fn from(messages: Vec<McapMessageMeta<mcap::Message>>) -> Result<Self, Error> {
        let mut page = Self::default();

        for current_message in messages {
            let current_channel_schema =
                current_message
                    .message
                    .channel
                    .schema
                    .clone()
                    .ok_or(ChannelWithoutSchema(
                        current_message.message.channel.topic.as_str().into(),
                    ))?;

            let ros_message_type = if let Ok(message_type) =
                RosMessageType::from_str(current_channel_schema.name.as_str())
            {
                message_type
            } else {
                warn!("Unknown message type {}", current_channel_schema.name);
                continue;
            };

            match ros_message_type {
                RosMessageType::SensorMessagesImu => {
                    page.imu_messages
                        .entry(current_message.channel_topic.clone())
                        .or_default()
                        .push(convert_message(&current_message)?);
                }
                RosMessageType::SensorMessagesNavSatFix => {
                    page.nav_sat_fix_messages
                        .entry(current_message.channel_topic.clone())
                        .or_default()
                        .push(convert_message(&current_message)?);
                }
                RosMessageType::SensorMessagesPointCloud2 => {
                    page.point_cloud_messages
                        .entry(current_message.channel_topic.clone())
                        .or_default()
                        .push(convert_message(&current_message)?);
                }
                RosMessageType::SensorMessagesImage => {
                    page.image_messages
                        .entry(current_message.channel_topic.clone())
                        .or_default()
                        .push(convert_message(&current_message)?);
                }
                RosMessageType::SensorMessagesCameraInfo => {}
                RosMessageType::Tf2MessagesTFMessage => {
                    page.tf_messages
                        .entry(current_message.channel_topic.clone())
                        .or_default()
                        .push(convert_message(&current_message)?);
                }
                RosMessageType::NavMessagesOdometry => {
                    page.odometry_messages
                        .entry(current_message.channel_topic.clone())
                        .or_default()
                        .push(convert_message(&current_message)?);
                }
                RosMessageType::VisualizationMessagesMarker => {
                    page.visualization_marker_messages
                        .entry(current_message.channel_topic.clone())
                        .or_default()
                        .push(convert_message(&current_message)?);
                }
                RosMessageType::VisualizationMessagesMarkerArray => {
                    page.visualization_marker_array_messages
                        .entry(current_message.channel_topic.clone())
                        .or_default()
                        .push(convert_message(&current_message)?);
                }
            }
        }

        Ok(page)
    }

    pub fn get_channel_topics(&self) -> HashSet<ChannelTopic> {
        self.imu_messages
            .keys()
            .chain(self.nav_sat_fix_messages.keys())
            .chain(self.point_cloud_messages.keys())
            .chain(self.image_messages.keys())
            .chain(self.tf_messages.keys())
            .chain(self.odometry_messages.keys())
            .chain(self.visualization_marker_messages.keys())
            .chain(self.visualization_marker_array_messages.keys())
            .cloned()
            .collect()
    }
}

fn convert_message<T>(message: &McapMessageMeta<mcap::Message>) -> Result<McapMessageMeta<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    let deserialized_message_data = cdr::deserialize::<T>(&message.message.data[..])?;

    let result_message = McapMessageMeta::new(
        message.file_name.clone(),
        message.channel_topic.clone(),
        message.chunk_id,
        message.message_id,
        message.log_date_time,
        message.publish_date_time,
        deserialized_message_data,
    );
    Ok(result_message)
}

impl McapMessagePage {
    pub fn combine(pages: Vec<McapMessagePage>) -> Self {
        pages.into_iter().fold(Self::default(), |mut acc, page| {
            acc.imu_messages = merge_hashmaps(acc.imu_messages, page.imu_messages);
            acc.nav_sat_fix_messages =
                merge_hashmaps(acc.nav_sat_fix_messages, page.nav_sat_fix_messages);
            acc.point_cloud_messages =
                merge_hashmaps(acc.point_cloud_messages, page.point_cloud_messages);
            acc.image_messages = merge_hashmaps(acc.image_messages, page.image_messages);
            acc.tf_messages = merge_hashmaps(acc.tf_messages, page.tf_messages);
            acc.odometry_messages = merge_hashmaps(acc.odometry_messages, page.odometry_messages);
            acc.visualization_marker_messages = merge_hashmaps(
                acc.visualization_marker_messages,
                page.visualization_marker_messages,
            );
            acc.visualization_marker_array_messages = merge_hashmaps(
                acc.visualization_marker_array_messages,
                page.visualization_marker_array_messages,
            );
            acc
        })
    }
}

fn merge_hashmaps<T: Clone>(
    mut base: HashMap<ChannelTopic, Vec<T>>,
    other: HashMap<ChannelTopic, Vec<T>>,
) -> HashMap<ChannelTopic, Vec<T>> {
    for (channel_id, mut messages) in other {
        base.entry(channel_id).or_default().append(&mut messages);
    }
    base
}

impl McapMessagePage {
    pub fn get_point_cloud_messages_combined(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        stop_date_time: &Option<DateTime<Utc>>,
        channel_topics: &Option<HashSet<ChannelTopic>>,
    ) -> Result<epoint::PointCloud, Error> {
        let point_cloud_messages =
            self.get_point_cloud_messages(start_date_time, stop_date_time, channel_topics)?;
        let point_clouds = point_cloud_messages
            .into_par_iter()
            .map(|x| x.message)
            .collect();

        let mut merged_point_cloud = epoint::transform::merge(point_clouds)?;
        merged_point_cloud.reference_frames = self.get_all_reference_frames()?;
        Ok(merged_point_cloud)
    }

    pub fn get_point_cloud_messages(
        &self,
        start_date_time: &Option<DateTime<Utc>>,
        stop_date_time: &Option<DateTime<Utc>>,
        channel_topics: &Option<HashSet<ChannelTopic>>,
    ) -> Result<Vec<McapMessageMeta<epoint::PointCloud>>, Error> {
        let mut messages = if let Some(channel_topics) = channel_topics {
            channel_topics
                .iter()
                .map(|x| {
                    self.point_cloud_messages.get(x).ok_or(ChannelDoesNotHold(
                        x.clone(),
                        RosMessageType::SensorMessagesPointCloud2,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
        } else {
            self.point_cloud_messages
                .values()
                .flat_map(|x| x.iter())
                .collect()
        };

        if let Some(start_date_time) = start_date_time {
            messages.retain(|x| *start_date_time <= x.log_date_time);
        }

        if let Some(stop_date_time) = stop_date_time {
            messages.retain(|x| x.log_date_time < *stop_date_time);
        }

        let point_clouds = messages
            .into_iter()
            .map(|x| {
                let point_cloud: epoint::PointCloud = x.message.clone().into();
                McapMessageMeta::new(
                    x.file_name.clone(),
                    x.channel_topic.clone(),
                    x.chunk_id,
                    x.message_id,
                    x.log_date_time,
                    x.publish_date_time,
                    point_cloud,
                )
            })
            .collect();

        Ok(point_clouds)
    }

    pub fn get_point_cloud_of_channel(
        &self,
        channel_topic: &ChannelTopic,
    ) -> Result<epoint::PointCloud, Error> {
        let point_clouds: Vec<epoint::PointCloud> = self
            .point_cloud_messages
            .get(channel_topic)
            .ok_or(ChannelDoesNotHold(
                channel_topic.clone(),
                RosMessageType::SensorMessagesPointCloud2,
            ))?
            .into_par_iter()
            .map(|x| {
                let point_cloud: epoint::PointCloud = x.message.clone().into();
                /*let message_id = vec![x.header. as u32; point_cloud.size()];
                point_cloud
                    .point_data
                    .add_u32_column(RosMessageId.as_str(), message_id)
                    .expect("Adding the message_id column should work");*/
                point_cloud
            })
            .collect();

        let merged_point_cloud = epoint::transform::merge(point_clouds)?;
        Ok(merged_point_cloud)
    }

    pub fn get_all_images(&self) -> Result<eimage::ImageCollection, Error> {
        let image_series: HashMap<FrameId, ImageSeries> = self
            .image_messages
            .keys()
            .map(|x| {
                // TODO: this should be the actual frame id and not the channel's topic id
                let frame_id: ecoord::FrameId = x.to_string().into();
                self.get_images_of_channel(x)
                    .map(|series| (frame_id, series))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let reference_frames = self.get_all_reference_frames()?;
        let image_collection = eimage::ImageCollection::new(image_series, reference_frames)?;
        Ok(image_collection)
    }

    pub fn get_images_of_channel(
        &self,
        channel_topic: &ChannelTopic,
    ) -> Result<eimage::ImageSeries, Error> {
        let images: Vec<eimage::Image> = self
            .image_messages
            .get(channel_topic)
            .ok_or(ChannelDoesNotHold(
                channel_topic.clone(),
                RosMessageType::SensorMessagesImage,
            ))?
            .into_par_iter()
            .map(|x| {
                let image: eimage::Image = x.message.clone().into();
                image
            })
            .collect();

        let image_series = eimage::ImageSeries::new(images)?;
        Ok(image_series)
    }

    pub fn get_all_reference_frames(&self) -> Result<ecoord::ReferenceFrames, Error> {
        let mut transforms: HashMap<(ecoord::ChannelId, TransformId), Vec<ecoord::Transform>> =
            HashMap::new();
        let frame_info: HashMap<FrameId, ecoord::FrameInfo> = HashMap::new();
        let channel_info: HashMap<ecoord::ChannelId, ecoord::ChannelInfo> = HashMap::new();
        let transform_info: HashMap<TransformId, TransformInfo> = HashMap::new();

        for (current_channel_id, current_messages) in &self.tf_messages {
            let tf_messages: Vec<geometry_msgs::TransformStamped> = current_messages
                .iter()
                .flat_map(|x| x.message.transforms.clone())
                .collect();
            let current_channel_id: ecoord::ChannelId = self
                .get_channel_topics()
                .get(current_channel_id)
                .unwrap()
                .clone()
                .to_string()
                .into();
            for current_tf_message in tf_messages.into_iter() {
                let transform: Transform = (&current_tf_message).into();

                let current_frame_id: FrameId = current_tf_message.header.frame_id.into();
                let current_child_frame_id: FrameId = current_tf_message.child_frame_id.into();
                let current_transform_id =
                    TransformId::new(current_frame_id, current_child_frame_id);

                transforms
                    .entry((current_channel_id.clone(), current_transform_id))
                    .or_default()
                    .push(transform);
            }
        }

        transforms.iter_mut().for_each(|(_, transforms_vec)| {
            transforms_vec.sort_by_key(|transform| transform.timestamp);
        });

        let merged_reference_frames =
            ecoord::ReferenceFrames::new(transforms, frame_info, channel_info, transform_info)?;
        Ok(merged_reference_frames)
    }
}
