///! Implementation of the [`sensor_msgs`] messages of ROS2.
///!
///! [`sensor_msgs`]: https://github.com/ros2/common_interfaces/tree/rolling/sensor_msgs/msg
use super::geometry_msgs;
use super::std_msgs;
use crate::ros_messages::{MessageType, RosMessageType};

use serde_derive::{Deserialize, Serialize};

use crate::bagfile::point_cloud_extensions::RosPointDataColumnType;
use chrono::{DateTime, Utc};
use image::{ImageBuffer, Rgb};
use itertools::izip;
use nalgebra::{Isometry3, Point3};

/// Implements the [`CompressedImage`] message of ROS2.
///
/// [`CompressedImage`]: https://github.com/ros2/common_interfaces/blob/rolling/sensor_msgs/msg/CompressedImage.msg
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct CompressedImage {
    pub header: std_msgs::Header,
    pub format: String,
    pub data: Vec<u8>,
}

/// Implements the [`Image`] message of ROS2.
///
/// [`Image`]: https://github.com/ros2/common_interfaces/blob/rolling/sensor_msgs/msg/Image.msg
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Image {
    pub header: std_msgs::Header,
    pub height: u32,
    pub width: u32,
    pub encoding: String,

    pub is_bigendian: bool,
    pub step: u32,
    pub data: Vec<u8>,
}

impl MessageType for Image {
    fn ros_message_type(&self) -> &RosMessageType {
        &RosMessageType::SensorMessagesImage
    }
}

impl From<Image> for eimage::Image {
    fn from(item: Image) -> Self {
        let buffer: Vec<u8> = item
            .data
            .chunks(3)
            .flat_map(|c| [c[2], c[1], c[0]])
            .collect();

        let image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_vec(item.width, item.height, buffer).unwrap();

        eimage::Image::new(image_buffer, item.header.stamp.into())
    }
}

/// Implements the [`Imu`] message of ROS2.
///
/// [`Imu`]: https://github.com/ros2/common_interfaces/blob/rolling/sensor_msgs/msg/Imu.msg
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Imu {
    pub header: std_msgs::Header,
    pub angular_velocity: geometry_msgs::Vector3,
    pub linear_acceleration: geometry_msgs::Vector3,
}

impl MessageType for Imu {
    fn ros_message_type(&self) -> &RosMessageType {
        &RosMessageType::SensorMessagesImu
    }
}

/// Implements the [`NavSatFix`] message of ROS2.
///
/// [`NavSatFix`]: https://github.com/ros2/common_interfaces/blob/rolling/sensor_msgs/msg/NavSatFix.msg
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NavSatFix {
    pub header: std_msgs::Header,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub position_covariance: [f64; 9],
    pub position_covariance_type: u8,
}

