///! Implementation of the [`builtin_interfaces`] messages of ROS2.
///!
///! [`builtin_interfaces`]: https://github.com/ros2/rcl_interfaces/tree/rolling/builtin_interfaces/msg
use chrono::{DateTime, TimeZone, Timelike, Utc};
use serde_derive::{Deserialize, Serialize};

/// Implements the [`Duration`] message of ROS2.
///
/// [`Duration`]: https://github.com/ros2/rcl_interfaces/blob/rolling/builtin_interfaces/msg/Duration.msg
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Duration {
    pub sec: i32,
    pub nanosec: u32,
}

impl Duration {
    pub const MAX: Self = Self {
        sec: i32::MAX,
        nanosec: u32::MAX,
    };
}

/// Implements the [`Time`] message of ROS2.
///
/// [`Time`]: https://github.com/ros2/rcl_interfaces/blob/rolling/builtin_interfaces/msg/Time.msg
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Time {
    pub sec: i32,
    pub nanosec: u32,
}

impl From<Time> for DateTime<Utc> {
    fn from(item: Time) -> Self {
        Utc.timestamp_opt(item.sec as i64, item.nanosec).unwrap()
    }
}

impl From<Time> for i64 {
    fn from(item: Time) -> Self {
        Utc.timestamp_opt(item.sec as i64, item.nanosec)
            .unwrap()
            .timestamp()
    }
}

impl From<DateTime<Utc>> for Time {
    fn from(item: DateTime<Utc>) -> Self {
        Self {
            sec: item.timestamp() as i32,
            nanosec: item.nanosecond(),
        }
    }
}
