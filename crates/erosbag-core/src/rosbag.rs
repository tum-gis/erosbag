use crate::bagfile::rosbag_file::RosbagFile;
use std::collections::{HashMap, HashSet};

use crate::rosbag_metadata::{
    DurationElement, FileElement, Rosbag2BagfileInformationElement, RosbagMetaDataDocument,
    StartingTimeElement, TopicMetaDataElement, TopicWithMessageCountElement,
};

use crate::error::Error;
use crate::ros_messages::{MessageType, Time};

use crate::ros_messages::sensor_msgs::NavSatFix;
use crate::topics::qos_profile::QualityOfServiceProfile;
use crate::topics::topic::TopicMetadata;
use crate::Error::{ContainsNoRosbagFile, MultipleBagfilesNotSupported};
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
        if bagfiles.len() > 1 {
            return Err(MultipleBagfilesNotSupported);
        }

        Ok(Self {
            directory_path: directory_path.as_ref().to_owned(),
            bagfiles,
            topics_metadata,
        })
    }

    pub fn create_topic(&self, name: &String, metadata: &TopicMetadata) -> Result<(), Error> {
        if self.bagfiles.len() != 1 {
            return Err(MultipleBagfilesNotSupported);
        }
        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;
        bagfile.create_topic(name, metadata)?;

        Ok(())
        // TopicId::from(0)
    }

    pub fn get_all_topic_names(&self) -> HashSet<String> {
        let t: HashSet<String> = self
            .bagfiles
            .iter()
            .flat_map(|f| f.get_all_topic_names().unwrap())
            .collect();
        t
    }

    pub fn get_start_date_time(&self) -> Result<Option<DateTime<Utc>>, Error> {
        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;
        let start_date = bagfile.get_start_date_time()?;
        Ok(start_date)
    }

    pub fn get_stop_date_time(&self) -> Result<Option<DateTime<Utc>>, Error> {
        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;
        let stop_date = bagfile.get_stop_date_time()?;
        Ok(stop_date)
    }

    pub fn get_images(
        &self,
        topic_name: &String,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<eimage::ImageSeries, Error> {
        if self.bagfiles.len() != 1 {
            return Err(MultipleBagfilesNotSupported);
        }
        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;
        let image_series = bagfile.get_images(topic_name, start_time, stop_time)?;
        Ok(image_series)
    }

    pub fn get_visualization_markers(&self, topic_name: &String) -> Result<(), Error> {
        if self.bagfiles.len() != 1 {
            return Err(MultipleBagfilesNotSupported);
        }
        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;
        bagfile.get_visualization_markers(topic_name)?;
        Ok(())
    }

    fn generate_metadata(&self) -> Result<RosbagMetaDataDocument, Error> {
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
                    nanoseconds_since_epoch: f.get_start_time().unwrap().unwrap_or_default(),
                },
                duration: DurationElement {
                    nanoseconds: f.get_duration().unwrap().unwrap_or_default(),
                },
                message_count: f.get_message_count().unwrap(),
            })
            .collect();

        let duration = DurationElement {
            nanoseconds: self
                .bagfiles
                .last()
                .unwrap()
                .get_stop_time()?
                .unwrap_or_default()
                - self
                    .bagfiles
                    .first()
                    .unwrap()
                    .get_start_time()?
                    .unwrap_or_default(),
        };

        let message_count = files.iter().map(|f| f.message_count).sum();

        let topics_with_message_count: Vec<TopicWithMessageCountElement> = self
            .bagfiles
            .first()
            .unwrap()
            .get_all_topics()
            .unwrap()
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

        Ok(metadata)
    }

    pub fn append_transform_messages(
        &mut self,
        reference_frames: ecoord::ReferenceFrames,
    ) -> Result<(), Error> {
        if self.bagfiles.len() != 1 {
            return Err(MultipleBagfilesNotSupported);
        }

        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;
        bagfile.append_transform_messages(reference_frames)?;

        Ok(())
    }

    pub fn append_message(
        &mut self,
        topic_name: &String,
        ros_message: &(impl MessageType + Time + Serialize),
    ) -> Result<(), Error> {
        if self.bagfiles.len() != 1 {
            return Err(MultipleBagfilesNotSupported);
        }

        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;
        bagfile.append_message(topic_name, ros_message)?;
        Ok(())
    }

    pub fn set_qos_profile(
        &mut self,
        topic_name: &String,
        offered_qos_profile: &QualityOfServiceProfile,
    ) {
        self.bagfiles.iter().for_each(|f| {
            f.set_qos_profile(topic_name, offered_qos_profile)
                .expect("should work")
        });
    }

    pub fn get_transforms(
        &self,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<ecoord::ReferenceFrames, Error> {
        if self.bagfiles.len() != 1 {
            return Err(MultipleBagfilesNotSupported);
        }

        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;
        let reference_rames = bagfile.get_transforms(start_time, stop_time)?;

        Ok(reference_rames)
    }

    /// Returns the point cloud on all topics for a time window between start_time (inclusive) and
    /// stop_time (exclusive).
    pub fn get_point_clouds(
        &self,
        start_time: &Option<DateTime<Utc>>,
        stop_time: &Option<DateTime<Utc>>,
    ) -> Result<epoint::PointCloud, Error> {
        if self.bagfiles.len() != 1 {
            return Err(MultipleBagfilesNotSupported);
        }
        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;

        let mut point_cloud = bagfile.get_all_point_cloud(start_time, stop_time)?;
        let mut reference_frames = self.get_transforms(start_time, stop_time)?;
        let transform_id = TransformId::new(FrameId::from("slam_map"), FrameId::from("base_link"));
        reference_frames.set_interpolation_method(transform_id, Some(InterpolationMethod::Linear));
        point_cloud.set_reference_frames(reference_frames);

        Ok(point_cloud)
    }

    pub fn get_nav_sat_messages(&self, topic_name: &String) -> Result<Vec<NavSatFix>, Error> {
        let bagfile = self.bagfiles.first().ok_or(ContainsNoRosbagFile)?;

        let messages = bagfile.get_nav_sat_fix_messages_of_topic(topic_name, &None, &None)?;
        Ok(messages)
    }

    pub fn close(self) {
        let metadata = self.generate_metadata().unwrap();
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
