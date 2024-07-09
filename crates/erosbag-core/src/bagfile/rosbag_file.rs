use crate::bagfile::error::BagFileError;
use crate::bagfile::error::BagFileError::{
    ContainsNoMessages, ContainsNoPointCloud2Topics, InvalidMessageType, RequestedTimeInvalid,
    TopicAlreadyExists, TopicWithIdDoesNotExist, TopicWithNameDoesNotExist, UnsupportedMessageType,
};
use crate::bagfile::models;
use crate::bagfile::models::{Message, NewMessage, NewTopic};
use crate::bagfile::schema;
use crate::ros_messages::sensor_msgs::{Image, NavSatFix, PointCloud2};
use crate::ros_messages::tf2_msgs::TFMessage;
use crate::ros_messages::visualization_msgs;
use crate::ros_messages::{geometry_msgs, MessageType, RosMessageType, Time};
use crate::topics::qos_profile::QualityOfServiceProfile;
use crate::topics::topic::{TopicMetadata, TopicMetadataWithName, TopicSerializationFormat};
use crate::RosPointCloudColumnType::RosMessageId;
use cdr::{CdrBe, Infinite};
use chrono::{DateTime, TimeZone, Utc};
use diesel::dsl::{max, min};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use ecoord::{ChannelId, FrameId, Transform, TransformId, TransformInfo};
use epoint::PointCloud;

use serde::Serialize;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{info, warn};

/// Dedicated type for an identifier of a topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TopicId(i32);

impl fmt::Display for TopicId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TopicId> for i32 {
    fn from(item: TopicId) -> Self {
        item.0
    }
}

impl From<i32> for TopicId {
    fn from(item: i32) -> Self {
        Self(item)
    }
}

/// Dedicated type for an identifier of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MessageId(i32);

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<MessageId> for i32 {
    fn from(item: MessageId) -> Self {
        item.0
    }
}

impl From<i32> for MessageId {
    fn from(item: i32) -> Self {
        Self(item)
    }
}

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct RosbagFile {
    pub(crate) filename_path: PathBuf,
    pub(crate) connection_pool: DbPool,
}

impl RosbagFile {
    pub fn new(
        directory_path: impl AsRef<Path>,
        filename_path: impl AsRef<Path>,
    ) -> Result<Self, BagFileError> {
        let absolute_path = directory_path
            .as_ref()
            .to_owned()
            .join(filename_path.as_ref());

        let is_new_rosbag = !absolute_path.exists();

        let manager: ConnectionManager<SqliteConnection> =
            diesel::r2d2::ConnectionManager::new(absolute_path.to_str().unwrap());
        let connection_pool = diesel::r2d2::Pool::builder().max_size(15).build(manager)?;

        let mut connection = connection_pool.get()?;

        if is_new_rosbag {
            // TODO: run migration scripts only when opening a new bag
            pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
            connection
                .run_pending_migrations(MIGRATIONS)
                .map_err(|e| warn!("{}", e))
                .ok();
        }

        Ok(Self {
            filename_path: filename_path.as_ref().to_owned(),
            connection_pool,
        })
    }

    pub fn create_topic(
        &self,
        name: &String,
        metadata: &TopicMetadata,
    ) -> Result<(), BagFileError> {
        if self.contains_topic(name)? {
            // TODO: check whether metadata is the same
            // TODO: better way of handling str
            return Err(TopicAlreadyExists(name.clone()));
        }
        let mut connection = self.connection_pool.get()?;

        let name_with_metadata: TopicMetadataWithName =
            TopicMetadataWithName::new(name.clone(), metadata.clone());
        let new_topic: NewTopic = NewTopic::from(&name_with_metadata);

        let _rows_inserted = diesel::insert_into(schema::topics::dsl::topics)
            .values(&new_topic)
            .execute(&mut connection);

        Ok(())
    }

    pub fn get_message_count(&self) -> Result<u64, BagFileError> {
        let mut connection = self.connection_pool.get()?;
        let count: i64 = schema::messages::dsl::messages
            .count()
            .get_result(&mut connection)?;
        Ok(count as u64)
    }

