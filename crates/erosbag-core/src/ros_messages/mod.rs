use std::str::FromStr;

pub mod builtin_msgs;
pub mod geometry_msgs;
pub mod nav_msgs;
pub mod sensor_msgs;
pub mod std_msgs;
pub mod tf2_msgs;
pub mod visualization_msgs;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
            "sensor_msgs/msg/Imu" => Ok(RosMessageType::SensorMessagesImu),
            "sensor_msgs/msg/NavSatFix" => Ok(RosMessageType::SensorMessagesNavSatFix),
            "sensor_msgs/msg/PointCloud2" => Ok(RosMessageType::SensorMessagesPointCloud2),
            "sensor_msgs/msg/Image" => Ok(RosMessageType::SensorMessagesImage),
            "sensor_msgs/msg/CameraInfo" => Ok(RosMessageType::SensorMessagesCameraInfo),
            "tf2_msgs/msg/TFMessage" => Ok(RosMessageType::Tf2MessagesTFMessage),
            "nav_msgs/msg/Odometry" => Ok(RosMessageType::NavMessagesOdometry),
            "visualization_msgs/msg/Marker" => Ok(RosMessageType::VisualizationMessagesMarker),
            "visualization_msgs/msg/MarkerArray" => {
                Ok(RosMessageType::VisualizationMessagesMarkerArray)
            }
            _ => Err(()),
        }
    }
}

impl RosMessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RosMessageType::SensorMessagesImu => "sensor_msgs/msg/Imu",
            RosMessageType::SensorMessagesNavSatFix => "sensor_msgs/msg/NavSatFix",
            RosMessageType::SensorMessagesPointCloud2 => "sensor_msgs/msg/PointCloud2",
            RosMessageType::SensorMessagesImage => "sensor_msgs/msg/Image",
            RosMessageType::SensorMessagesCameraInfo => "sensor_msgs/msg/CameraInfo",
            RosMessageType::Tf2MessagesTFMessage => "tf2_msgs/msg/TFMessage",
            RosMessageType::NavMessagesOdometry => "nav_msgs/msg/Odometry",
            RosMessageType::VisualizationMessagesMarker => "visualization_msgs/msg/Marker",
            RosMessageType::VisualizationMessagesMarkerArray => {
                "visualization_msgs/msg/MarkerArray"
            }
        }
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
