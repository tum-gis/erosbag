///! Implementation of the [`nav_msgs`] messages of ROS2.
///!
///! [`nav_msgs`]: https://github.com/ros2/common_interfaces/tree/rolling/nav_msgs/msg
use super::geometry_msgs;
use super::std_msgs;
use crate::ros_messages::{MessageType, RosMessageType};
use serde_derive::{Deserialize, Serialize};

/// Implements the [`Odometry`] message of ROS2.
///
/// [`Odometry`]: https://github.com/ros2/common_interfaces/blob/rolling/nav_msgs/msg/Odometry.msg
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Odometry {
    pub header: std_msgs::Header,
    pub child_frame_id: String,
    pub pose: geometry_msgs::PoseWithCovariance,
    pub twist: geometry_msgs::TwistWithCovariance,
}

impl MessageType for Odometry {
    fn ros_message_type(&self) -> &RosMessageType {
        &RosMessageType::NavMessagesOdometry
    }
}
