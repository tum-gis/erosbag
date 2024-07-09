use crate::bagfile::models::Message;
use crate::bagfile::models::Topic;
use crate::bagfile::rosbag_file::RosbagFile;
use crate::bagfile::schema;

use crate::ros_messages::tf2_msgs;
use crate::ros_messages::tf2_msgs::TFMessage;
use crate::ros_messages::RosMessageType;

use crate::{Rosbag, SQLITE3_EXTENSION};

use diesel::prelude::*;
use diesel::SqliteConnection;
use ecoord::{InterpolationMethod, TransformInfo};
use std::collections::{HashMap, HashSet};
use std::fs;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct RosbagOpenOptions {
    read_write: bool,
    create_new: bool,
    transform_interpolation_method: HashMap<ecoord::TransformId, InterpolationMethod>,
}

impl Default for RosbagOpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl RosbagOpenOptions {
    pub fn new() -> Self {
        // let selected_transform_topics = HashSet::from([String::from("/tf")]);

        Self {
            read_write: false,
            create_new: false,
            transform_interpolation_method: HashMap::new(),
        }
    }

    pub fn read_write(&mut self, read_write: bool) -> &mut Self {
        self.read_write = read_write;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn with_transform_interpolation_method(
        mut self,
        transform_interpolation_method: HashMap<ecoord::TransformId, InterpolationMethod>,
    ) -> Self {
        self.transform_interpolation_method = transform_interpolation_method;
        self
    }

    pub fn open(&self, directory_path: impl AsRef<Path>) -> Result<Rosbag, Error> {
        if !self.read_write && !self.create_new {
            return Err(Error::InvalidInput);
        }

        if self.create_new {
            return self.create_new_rosbag(directory_path);
        }

        if self.read_write {
            return self.read_write_rosbag(directory_path);
        }

        Err(Error::InvalidInput)
    }

    fn create_new_rosbag(&self, directory_path: impl AsRef<Path>) -> Result<Rosbag, Error> {
        if self.create_new {
            fs::create_dir_all(directory_path.as_ref()).unwrap();
        }

        let filename_path = self.get_filename_path(directory_path.as_ref(), 0);
        let rosbag_file = RosbagFile::new(directory_path.as_ref(), filename_path).unwrap();

        let bagfiles = vec![rosbag_file];
        let topics = HashMap::new();
        let rosbag = Rosbag::new(directory_path.as_ref(), bagfiles, topics)?;
        Ok(rosbag)
    }

    fn read_write_rosbag(&self, directory_path: impl AsRef<Path>) -> Result<Rosbag, Error> {
        if !directory_path.as_ref().is_dir() {
            return Err(Error::RosbagPathIsNoDirectory);
        }

        let mut bagfiles: Vec<RosbagFile> = vec![];
        let mut file_index = 0;
        loop {
            let filename_path = self.get_filename_path(directory_path.as_ref(), file_index);
            let rosbag_file_path = directory_path.as_ref().join(&filename_path);

            if rosbag_file_path.exists() {
                let rosbag_file =
                    RosbagFile::new(directory_path.as_ref(), filename_path.clone()).unwrap();
                bagfiles.push(rosbag_file);
                file_index += 1;
            } else {
                break;
            }
        }

        /*let transforms: HashMap<
            (ecoord::ChannelId, ecoord::TransformId),
            Vec<ecoord::Transform>,
        > = get_all_transforms(&mut a);*/

        let _transform_info: HashMap<ecoord::TransformId, TransformInfo> = self
            .transform_interpolation_method
            .iter()
            .map(|(transform_id, interpolation_method)| {
                (
                    transform_id.clone(),
                    TransformInfo::new(Some(*interpolation_method)),
                )
            })
            .collect();
        /*let reference_frames = ecoord::ReferenceFrames::new(
            HashMap::new(),
            // transforms,
            HashMap::new(),
            HashMap::new(),
            transform_info,
        );*/
        // TODO read and parse topics
        let topics = HashMap::new();

        let rosbag = Rosbag::new(directory_path.as_ref(), bagfiles, topics)?;
        Ok(rosbag)
    }

    fn get_filename_path(&self, directory_path: impl AsRef<Path>, file_index: u16) -> PathBuf {
        let bag_name = self.get_rosbag_name(directory_path);
        let file_index: String = file_index.to_string();
        let bagfile_path =
            PathBuf::from(bag_name + "_" + file_index.as_str()).with_extension(SQLITE3_EXTENSION);
        bagfile_path
    }

    fn get_rosbag_name(&self, directory_path: impl AsRef<Path>) -> String {
        let bag_name = directory_path
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        bag_name.to_string()
    }
}

pub fn get_all_transforms(
    connection: &mut SqliteConnection,
) -> HashMap<(ecoord::ChannelId, ecoord::TransformId), Vec<ecoord::Transform>> {
    let topic_names: HashSet<String> = schema::topics::dsl::topics
        .filter(schema::topics::type_.eq_all(RosMessageType::Tf2MessagesTFMessage.as_str()))
        .load::<Topic>(connection)
        .expect("Error loading messages")
        .iter()
        .map(|t| t.name.clone())
        .collect();

    assert!(!topic_names.is_empty(), "Must at least include one topic.");

    let mut all_transforms: HashMap<
        (ecoord::ChannelId, ecoord::TransformId),
        Vec<ecoord::Transform>,
    > = HashMap::new();

    for current_topic_name in topic_names {
        let current_channel_id: ecoord::ChannelId = current_topic_name.clone().into();
        let current_transforms = get_all_transforms_of_topic(connection, current_topic_name);

        for (current_transform_id, mut current_transform) in current_transforms {
            let key = (current_channel_id.clone(), current_transform_id);
            all_transforms
                .entry(key)
                .or_default()
                .append(&mut current_transform);
        }
    }

    all_transforms
}

fn get_all_transforms_of_topic(
    connection: &mut SqliteConnection,
    topic_name: String,
) -> HashMap<ecoord::TransformId, Vec<ecoord::Transform>> {
    let topic_entries: Vec<Topic> = schema::topics::dsl::topics
        .filter(schema::topics::name.eq_all(&topic_name))
        .load::<Topic>(connection)
        .expect("Error loading messages");
    assert_eq!(
        topic_entries.len(),
        1,
        "The topic name {topic_name} is used multiple times"
    );
    let topic_id = topic_entries.first().unwrap().id;
    let topic_type: RosMessageType =
        RosMessageType::from_str(&topic_entries.first().unwrap().type_).unwrap();
    assert_eq!(
        topic_type,
        RosMessageType::Tf2MessagesTFMessage,
        "Topic must contain messages of type Tf2MessagesTF."
    );

    let message_entries: Vec<Message> = schema::messages::dsl::messages
        .filter(schema::messages::topic_id.eq_all(topic_id))
        .load::<Message>(connection)
        .expect("Error loading messages");

    let decoded_ros_messages: Vec<TFMessage> = message_entries
        .iter()
        .map(|m| cdr::deserialize::<tf2_msgs::TFMessage>(&m.data[..]).unwrap())
        .collect();

    let mut transforms: HashMap<ecoord::TransformId, Vec<ecoord::Transform>> = HashMap::new();
    for current_ros_message in decoded_ros_messages {
        current_ros_message.transforms.iter().for_each(|t| {
            let transform_id = ecoord::TransformId::new(
                t.header.frame_id.clone().into(),
                t.child_frame_id.clone().into(),
            );
            let current_transform: ecoord::Transform = t.into();

            transforms
                .entry(transform_id)
                .or_default()
                .push(current_transform);
        })
    }

    transforms
}
