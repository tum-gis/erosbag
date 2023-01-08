///! Implementation of the [`tf2_msgs`] messages of ROS2.
///!
///! [`tf2_msgs`]: https://github.com/ros2/geometry2/blob/rolling/tf2_msgs/msg
use crate::ros_messages::{geometry_msgs, MessageType, RosMessageType};
use serde_derive::{Deserialize, Serialize};

/// Implements the [`TFMessage`] of ROS2.
///
/// [`TFMessage`]: https://github.com/ros2/geometry2/blob/rolling/tf2_msgs/msg/TFMessage.msg
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TFMessage {
    pub transforms: Vec<geometry_msgs::TransformStamped>,
}

impl MessageType for TFMessage {
    fn ros_message_type(&self) -> &RosMessageType {
        &RosMessageType::Tf2MessagesTFMessage
    }
}
