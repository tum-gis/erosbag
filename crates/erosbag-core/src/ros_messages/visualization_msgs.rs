///! Implementation of the [`visualization_msgs`] messages of ROS2.
///!
///! [`visualization_msgs`]: https://github.com/ros2/common_interfaces/tree/rolling/visualization_msgs/msg
use super::builtin_msgs;
use super::geometry_msgs;
use super::sensor_msgs;
use super::std_msgs;
use crate::ros_messages::{Header, MessageType, RosMessageType};
use serde_derive::{Deserialize, Serialize};

/// Implements the [`Marker`] message of ROS2.
///
/// [`Marker`]: https://github.com/ros2/common_interfaces/blob/rolling/visualization_msgs/msg/Marker.msg
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Marker {
    pub header: std_msgs::Header,
    pub ns: String,
    pub id: i32,
    #[serde(rename = "type")]
    pub type_: i32,
    pub action: i32,
    pub pose: geometry_msgs::Pose,
    pub scale: geometry_msgs::Vector3,
    pub color: std_msgs::ColorRGBA,
    pub lifetime: builtin_msgs::Duration,
    pub frame_locked: bool,

    pub points: Vec<geometry_msgs::Point>,
    pub colors: Vec<std_msgs::ColorRGBA>,

    pub texture_resource: String,
    pub texture: sensor_msgs::CompressedImage,
    pub uv_coordinates: Vec<UVCoordinate>,

    pub text: String,

    pub mesh_resource: String,
    pub mesh_file: MeshFile,
    pub mesh_use_embedded_materials: bool,
}

impl MessageType for Marker {
    fn ros_message_type(&self) -> &RosMessageType {
        &RosMessageType::VisualizationMessagesMarker
    }
}

impl Header for Marker {
    fn header(&self) -> &std_msgs::Header {
        &self.header
    }
}

/*impl Time for Marker {
    fn time(&self) -> &builtin_msgs::Time {
        &self.header.stamp
    }
}*/

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum MarkerObjectType {
    Arrow = 0,
    Cube = 1,
    Sphere = 2,
    Cylinder = 3,
    LineStrip = 4,
    LineList = 5,
    CubeList = 6,
    SphereList = 7,
    Points = 8,
    TextViewFacing = 9,
    MeshResource = 10,
    TriangleList = 11,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum MarkerActionType {
    Add = 0,
    Modify = 1,
    Delete = 2,
    DeleteAll = 3,
}

/// Implements the [`MarkerArray`] message of ROS2.
///
/// [`MarkerArray`]: https://github.com/ros2/common_interfaces/blob/rolling/visualization_msgs/msg/MarkerArray.msg
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct MarkerArray {
    pub markers: Vec<Marker>,
}

impl MessageType for MarkerArray {
    fn ros_message_type(&self) -> &RosMessageType {
        &RosMessageType::VisualizationMessagesMarkerArray
    }
}

impl Header for MarkerArray {
    fn header(&self) -> &std_msgs::Header {
        // TODO: improve solution
        &self.markers.first().unwrap().header
    }
}

/// Implements the [`MeshFile`] message of ROS2.
///
/// [`MeshFile`]: https://github.com/ros2/common_interfaces/blob/rolling/visualization_msgs/msg/MeshFile.msg
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct MeshFile {
    pub filename: String,
    pub data: Vec<u8>,
}

/// Implements the [`UVCoordinate`] message of ROS2.
///
/// [`UVCoordinate`]: https://github.com/ros2/common_interfaces/blob/rolling/visualization_msgs/msg/UVCoordinate.msg
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct UVCoordinate {
    pub u: f32,
    pub v: f32,
}
