use cdr::{CdrBe, Infinite};
use chrono::{DateTime, TimeZone, Utc};
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt;

use diesel::dsl::{max, min};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::bagfile::models;
use crate::bagfile::models::{Message, NewMessage, NewTopic};
use crate::bagfile::schema;

use crate::ros_messages::tf2_msgs::TFMessage;
use crate::ros_messages::{geometry_msgs, MessageType, RosMessageType, Time};

use crate::ros_messages::visualization_msgs;
use crate::topics::qos_profile::QualityOfServiceProfile;
use crate::topics::topic::{TopicMetadata, TopicMetadataWithName, TopicSerializationFormat};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use ecoord::{ChannelId, FrameId, Transform, TransformId, TransformInfo};
use epoint::PointCloud;

use crate::error::Error;
use crate::ros_messages::sensor_msgs::PointCloud2;
use serde::Serialize;
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
    pub(crate) connection: SqliteConnection,
}

impl RosbagFile {
    pub fn new(
        directory_path: impl AsRef<Path>,
        filename_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let absolute_path = directory_path
            .as_ref()
            .to_owned()
            .join(filename_path.as_ref());

        let is_new_rosbag = !absolute_path.exists();

        let manager: ConnectionManager<SqliteConnection> =
            diesel::r2d2::ConnectionManager::new(absolute_path.to_str().unwrap());
        let connection_pool = diesel::r2d2::Pool::builder()
            .max_size(15)
            .build(manager)
            .unwrap();

        let mut connection = SqliteConnection::establish(absolute_path.to_str().unwrap())?;

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
            connection,
        })
    }

    pub fn create_topic(&self, name: &String, metadata: &TopicMetadata) {
        if self.contains_topic(name) {
            warn!(
                "Cannot create topic with name {}, since it already exists.",
                name
            );
            // TODO: check whether metadata is the same
            return;
        }
        let mut connection = self.connection_pool.get().unwrap();

        let name_with_metadata: TopicMetadataWithName =
            TopicMetadataWithName::new(name.clone(), metadata.clone());
        let new_topic: NewTopic = NewTopic::from(&name_with_metadata);

        let _rows_inserted = diesel::insert_into(schema::topics::dsl::topics)
            .values(&new_topic)
            .execute(&mut connection);
    }

    pub fn get_message_count(&self) -> u64 {
        let mut connection = self.connection_pool.get().unwrap();
        let count: i64 = schema::messages::dsl::messages
            .count()
            .get_result(&mut connection)
            .unwrap();
        count as u64
    }

    pub fn get_visualization_markers(&self, topic_name: &String) {
        let mut connection = self.connection_pool.get().unwrap();

        let topic_id: i32 = self.get_topic_id(topic_name).unwrap().into();
        let messages: Vec<Message> = schema::messages::dsl::messages
            .filter(schema::messages::topic_id.eq_all(topic_id))
            //.limit(10)
            .load::<Message>(&mut connection)
            .expect("Error loading messages");

        let _d: Vec<visualization_msgs::Marker> = messages
            .iter()
            .map(|m| cdr::deserialize::<visualization_msgs::Marker>(&m.data).unwrap())
            .collect();

        //let decoded_message =
        //    cdr::deserialize::<visualization_msgs::Marker>(&message_entry.data[..]).unwrap();

        info!("found a lot of messages: {}", messages.len());
    }

    pub fn get_all_topic_names(&self) -> HashSet<String> {
        let mut connection = self.connection_pool.get().unwrap();

        let topic_res: Vec<models::Topic> = schema::topics::dsl::topics
            .load::<models::Topic>(&mut connection)
            .expect("Error loading messages");

        topic_res.iter().map(|t| t.name.clone()).collect()
    }

    pub fn get_all_topics(&self) -> HashMap<String, TopicMetadata> {
        let mut connection = self.connection_pool.get().unwrap();

        let topic_res: Vec<models::Topic> = schema::topics::dsl::topics
            .load::<models::Topic>(&mut connection)
            .expect("Error loading messages");

        let all_topics: Vec<TopicMetadataWithName> = topic_res.iter().map(|t| t.into()).collect();

        all_topics
            .iter()
            .map(|t| (t.name.clone(), t.topic.clone()))
            .collect()
    }

    pub fn get_topic_names_of_type(&self, message_type: RosMessageType) -> HashSet<String> {
        let mut connection = self.connection_pool.get().unwrap();

        let message_type: &str = message_type.as_str();
        let topic_res: Vec<models::Topic> = schema::topics::dsl::topics
            .filter(schema::topics::type_.eq_all(message_type))
            .load::<models::Topic>(&mut connection)
            .expect("Error loading messages");

        topic_res.iter().map(|t| t.name.clone()).collect()
    }

    pub fn get_start_time(&self) -> Option<u64> {
        let mut connection = self.connection_pool.get().unwrap();

        let time: Option<i64> = schema::messages::dsl::messages
            .select(min(schema::messages::timestamp))
            .first(&mut connection)
            .unwrap();

        time.map(|t| t as u64)
    }

    pub fn get_stop_time(&self) -> Option<u64> {
        let mut connection = self.connection_pool.get().unwrap();

        let time: Option<i64> = schema::messages::dsl::messages
            .select(max(schema::messages::timestamp))
            .first(&mut connection)
            .unwrap();

        time.map(|t| t as u64)
    }

    pub fn get_duration(&self) -> Option<u64> {
        let start_time = self.get_start_time()?;
        let stop_time = self.get_stop_time()?;

        Some(stop_time - start_time)
    }

    pub fn get_all_message_timestamps(&self) -> Vec<(i32, i64)> {
        let mut connection = self.connection_pool.get().unwrap();
        let times: Vec<(i32, i64)> = schema::messages::dsl::messages
            .select((schema::messages::id, schema::messages::timestamp))
            .load(&mut connection)
            .expect("Error loading messages");

        times
    }

    pub fn set_qos_profile(
        &self,
        topic_name: &String,
        offered_qos_profile: &QualityOfServiceProfile,
    ) {
        let profiles = vec![offered_qos_profile];
        let encoded_profiles = serde_yaml::to_string(&profiles).unwrap();

        let mut connection = self.connection_pool.get().unwrap();
        let _updated_row = diesel::update(
            schema::topics::dsl::topics.filter(schema::topics::name.eq_all(topic_name)),
        )
        .set(schema::topics::offered_qos_profiles.eq(encoded_profiles))
        .execute(&mut connection);
    }

    //pub fn get_all_messages_of_type(&mut self, message_type: RosMessageType) -> Vec<i32> {
    //}

    pub fn contains_topic(&self, topic_name: &String) -> bool {
        let mut connection = self.connection_pool.get().unwrap();

        let number_of_topics: Result<i64, diesel::result::Error> = schema::topics::dsl::topics
            .filter(schema::topics::name.eq_all(topic_name))
            .count()
            .get_result(&mut connection);
        //.load::<models::Topic>(&mut connection)
        //.expect("Error loading messages");

        // info!("top: {:?}", number_of_topics);
        number_of_topics.map(|x| x > 0).unwrap_or(false)
    }

    fn get_all_message_ids_of_topic(&self, topic_name: &String) -> Vec<i32> {
        let mut connection = self.connection_pool.get().unwrap();

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

        messages.iter().map(|m| m.id).collect()
    }

    pub fn get_all_point_cloud(
        &self,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<epoint::PointCloud, Error> {
        let topic_names = self.get_topic_names_of_type(RosMessageType::SensorMessagesPointCloud2);

        let point_clouds: Vec<PointCloud> = topic_names
            .iter()
            .map(|n| self.get_point_cloud(n, start_time, stop_time))
            .collect();
        let merged_point_cloud = epoint::transform::merge(point_clouds)?;

        Ok(merged_point_cloud)
    }

    pub fn get_point_cloud(
        &self,
        topic_name: &String,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> epoint::PointCloud {
        let mut connection = self.connection_pool.get().unwrap();

        let topic_id: i32 = self.get_topic_id(topic_name).unwrap().into();
        let mut messages_query = schema::messages::dsl::messages
            .filter(schema::messages::topic_id.eq_all(topic_id))
            .into_boxed();

        // filter based on times
        if let Some(start_time) = start_time {
            messages_query =
                messages_query.filter(schema::messages::timestamp.ge(start_time.timestamp_nanos()));
        }
        if let Some(stop_time) = stop_time {
            messages_query =
                messages_query.filter(schema::messages::timestamp.le(stop_time.timestamp_nanos()));
        }
        let messages_query_results = messages_query.load::<Message>(&mut connection).unwrap();

        let decoded_messages: Vec<PointCloud2> = messages_query_results
            .iter()
            .map(|m| cdr::deserialize::<PointCloud2>(&m.data[..]).unwrap())
            .collect();

        let point_clouds: Vec<epoint::PointCloud> =
            decoded_messages.into_iter().map(|m| m.into()).collect();

        epoint::transform::merge(point_clouds).unwrap()
    }

    fn get_transform_messages_of_topic(
        &self,
        topic_name: &String,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Vec<TFMessage> {
        let mut connection = self.connection_pool.get().unwrap();

        let topic_id: i32 = self.get_topic_id(topic_name).unwrap().into();
        let mut messages_query = schema::messages::dsl::messages
            .filter(schema::messages::topic_id.eq_all(topic_id))
            .into_boxed();

        // filter based on times
        if let Some(start_time) = start_time {
            messages_query =
                messages_query.filter(schema::messages::timestamp.ge(start_time.timestamp_nanos()));
        }
        if let Some(stop_time) = stop_time {
            messages_query =
                messages_query.filter(schema::messages::timestamp.le(stop_time.timestamp_nanos()));
        }

        let messages_query_results: Vec<Message> = messages_query
            //.limit(10)
            .load::<Message>(&mut connection)
            .expect("Error loading messages");

        let decoded: Vec<TFMessage> = messages_query_results
            .iter()
            .map(|m| cdr::deserialize::<TFMessage>(&m.data[..]).unwrap())
            .collect();
        decoded
    }

    pub fn get_transforms(
        &self,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<ecoord::ReferenceFrames, Error> {
        let mut transforms: HashMap<(ChannelId, TransformId), Vec<ecoord::Transform>> =
            HashMap::new();
        let frame_info: HashMap<FrameId, ecoord::FrameInfo> = HashMap::new();
        let channel_info: HashMap<ChannelId, ecoord::ChannelInfo> = HashMap::new();
        let transform_info: HashMap<TransformId, TransformInfo> = HashMap::new();

        for current_topic_name in self.get_topic_names_of_type(RosMessageType::Tf2MessagesTFMessage)
        {
            let tf_messages: Vec<geometry_msgs::TransformStamped> = self
                .get_transform_messages_of_topic(&current_topic_name, start_time, stop_time)
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
            ecoord::ReferenceFrames::new(transforms, frame_info, channel_info, transform_info);

        Ok(efr)
    }

    /// Returns the time of a message
    ///
    fn get_time(&self, message_id: MessageId) -> DateTime<Utc> {
        let mut connection = self.connection_pool.get().unwrap();

        let message_entry: Message = schema::messages::dsl::messages
            .find::<i32>(message_id.into())
            .first(&mut connection)
            .unwrap();

        Utc.timestamp_nanos(message_entry.timestamp)
    }

    fn get_topic_metadata(&self, topic_name: &String) -> Result<(TopicId, TopicMetadata), ()> {
        let mut connection = self.connection_pool.get().unwrap();

        let selected_row: models::Topic = schema::topics::dsl::topics
            .filter(schema::topics::name.eq_all(topic_name))
            .first(&mut connection)
            .ok()
            .ok_or(())?;

        // let a: Topic = selected_row?;
        let id: TopicId = selected_row.id.into();
        let metadata: TopicMetadata = selected_row.borrow().into();

        Ok((id, metadata))
    }

    fn get_topic_type(&self, topic_id: TopicId) -> Result<RosMessageType, ()> {
        let mut connection = self.connection_pool.get().unwrap();

        let selected_row: models::Topic = schema::topics::dsl::topics
            .find::<i32>(topic_id.into())
            .first(&mut connection)
            .unwrap();

        RosMessageType::from_str(&selected_row.type_)
    }

    fn get_topic_name(&self, topic_id: TopicId) -> String {
        let mut connection = self.connection_pool.get().unwrap();

        let selected_row: models::Topic = schema::topics::dsl::topics
            .find::<i32>(topic_id.into())
            .first(&mut connection)
            .unwrap();

        selected_row.name
    }

    fn get_topic_id(&self, topic_name: &String) -> Option<TopicId> {
        let mut connection = self.connection_pool.get().unwrap();

        let selected_row: Option<models::Topic> = schema::topics::dsl::topics
            .filter(schema::topics::name.eq_all(topic_name))
            .first(&mut connection)
            .ok();

        selected_row.map(|r| TopicId::from(r.id))
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

    pub fn append_transform_messages(&self, reference_frames: ecoord::ReferenceFrames) {
        let mut connection = self.connection_pool.get().unwrap();

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
            let topic_id = self.get_topic_id(&current_channel_id.clone().into());

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

                println!(); // TODO
            }
        }
    }

    pub fn append_message(
        &self,
        topic_name: &String,
        ros_message: &(impl MessageType + Time + Serialize),
    ) {
        info!("Message type: {:?}", ros_message.ros_message_type());
        let (topic_id, topic_metadata) = self.get_topic_metadata(topic_name).unwrap();

        if *ros_message.ros_message_type() != topic_metadata.message_type {
            warn!("problemo");
            return;
        }

        let encoded_message =
            cdr::serialize::<_, _, cdr::CdrBe>(&ros_message, cdr::Infinite).unwrap();
        info!("test: {:?}", &encoded_message);
        let timestamp_message = ros_message.time();
        let t: chrono::DateTime<Utc> = (*timestamp_message).into();
        let new_message = NewMessage {
            topic_id: topic_id.into(),
            timestamp: t.timestamp_nanos(),
            data: encoded_message,
        };

        let mut connection = self.connection_pool.get().unwrap();
        let _rows_inserted = diesel::insert_into(schema::messages::dsl::messages)
            .values(&new_message)
            .execute(&mut connection);

        //info!("a: {:?}", a);
        //self.append_topic();
    }
}
