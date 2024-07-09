use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Implements the [`Quality of Service`] profile for communication.
/// See [code](https://github.com/ros2/rmw/blob/31c6fd094c8bd01d0a231856df1bd9a476bea26a/rmw/include/rmw/types.h#L573-L617) for implementation.
///
/// [`Quality of Service`]: https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct QualityOfServiceProfile {
    pub history: HistoryPolicy,
    pub depth: u32,
    pub reliability: ReliabilityPolicy,
    pub durability: DurabilityPolicy,
    pub deadline: DateTime,
    pub lifespan: DateTime,
    pub liveliness: LivelinessPolicy,
    pub liveliness_lease_duration: DateTime,
    pub avoid_ros_namespace_conventions: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct DateTime {
    pub sec: i32,
    pub nsec: u32,
}

impl DateTime {
    const MAX: DateTime = Self {
        sec: i32::MAX,
        nsec: u32::MAX,
    };
}

/// [`QoS History policy`]
///
/// [`QoS History policy`]: https://github.com/ros2/rmw/blob/31c6fd094c8bd01d0a231856df1bd9a476bea26a/rmw/include/rmw/types.h#L408
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Clone, Hash)]
#[repr(u8)]
pub enum HistoryPolicy {
    SystemDefault = 0,
    /// Only store up to N samples, configurable via the queue depth option
    /// (see [docs](https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html#qos-policies)).
    KeepLast = 1,
    /// Store all samples, subject to the configured resource limits of the underlying middleware
    /// (see [docs](https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html#qos-policies)).
    KeepAll = 2,
    Unknown = 3,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Clone, Hash)]
#[repr(u8)]
pub enum ReliabilityPolicy {
    SystemDefault = 0,
    /// Guarantee that samples are delivered, may retry multiple times
    /// (see [docs](https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html#qos-policies)).
    Reliable = 1,
    /// Attempt to deliver samples, but may lose them if the network is not robust
    /// (see [docs](https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html#qos-policies)).
    BestEffort = 2,
    Unknown = 3,
    BestAvailable = 4,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Clone, Hash)]
#[repr(u8)]
pub enum DurabilityPolicy {
    SystemDefault = 0,
    /// The publisher becomes responsible for persisting samples for “late-joining” subscriptions
    /// (see [docs](https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html#qos-policies)).
    TransientLocal = 1,
    /// No attempt is made to persist samples
    /// (see [docs](https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html#qos-policies)).
    Volatile = 2,
    Unknown = 3,
    BestAvailable = 4,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Clone, Hash)]
#[repr(u8)]
pub enum LivelinessPolicy {
    SystemDefault = 0,
    /// The system will consider all of the node’s publishers to be alive for another
    /// “lease duration” when any one of its publishers has published a message
    /// (see [docs](https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html#qos-policies)).
    Automatic = 1,
    /// The system will consider the publisher to be alive for another “lease duration”
    /// if it manually asserts that it is still alive (via a call to the publisher API)
    /// (see [docs](https://docs.ros.org/en/rolling/Concepts/About-Quality-of-Service-Settings.html#qos-policies)).
    ManualByTopic = 3,
    Unknown = 4,
    BestAvailable = 5,
}

impl QualityOfServiceProfile {
    pub fn new_for_static_tf_topic() -> Self {
        Self {
            history: HistoryPolicy::Unknown,
            depth: 0,
            reliability: ReliabilityPolicy::Reliable,
            durability: DurabilityPolicy::TransientLocal,
            deadline: DateTime::MAX,
            lifespan: DateTime::MAX,
            liveliness: LivelinessPolicy::Automatic,
            liveliness_lease_duration: DateTime::MAX,
            avoid_ros_namespace_conventions: false,
        }
    }

    pub fn new_for_tf_topic() -> Self {
        Self {
            history: HistoryPolicy::KeepLast,
            depth: 0,
            reliability: ReliabilityPolicy::Reliable,
            durability: DurabilityPolicy::TransientLocal,
            deadline: DateTime::MAX,
            lifespan: DateTime::MAX,
            liveliness: LivelinessPolicy::Automatic,
            liveliness_lease_duration: DateTime::MAX,
            avoid_ros_namespace_conventions: false,
        }
    }
}
