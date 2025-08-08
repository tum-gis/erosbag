use polars::datatypes::PlSmallStr;

const COLUMN_NAME_ROS_MESSAGE_ID_STR: &str = "ros_message_id";
const COLUMN_NAME_ROS_POINT_ID_STR: &str = "ros_point_id";

/// Additional column names for ROS specific fields for `epoint::PointCloud`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RosPointDataColumnType {
    RosMessageId,
    RosPointId,
}

impl RosPointDataColumnType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RosPointDataColumnType::RosMessageId => COLUMN_NAME_ROS_MESSAGE_ID_STR,
            RosPointDataColumnType::RosPointId => COLUMN_NAME_ROS_POINT_ID_STR,
        }
    }
}

impl From<RosPointDataColumnType> for PlSmallStr {
    fn from(value: RosPointDataColumnType) -> Self {
        value.as_str().into()
    }
}
