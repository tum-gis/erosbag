use crate::bagfile::rosbag_file::RosbagFile;
use std::collections::{HashMap, HashSet};

use crate::rosbag_metadata::{
    DurationElement, FileElement, Rosbag2BagfileInformationElement, RosbagMetaDataDocument,
    StartingTimeElement, TopicMetaDataElement, TopicWithMessageCountElement,
};

use crate::error::Error;
use crate::ros_messages::{MessageType, Time};

use crate::topics::qos_profile::QualityOfServiceProfile;
use crate::topics::topic::TopicMetadata;
use crate::Error::ContainsNoRosbagFile;
use chrono::{DateTime, Utc};
use ecoord::{FrameId, InterpolationMethod, TransformId};
use serde::Serialize;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

pub struct Rosbag {
    directory_path: PathBuf,
    bagfiles: Vec<RosbagFile>,
    topics_metadata: HashMap<String, TopicMetadata>,
}

impl Rosbag {
    pub fn new(
        directory_path: impl AsRef<Path>,
        bagfiles: Vec<RosbagFile>,
        topics_metadata: HashMap<String, TopicMetadata>,
    ) -> Result<Self, Error> {
        if bagfiles.is_empty() {
            return Err(ContainsNoRosbagFile);
        }

        Ok(Self {
            directory_path: directory_path.as_ref().to_owned(),
            bagfiles,
            topics_metadata,
        })
    }

    pub fn create_topic(&self, name: &String, metadata: &TopicMetadata) {
        assert_eq!(
            self.bagfiles.len(),
            1,
            "Currently only a single bagfile is supported for this operation"
        );
        let bagfile = self.bagfiles.first().unwrap();
        bagfile.create_topic(name, metadata);

        // TopicId::from(0)
    }

    pub fn get_all_topic_names(&self) -> HashSet<String> {
        let t: HashSet<String> = self
            .bagfiles
            .iter()
            .flat_map(|f| f.get_all_topic_names())
            .collect();
        t
    }

    pub fn get_images(&self) {
        todo!("TODO return images")
    }

    pub fn get_visualization_markers(&self, topic_name: &String) {
        assert_eq!(
            self.bagfiles.len(),
            1,
            "Currently only a single bagfile is supported for this operation"
        );
        let bagfile = self.bagfiles.first().unwrap();
        bagfile.get_visualization_markers(topic_name);
    }

    fn generate_metadata(&self) -> RosbagMetaDataDocument {
        let relative_file_paths: Vec<String> = self
            .bagfiles
            .iter()
            .map(|f| {
                f.filename_path
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap()
            })
            .collect();
        let files: Vec<FileElement> = self
            .bagfiles
            .iter()
            .map(|f| FileElement {
                path: f
                    .filename_path
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap(),
                starting_time: StartingTimeElement {
                    nanoseconds_since_epoch: f.get_start_time().unwrap_or(0),
                },
                duration: DurationElement {
                    nanoseconds: f.get_duration().unwrap_or(0),
                },
                message_count: f.get_message_count(),
            })
            .collect();

        let duration = DurationElement {
            nanoseconds: self
                .bagfiles
                .last()
                .unwrap()
                .get_stop_time()
                .unwrap_or_default()
                - self
                    .bagfiles
                    .first()
                    .unwrap()
                    .get_start_time()
                    .unwrap_or_default(),
        };

        let message_count = files.iter().map(|f| f.message_count).sum();

        let topics_with_message_count: Vec<TopicWithMessageCountElement> = self
            .bagfiles
            .first()
            .unwrap()
            .get_all_topics()
            .iter()
            .map(|t| TopicWithMessageCountElement {
                topic_metadata: TopicMetaDataElement {
                    name: t.0.clone(),
                    type_: t.1.message_type.as_str().to_string(),
                    serialization_format: t.1.serialization_format.as_str().to_string(),
                    offered_qos_profiles: serde_yaml::to_string(&t.1.offered_qos_profiles).unwrap(),
                },
                message_count,
            })
            .collect();

        let metadata = RosbagMetaDataDocument {
            rosbag2_bagfile_information: Rosbag2BagfileInformationElement {
                version: 6,
                storage_identifier: "sqlite3".to_string(),
                duration,
                starting_time: files.first().unwrap().starting_time.clone(),
                message_count,
                topics_with_message_count,
                compression_format: String::new(),
                compression_mode: String::new(),
                relative_file_paths,
                files,
                custom_data: HashMap::new(),
            },
        };

        metadata
    }

    pub fn append_transform_messages(&mut self, reference_frames: ecoord::ReferenceFrames) {
        assert_eq!(
            self.bagfiles.len(),
            1,
            "Currently only a single bagfile is supported for this operation"
        );

        let bagfile = self.bagfiles.first().unwrap();
        bagfile.append_transform_messages(reference_frames);
    }

    pub fn append_message(
        &mut self,
        topic_name: &String,
        ros_message: &(impl MessageType + Time + Serialize),
    ) {
        assert_eq!(
            self.bagfiles.len(),
            1,
            "Currently only a single bagfile is supported for this operation"
        );

        let bagfile = self.bagfiles.first().unwrap();
        bagfile.append_message(topic_name, ros_message);
    }

    pub fn set_qos_profile(
        &mut self,
        topic_name: &String,
        offered_qos_profile: &QualityOfServiceProfile,
    ) {
        self.bagfiles
            .iter()
            .for_each(|f| f.set_qos_profile(topic_name, offered_qos_profile));
    }

    pub fn get_transforms(
        &self,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<ecoord::ReferenceFrames, Error> {
        assert_eq!(
            self.bagfiles.len(),
            1,
            "Currently only a single bagfile is supported for this operation"
        );
        let bagfile = self.bagfiles.first().unwrap();
        bagfile.get_transforms(start_time, stop_time)
    }

    pub fn get_point_clouds(
        &self,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<epoint::PointCloud, Error> {
        assert_eq!(
            self.bagfiles.len(),
            1,
            "Currently only a single bagfile is supported for this operation"
        );
        let bagfile = self.bagfiles.first().unwrap();

        let mut point_cloud = bagfile.get_all_point_cloud(start_time, stop_time)?;
        let mut reference_frames = self.get_transforms(start_time, stop_time)?;
        let transform_id = TransformId::new(FrameId::from("slam_map"), FrameId::from("base_link"));
        reference_frames.set_interpolation_method(transform_id, Some(InterpolationMethod::Linear));
        point_cloud.set_reference_frames(reference_frames);

        Ok(point_cloud)
    }

    pub fn close(self) {
        let metadata = self.generate_metadata();
        let document_path = self.directory_path.join(PathBuf::from("metadata.yaml"));
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(document_path)
            .unwrap();

        serde_yaml::to_writer(&file, &metadata).unwrap();
    }
}
