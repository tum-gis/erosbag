///! Implementation of the [`std_msgs`] messages of ROS2.
///!
///! [`std_msgs`]: https://github.com/ros2/common_interfaces/tree/rolling/std_msgs/msg
use super::builtin_msgs;
use serde_derive::{Deserialize, Serialize};

/// Implements the [`ColorRGBA`] message of ROS2.
///
/// [`ColorRGBA`]: https://github.com/ros2/common_interfaces/blob/rolling/std_msgs/msg/ColorRGBA.msg
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for ColorRGBA {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }
}

/// Implements the [`Header`] message of ROS2.
///
/// [`Header`]: https://github.com/ros2/common_interfaces/blob/rolling/std_msgs/msg/Header.msg
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Header {
    pub stamp: builtin_msgs::Time,
    pub frame_id: String,
}