    pub fn get_visualization_markers(&self, topic_name: &String) -> Result<(), BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let topic_id: i32 = self.get_topic_id(topic_name)?.into();
        let messages: Vec<Message> = schema::messages::dsl::messages
            .filter(schema::messages::topic_id.eq_all(topic_id))
            //.limit(10)
            .load::<Message>(&mut connection)
            .expect("Error loading messages");

        let _d: Vec<visualization_msgs::Marker> = messages
            .iter()
            .map(|m| cdr::deserialize::<visualization_msgs::Marker>(&m.data))
            .collect::<Result<Vec<_>, _>>()?;

        //let decoded_message =
        //    cdr::deserialize::<visualization_msgs::Marker>(&message_entry.data[..]).unwrap();

        info!("found a lot of messages: {}", messages.len());
        Ok(())
    }

    pub fn get_all_topic_names(&self) -> Result<HashSet<String>, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let topic_res: Vec<models::Topic> = schema::topics::dsl::topics
            .load::<models::Topic>(&mut connection)
            .expect("Error loading messages");

        let topic_names = topic_res.iter().map(|t| t.name.clone()).collect();
        Ok(topic_names)
    }

    pub fn get_all_topics(&self) -> Result<HashMap<String, TopicMetadata>, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let topic_res: Vec<models::Topic> = schema::topics::dsl::topics
            .load::<models::Topic>(&mut connection)
            .expect("Error loading messages");

        let all_topics: Vec<TopicMetadataWithName> = topic_res.iter().map(|t| t.into()).collect();

        Ok(all_topics
            .iter()
            .map(|t| (t.name.clone(), t.topic.clone()))
            .collect())
    }

    pub fn get_topic_names_of_type(
        &self,
        message_type: RosMessageType,
    ) -> Result<HashSet<String>, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let message_type: &str = message_type.as_str();
        let topic_res: Vec<models::Topic> = schema::topics::dsl::topics
            .filter(schema::topics::type_.eq_all(message_type))
            .load::<models::Topic>(&mut connection)
            .expect("Error loading messages");

        let topic_names = topic_res.iter().map(|t| t.name.clone()).collect();
        Ok(topic_names)
    }

    pub fn get_start_time(&self) -> Result<Option<u64>, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let time_result: Option<i64> = schema::messages::dsl::messages
            .select(min(schema::messages::timestamp))
            .first(&mut connection)?;

        Ok(time_result.map(|t| t as u64))
    }

    pub fn get_start_date_time(&self) -> Result<Option<DateTime<Utc>>, BagFileError> {
        let start_time = self.get_start_time()?;
        Ok(start_time.map(|t| Utc.timestamp_nanos(t as i64)))
    }

    pub fn get_stop_time(&self) -> Result<Option<u64>, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let time_result: Option<i64> = schema::messages::dsl::messages
            .select(max(schema::messages::timestamp))
            .first(&mut connection)?;

        Ok(time_result.map(|t| t as u64))
    }

    pub fn get_stop_date_time(&self) -> Result<Option<DateTime<Utc>>, BagFileError> {
        let stop_time = self.get_stop_time()?;
        Ok(stop_time.map(|t| Utc.timestamp_nanos(t as i64)))
    }

    pub fn get_duration(&self) -> Result<Option<u64>, BagFileError> {
        let start_time = self.get_start_time()?;
        let stop_time = self.get_stop_time()?;

        if start_time.is_none() && stop_time.is_none() {
            return Ok(None);
        }
        if start_time.is_some() || stop_time.is_some() {
            return Ok(Some(0));
        }

        Ok(Some(stop_time.unwrap() - start_time.unwrap()))
    }

    pub fn get_all_message_timestamps(&self) -> Result<Vec<(i32, i64)>, BagFileError> {
        let mut connection = self.connection_pool.get()?;
        let times: Vec<(i32, i64)> = schema::messages::dsl::messages
            .select((schema::messages::id, schema::messages::timestamp))
            .load(&mut connection)
            .expect("Error loading messages");

        Ok(times)
    }

    pub fn set_qos_profile(
        &self,
        topic_name: &String,
        offered_qos_profile: &QualityOfServiceProfile,
    ) -> Result<(), BagFileError> {
        let profiles = vec![offered_qos_profile];
        let encoded_profiles = serde_yaml::to_string(&profiles).unwrap();

        let mut connection = self.connection_pool.get()?;
        let _updated_row = diesel::update(
            schema::topics::dsl::topics.filter(schema::topics::name.eq_all(topic_name)),
        )
        .set(schema::topics::offered_qos_profiles.eq(encoded_profiles))
        .execute(&mut connection);

        Ok(())
    }

    pub fn contains_topic(&self, topic_name: &String) -> Result<bool, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let number_of_topics: i64 = schema::topics::dsl::topics
            .filter(schema::topics::name.eq_all(topic_name))
            .count()
            .get_result(&mut connection)?;
        //.load::<models::Topic>(&mut connection)
        //.expect("Error loading messages");

        // info!("top: {:?}", number_of_topics);
        Ok(number_of_topics > 0)
    }

    fn get_all_message_ids_of_topic(&self, topic_name: &String) -> Result<Vec<i32>, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let topic_entry: Vec<models::Topic> = schema::topics::dsl::topics
            .filter(schema::topics::name.eq_all(topic_name))
            .load::<models::Topic>(&mut connection)
            .expect("Error loading messages");
        let topic_id = topic_entry.first().unwrap().id;

        let messages: Vec<Message> = schema::messages::dsl::messages
            .filter(schema::messages::topic_id.eq_all(topic_id))
            //.limit(10)
            .load::<Message>(&mut connection)
            .expect("Error loading messages");

        Ok(messages.iter().map(|m| m.id).collect())
    }

    fn get_all_messages_from_topic(
        &self,
        topic_id: TopicId,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>, BagFileError> {
        let mut connection = self.connection_pool.get()?;
        let topic_id_num: i32 = topic_id.into();

        let mut messages_query = schema::messages::dsl::messages
            .filter(schema::messages::topic_id.eq_all(topic_id_num))
            .into_boxed();

        // filter based on times
        if let Some(start_time) = start_time {
            messages_query =
                messages_query.filter(schema::messages::timestamp.ge(start_time.timestamp_nanos()));
        }
        if let Some(stop_time) = stop_time {
            messages_query =
                messages_query.filter(schema::messages::timestamp.lt(stop_time.timestamp_nanos()));
        }
        let messages_query_results: Vec<Message> =
            messages_query.load::<Message>(&mut connection)?;

        Ok(messages_query_results)
    }

    pub fn get_all_point_cloud(
        &self,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<epoint::PointCloud, BagFileError> {
        let topic_names =
            self.get_topic_names_of_type(RosMessageType::SensorMessagesPointCloud2)?;
        if topic_names.is_empty() {
            return Err(ContainsNoPointCloud2Topics);
        }

        let point_clouds: Vec<PointCloud> = topic_names
            .iter()
            .map(|n| self.get_point_cloud(n, start_time, stop_time))
            .collect::<Result<Vec<PointCloud>, _>>()?;

        let merged_point_cloud = epoint::transform::merge(point_clouds)?;
        Ok(merged_point_cloud)
    }

    /// Returns a topic's point cloud with start_time <= point cloud time < stop_time.
    pub fn get_point_cloud(
        &self,
        topic_name: &String,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<epoint::PointCloud, BagFileError> {
        let bag_start_time = self.get_start_date_time()?.ok_or(ContainsNoMessages)?;
        let bag_stop_time = self.get_stop_date_time()?.ok_or(ContainsNoMessages)?;

        // check whether times are
        if let Some(start_time) = start_time {
            if start_time < &bag_start_time {
                return Err(RequestedTimeInvalid {
                    requested_time: *start_time,
                    bag_start_time,
                    bag_stop_time,
                });
            }
        }
        if let Some(stop_time) = stop_time {
            if &bag_stop_time < stop_time {
                return Err(RequestedTimeInvalid {
                    requested_time: *stop_time,
                    bag_start_time,
                    bag_stop_time,
                });
            }
        }

        let topic_id: i32 = self.get_topic_id(topic_name).unwrap().into();
        let messages_query_results =
            self.get_all_messages_from_topic(topic_id.into(), start_time, stop_time)?;

        let decoded_messages: HashMap<i32, PointCloud2> = messages_query_results
            .into_iter()
            .map(|m| (m.id, cdr::deserialize::<PointCloud2>(&m.data[..]).unwrap()))
            .collect();

        let point_clouds: Vec<epoint::PointCloud> = decoded_messages
            .into_iter()
            .map(|(id, point_cloud)| {
                let mut point_cloud: epoint::PointCloud = point_cloud.into();
                let message_id = vec![id as u32; point_cloud.size()];
                point_cloud
                    .point_data
                    .add_u32_column(RosMessageId.as_str(), message_id)
                    .expect("Adding the message_id column should work");
                point_cloud
            })
            .collect();

        let merged_point_cloud = epoint::transform::merge(point_clouds)?;
        Ok(merged_point_cloud)
    }

    pub fn get_all_images(
        &self,
        _start_time: &Option<DateTime<Utc>>,
        _stop_time: &Option<DateTime<Utc>>,
    ) {
    }

    pub fn get_images(
        &self,
        topic_name: &String,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<eimage::ImageSeries, BagFileError> {
        let bag_start_time = self.get_start_date_time()?.ok_or(ContainsNoMessages)?;
        let bag_stop_time = self.get_stop_date_time()?.ok_or(ContainsNoMessages)?;

        if let Some(start_time) = start_time {
            if *start_time < bag_start_time {
                return Err(RequestedTimeInvalid {
                    requested_time: *start_time,
                    bag_start_time,
                    bag_stop_time,
                });
            }
        }
        if let Some(stop_time) = stop_time {
            if bag_stop_time < *stop_time {
                return Err(RequestedTimeInvalid {
                    requested_time: *stop_time,
                    bag_start_time,
                    bag_stop_time,
                });
            }
        }

        let topic_id: i32 = self.get_topic_id(topic_name)?.into();
        let messages_query_results =
            self.get_all_messages_from_topic(topic_id.into(), start_time, stop_time)?;

        let images: Vec<eimage::Image> = messages_query_results
            .into_iter()
            .map(|m| cdr::deserialize::<Image>(&m.data[..]))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|i| i.into())
            .collect();
        /*let images: Vec<eimage::Image> = decoded_messages
        .values()
        .map(|i| i.clone().into())
        .collect();*/

        let image_series = eimage::ImageSeries::new(images)?;

        /*for (id, current_message) in &decoded_messages {
            println!("current message size: {:?}", current_message.data.len());

            let i: eimage::Image = current_message.clone().into();
        }

        let image_name: &str = &format!("fractal_{id}.png");
        let p = PathBuf::from(image_name);
        if !p.exists() {
            i.get_buffer().save(p).unwrap();
        }*/

        //println!("number of messages: {}", decoded_messages.len());
        Ok(image_series)
    }

    fn get_transform_messages_of_topic(
        &self,
        topic_name: &String,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<Vec<TFMessage>, BagFileError> {
        let topic_id: i32 = self.get_topic_id(topic_name)?.into();
        let messages_query_results =
            self.get_all_messages_from_topic(topic_id.into(), start_time, stop_time)?;

        let decoded: Vec<TFMessage> = messages_query_results
            .iter()
            .map(|m| cdr::deserialize::<TFMessage>(&m.data[..]).unwrap())
            .collect();
        Ok(decoded)
    }

    pub fn get_transforms(
        &self,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<ecoord::ReferenceFrames, BagFileError> {
        let mut transforms: HashMap<(ChannelId, TransformId), Vec<ecoord::Transform>> =
            HashMap::new();
        let frame_info: HashMap<FrameId, ecoord::FrameInfo> = HashMap::new();
        let channel_info: HashMap<ChannelId, ecoord::ChannelInfo> = HashMap::new();
        let transform_info: HashMap<TransformId, TransformInfo> = HashMap::new();

        for current_topic_name in
            self.get_topic_names_of_type(RosMessageType::Tf2MessagesTFMessage)?
        {
            let tf_messages: Vec<geometry_msgs::TransformStamped> = self
                .get_transform_messages_of_topic(&current_topic_name, start_time, stop_time)?
                .into_iter()
                .flat_map(|m| m.transforms)
                .collect();
            let current_channel_id: ChannelId = current_topic_name.into();
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

            // println!("channel {current_topic_name}");
        }

        let efr =
            ecoord::ReferenceFrames::new(transforms, frame_info, channel_info, transform_info)?;

        Ok(efr)
    }

    /// Returns the time of a message
    ///
    fn get_time(&self, message_id: MessageId) -> Result<DateTime<Utc>, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let message_entry: Message = schema::messages::dsl::messages
            .find::<i32>(message_id.into())
            .first(&mut connection)
            .unwrap();

        Ok(Utc.timestamp_nanos(message_entry.timestamp))
    }

    fn get_topic_metadata(
        &self,
        topic_name: &String,
    ) -> Result<(TopicId, TopicMetadata), BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let selected_row: models::Topic = schema::topics::dsl::topics
            .filter(schema::topics::name.eq_all(topic_name))
            .first(&mut connection)
            .ok()
            .ok_or(TopicWithNameDoesNotExist(topic_name.clone()))?;

        // let a: Topic = selected_row?;
        let id: TopicId = selected_row.id.into();
        let metadata: TopicMetadata = selected_row.borrow().into();

        Ok((id, metadata))
    }

    fn get_message_type(&self, topic_id: TopicId) -> Result<RosMessageType, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let selected_row: models::Topic = schema::topics::dsl::topics
            .find::<i32>(topic_id.into())
            .first(&mut connection)
            .map_err(|_| TopicWithIdDoesNotExist(topic_id.into()))?;

        let message_type =
            RosMessageType::from_str(&selected_row.type_).map_err(|_| UnsupportedMessageType())?;
        Ok(message_type)
    }

    fn get_topic_name(&self, topic_id: TopicId) -> Result<String, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let selected_row: models::Topic = schema::topics::dsl::topics
            .find::<i32>(topic_id.into())
            .first(&mut connection)
            .map_err(|_| TopicWithIdDoesNotExist(topic_id.into()))?;

        Ok(selected_row.name)
    }

    fn get_topic_id(&self, topic_name: &String) -> Result<TopicId, BagFileError> {
        let mut connection = self.connection_pool.get()?;

        let selected_row: Option<models::Topic> = schema::topics::dsl::topics
            .filter(schema::topics::name.eq_all(topic_name))
            .first(&mut connection)
            .ok();

        selected_row
            .map(|r| TopicId::from(r.id))
            .ok_or(TopicWithNameDoesNotExist(topic_name.clone()))
    }

    /*pub fn print_topics(&mut self) {
        let results = schema::topics::dsl::topics
            //.limit(5)
            .load::<Topic>(&mut self.connection)
            .expect("Error loading topics");

        info!("Displaying {} posts", results.len());
        for topic in results {
            //println!("{}", topic.id);
            info!("id: {}", topic.id);
            info!("name: {}", topic.name);
            info!("type: {}", topic.type_);
            info!("serialization_format: {}", topic.serialization_format);

            info!(" ");
        }
    }*/

    pub fn get_nav_sat_fix_messages_of_topic(
        &self,
        topic_name: &String,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<Vec<NavSatFix>, BagFileError> {
        let topic_id = self.get_topic_id(topic_name)?;
        let message_type = self.get_message_type(topic_id)?;

        if message_type != RosMessageType::SensorMessagesNavSatFix {
            return Err(InvalidMessageType(message_type.as_str()));
        }

        let messages_query_results =
            self.get_all_messages_from_topic(topic_id, start_time, stop_time)?;

        let decoded: Vec<NavSatFix> = messages_query_results
            .iter()
            .map(|m| cdr::deserialize::<NavSatFix>(&m.data[..]).unwrap())
            .collect();

        // TODO: better error handling
        // self.get_topic_type()
        Ok(decoded)
    }

    pub fn append_transform_messages(
        &self,
        reference_frames: ecoord::ReferenceFrames,
    ) -> Result<(), BagFileError> {
        let mut connection = self.connection_pool.get()?;

        /*let topic_res: Vec<String> = schema::topics::dsl::topics
            .load::<Topic>(&mut self.connection)
            .expect("Error loading messages")
            .iter()
            .map(|t| t.name.clone())
            .collect();
        warn!("ok");*/

        for ((current_channel_id, current_transform_id), current_transforms) in
            reference_frames.transforms()
        {
            let topic_id = self.get_topic_id(&current_channel_id.clone().into()).ok();

            let topic_id = match topic_id {
                Some(id) => id,
                None => {
                    let profiles = vec![QualityOfServiceProfile::new_for_static_tf_topic()];
                    let profiles = serde_yaml::to_string(&profiles).unwrap();

                    let new_topic = NewTopic {
                        name: current_channel_id.clone().into(),
                        type_: RosMessageType::Tf2MessagesTFMessage.as_str().to_string(),
                        serialization_format: TopicSerializationFormat::CDR.as_str().to_string(),
                        offered_qos_profiles: profiles,
                    };

                    // returning clause is only supported starting from v3.35 of SQLite
                    // see for SQLite: https://antonz.org/sqlite-3-35/
                    // see for diesel: https://github.com/diesel-rs/diesel/discussions/2684
                    diesel::insert_or_ignore_into(schema::topics::dsl::topics)
                        .values(&new_topic)
                        .execute(&mut connection)
                        .expect("TODO: panic message");

                    self.get_topic_id(&current_channel_id.clone().into())
                        .unwrap()
                }
            };

            for current_transform in current_transforms {
                let transform_stamped: geometry_msgs::TransformStamped =
                    (current_transform_id, current_transform).into();
                let tf_message: TFMessage = TFMessage {
                    transforms: vec![transform_stamped],
                };
                let encoded_message = cdr::serialize::<_, _, CdrBe>(&tf_message, Infinite).unwrap();

                let timestamp: i64 = current_transform.timestamp.timestamp_nanos();

                let new_message = NewMessage {
                    topic_id: topic_id.into(),
                    timestamp,
                    data: encoded_message,
                };

                let _rows_inserted = diesel::insert_into(schema::messages::dsl::messages)
                    .values(&new_message)
                    .execute(&mut connection);

                // TODO
                // println!();
            }
        }

        Ok(())
    }

    pub fn append_message(
        &self,
        topic_name: &String,
        ros_message: &(impl MessageType + Time + Serialize),
    ) -> Result<(), BagFileError> {
        info!("Message type: {:?}", ros_message.ros_message_type());
        let (topic_id, topic_metadata) = self.get_topic_metadata(topic_name)?;

        if *ros_message.ros_message_type() != topic_metadata.message_type {
            return Err(InvalidMessageType(ros_message.ros_message_type().as_str()));
        }

        let encoded_message = cdr::serialize::<_, _, cdr::CdrBe>(&ros_message, cdr::Infinite)?;
        info!("test: {:?}", &encoded_message);
        let timestamp_message = ros_message.time();
        let t: chrono::DateTime<Utc> = (*timestamp_message).into();
        let new_message = NewMessage {
            topic_id: topic_id.into(),
            timestamp: t.timestamp_nanos(),
            data: encoded_message,
        };

        let mut connection = self.connection_pool.get()?;
        let _rows_inserted = diesel::insert_into(schema::messages::dsl::messages)
            .values(&new_message)
            .execute(&mut connection);

        //info!("a: {:?}", a);
        //self.append_topic();
        Ok(())
    }
}