impl MessageType for NavSatFix {
    fn ros_message_type(&self) -> &RosMessageType {
        &RosMessageType::SensorMessagesNavSatFix
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum NavSatFixPositionCovarianceType {
    CovarianceTypeUnknown = 0,
    CovarianceTypeApproximated = 1,
    CovarianceTypeDiagonalKnown = 2,
    CovarianceTypeKnown = 3,
}

/// Implements the [`PointCloud2`] message of ROS2.
///
/// [`PointCloud2`]: https://github.com/ros2/common_interfaces/blob/rolling/sensor_msgs/msg/PointCloud2.msg
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PointCloud2 {
    pub header: std_msgs::Header,
    pub height: u32,
    pub width: u32,

    pub fields: Vec<PointField>,

    pub is_bigendian: bool,
    pub point_step: u32,
    pub row_step: u32,
    pub data: Vec<u8>,

    pub is_dense: bool,
}

impl MessageType for PointCloud2 {
    fn ros_message_type(&self) -> &RosMessageType {
        &RosMessageType::SensorMessagesPointCloud2
    }
}

impl From<PointCloud2> for epoint::PointCloud {
    fn from(item: PointCloud2) -> Self {
        let mut point_data = item.get_epoint();
        let frame_id: Vec<String> = vec![item.header.frame_id.clone(); point_data.len()];
        point_data.frame_id = Some(frame_id);
        let timestamp: Vec<DateTime<Utc>> = vec![item.header.stamp.into(); point_data.len()];
        point_data.timestamp = Some(timestamp);

        let mut point_cloud = epoint::PointCloud::new(
            point_data,
            epoint::PointCloudInfo::new(None),
            ecoord::ReferenceFrames::default(),
        )
        .expect("creating point cloud should work");

        point_cloud
            .point_data
            .add_unique_sensor_pose(Isometry3::identity())
            .expect("Adding the origin should work");

        let point_id: Vec<u32> = (0..point_cloud.size()).map(|x| x as u32).collect();
        point_cloud
            .point_data
            .add_u32_column(RosPointDataColumnType::RosPointId.as_str(), point_id)
            .expect("Extending the id column should work");

        point_cloud
    }
}

impl PointCloud2 {
    pub fn get_epoint(&self) -> epoint::PointDataColumns {
        let x_values = self.get_field_as_f32("x");
        let y_values = self.get_field_as_f32("y");
        let z_values = self.get_field_as_f32("z");
        let points: Vec<Point3<f64>> = izip!(&x_values, &y_values, &z_values)
            .map(|v| Point3::<f64>::new(*v.0 as f64, *v.1 as f64, *v.2 as f64))
            .collect();

        let intensity = self.get_field_as_f32("intensity");

        epoint::PointDataColumns::new(points, None, None, None, Some(intensity), None, None)
            .unwrap()
    }

    pub fn get_field_as_f32(&self, name: &str) -> Vec<f32> {
        let found_entries: Vec<(usize, &PointField)> = self
            .fields
            .iter()
            .enumerate()
            .filter(|f| f.1.name == name)
            .collect();
        assert_eq!(found_entries.len(), 1, "Must be exactly one");
        assert_eq!(
            found_entries.first().unwrap().1.datatype_a(),
            PointFieldDataType::FLOAT32,
            "Must be exactly one"
        );
        let current_field = found_entries.first().unwrap().1;

        let num_entries = self.data.len() as u32 / self.point_step;
        let values: Vec<f32> = (0..num_entries)
            .map(|current_point_index| {
                let start: usize =
                    (current_point_index * self.point_step + current_field.offset) as usize;

                let mut current_num_bytes: [u8; 4] = Default::default();
                current_num_bytes.copy_from_slice(&self.data[start..start + 4]);

                f32::from_le_bytes(current_num_bytes)
            })
            .collect();

        values
    }
}

/// Implements the [`PointField`] message of ROS2.
///
/// [`PointField`]: https://github.com/ros2/common_interfaces/blob/rolling/sensor_msgs/msg/PointField.msg
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct PointField {
    pub name: String,
    pub offset: u32,
    pub datatype: u8,
    pub count: u32,
}

impl PointField {
    pub fn datatype_a(&self) -> PointFieldDataType {
        self.datatype.try_into().unwrap()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum PointFieldDataType {
    INT8,
    UINT8,
    INT16,
    UINT16,
    INT32,
    UINT32,
    FLOAT32,
    FLOAT64,
}

impl TryFrom<u8> for PointFieldDataType {
    type Error = ();

    fn try_from(val: u8) -> Result<PointFieldDataType, ()> {
        match val {
            1 => Ok(PointFieldDataType::INT8),
            2 => Ok(PointFieldDataType::UINT8),
            3 => Ok(PointFieldDataType::INT16),
            4 => Ok(PointFieldDataType::UINT16),
            5 => Ok(PointFieldDataType::INT32),
            6 => Ok(PointFieldDataType::UINT32),
            7 => Ok(PointFieldDataType::FLOAT32),
            8 => Ok(PointFieldDataType::FLOAT64),
            _ => Err(()),
        }
    }
}
