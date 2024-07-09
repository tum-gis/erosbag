const COLUMN_NAME_ROS_MESSAGE_ID_STR: &str = "ros_message_id";
const COLUMN_NAME_ROS_POINT_ID_STR: &str = "ros_point_id";

/// Additional column names for ROS specific fields for `epoint::PointCloud`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RosPointCloudColumnType {
    RosMessageId,
    RosPointId,
}

impl RosPointCloudColumnType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RosPointCloudColumnType::RosMessageId => COLUMN_NAME_ROS_MESSAGE_ID_STR,
            RosPointCloudColumnType::RosPointId => COLUMN_NAME_ROS_POINT_ID_STR,
        }
    }
}
