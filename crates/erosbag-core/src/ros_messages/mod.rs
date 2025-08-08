use std::fmt;
use std::str::FromStr;

pub mod builtin_msgs;
pub mod geometry_msgs;
pub mod nav_msgs;
pub mod sensor_msgs;
pub mod std_msgs;
pub mod tf2_msgs;
pub mod visualization_msgs;

const ROS_MESSAGE_TYPE_SENSOR_MESSAGES_IMU_STR: &str = "sensor_msgs/msg/Imu";
const ROS_MESSAGE_TYPE_SENSOR_MESSAGES_NAV_SAT_FIX_STR: &str = "sensor_msgs/msg/NavSatFix";
const ROS_MESSAGE_TYPE_SENSOR_MESSAGES_POINT_CLOUD_2_STR: &str = "sensor_msgs/msg/PointCloud2";
const ROS_MESSAGE_TYPE_SENSOR_MESSAGES_IMAGE_STR: &str = "sensor_msgs/msg/Image";
const ROS_MESSAGE_TYPE_SENSOR_MESSAGES_CAMERA_INFO_STR: &str = "sensor_msgs/msg/CameraInfo";
const ROS_MESSAGE_TYPE_TF2_MESSAGES_TF_MESSAGE_STR: &str = "tf2_msgs/msg/TFMessage";
const ROS_MESSAGE_TYPE_NAV_MESSAGES_ODOMETRY_STR: &str = "nav_msgs/msg/Odometry";
const ROS_MESSAGE_TYPE_VISUALIZATION_MESSAGES_MARKER_STR: &str = "visualization_msgs/msg/Marker";
const ROS_MESSAGE_TYPE_VISUALIZATION_MESSAGES_MARKER_ARRAY_STR: &str =
    "visualization_msgs/msg/MarkerArray";

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RosMessageType {
    SensorMessagesImu,
    SensorMessagesNavSatFix,
    SensorMessagesPointCloud2,
    SensorMessagesImage,
    SensorMessagesCameraInfo,
    Tf2MessagesTFMessage,
    NavMessagesOdometry,
    VisualizationMessagesMarker,
    VisualizationMessagesMarkerArray,
}

impl FromStr for RosMessageType {
    type Err = ();

    fn from_str(input: &str) -> Result<RosMessageType, Self::Err> {
        match input {
            ROS_MESSAGE_TYPE_SENSOR_MESSAGES_IMU_STR => Ok(RosMessageType::SensorMessagesImu),
            ROS_MESSAGE_TYPE_SENSOR_MESSAGES_NAV_SAT_FIX_STR => {
                Ok(RosMessageType::SensorMessagesNavSatFix)
            }
            ROS_MESSAGE_TYPE_SENSOR_MESSAGES_POINT_CLOUD_2_STR => {
                Ok(RosMessageType::SensorMessagesPointCloud2)
            }
            ROS_MESSAGE_TYPE_SENSOR_MESSAGES_IMAGE_STR => Ok(RosMessageType::SensorMessagesImage),
            ROS_MESSAGE_TYPE_SENSOR_MESSAGES_CAMERA_INFO_STR => {
                Ok(RosMessageType::SensorMessagesCameraInfo)
            }
            ROS_MESSAGE_TYPE_TF2_MESSAGES_TF_MESSAGE_STR => {
                Ok(RosMessageType::Tf2MessagesTFMessage)
            }
            ROS_MESSAGE_TYPE_NAV_MESSAGES_ODOMETRY_STR => Ok(RosMessageType::NavMessagesOdometry),
            ROS_MESSAGE_TYPE_VISUALIZATION_MESSAGES_MARKER_STR => {
                Ok(RosMessageType::VisualizationMessagesMarker)
            }
            ROS_MESSAGE_TYPE_VISUALIZATION_MESSAGES_MARKER_ARRAY_STR => {
                Ok(RosMessageType::VisualizationMessagesMarkerArray)
            }
            _ => Err(()),
        }
    }
}

impl RosMessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RosMessageType::SensorMessagesImu => ROS_MESSAGE_TYPE_SENSOR_MESSAGES_IMU_STR,
            RosMessageType::SensorMessagesNavSatFix => {
                ROS_MESSAGE_TYPE_SENSOR_MESSAGES_NAV_SAT_FIX_STR
            }
            RosMessageType::SensorMessagesPointCloud2 => {
                ROS_MESSAGE_TYPE_SENSOR_MESSAGES_POINT_CLOUD_2_STR
            }
            RosMessageType::SensorMessagesImage => ROS_MESSAGE_TYPE_SENSOR_MESSAGES_IMAGE_STR,
            RosMessageType::SensorMessagesCameraInfo => {
                ROS_MESSAGE_TYPE_SENSOR_MESSAGES_CAMERA_INFO_STR
            }
            RosMessageType::Tf2MessagesTFMessage => ROS_MESSAGE_TYPE_TF2_MESSAGES_TF_MESSAGE_STR,
            RosMessageType::NavMessagesOdometry => ROS_MESSAGE_TYPE_NAV_MESSAGES_ODOMETRY_STR,
            RosMessageType::VisualizationMessagesMarker => {
                ROS_MESSAGE_TYPE_VISUALIZATION_MESSAGES_MARKER_STR
            }
            RosMessageType::VisualizationMessagesMarkerArray => {
                ROS_MESSAGE_TYPE_VISUALIZATION_MESSAGES_MARKER_ARRAY_STR
            }
        }
    }
}

impl fmt::Display for RosMessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub trait MessageType {
    fn ros_message_type(&self) -> &RosMessageType;
}

pub trait Header {
    fn header(&self) -> &std_msgs::Header;
}

pub trait Time {
    fn time(&self) -> &builtin_msgs::Time;
}

impl<T> Time for T
where
    T: Header,
{
    fn time(&self) -> &builtin_msgs::Time {
        &self.header().stamp
    }
}
