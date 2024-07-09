///! Implementation of the [`geometry_msgs`] messages of ROS2.
///!
///! [`geometry_msgs`]: https://github.com/ros2/common_interfaces/tree/rolling/geometry_msgs/msg
use crate::ros_messages::std_msgs;
use serde_big_array::BigArray;
use serde_derive::{Deserialize, Serialize};

/// Implements the [`Point`] message of ROS2.
///
/// [`Point`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/Point.msg
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<Point> for nalgebra::Vector3<f64> {
    fn from(item: Point) -> Self {
        Self::new(item.x, item.y, item.z)
    }
}

impl From<nalgebra::Vector3<f64>> for Point {
    fn from(item: nalgebra::Vector3<f64>) -> Self {
        Self {
            x: item.x,
            y: item.y,
            z: item.z,
        }
    }
}

/// Implements the [`Pose`] message of ROS2.
///
/// [`Pose`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/Pose.msg
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize)]
pub struct Pose {
    pub point: Point,
    pub quaternion: Quaternion,
}

impl From<Pose> for nalgebra::Isometry3<f64> {
    fn from(item: Pose) -> Self {
        let p: nalgebra::Vector3<f64> = item.point.into();
        let translation: nalgebra::Translation3<f64> = p.into();
        let rotation: nalgebra::UnitQuaternion<f64> = item.quaternion.into();
        Self::from_parts(translation, rotation)
    }
}

impl From<nalgebra::Isometry3<f64>> for Pose {
    fn from(item: nalgebra::Isometry3<f64>) -> Self {
        let point = Point {
            x: item.translation.x,
            y: item.translation.y,
            z: item.translation.z,
        };
        let quaternion = Quaternion {
            x: item.rotation.i,
            y: item.rotation.j,
            z: item.rotation.k,
            w: item.rotation.w,
        };

        Self { point, quaternion }
    }
}

/// Implements the [`PoseWithCovariance`] message of ROS2.
///
/// [`PoseWithCovariance`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/PoseWithCovariance.msg
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct PoseWithCovariance {
    pub pose: Pose,
    #[serde(with = "BigArray")]
    pub covariance: [f64; 36],
}

/// Implements the [`Quaternion`] message of ROS2.
///
/// [`Quaternion`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/Quaternion.msg
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl From<Quaternion> for nalgebra::Quaternion<f64> {
    fn from(item: Quaternion) -> Self {
        Self::new(item.w, item.x, item.y, item.z)
    }
}

impl From<Quaternion> for nalgebra::UnitQuaternion<f64> {
    fn from(item: Quaternion) -> Self {
        Self::from_quaternion(item.into())
    }
}

impl From<nalgebra::UnitQuaternion<f64>> for Quaternion {
    fn from(item: nalgebra::UnitQuaternion<f64>) -> Self {
        Self {
            w: item.w,
            x: item.j,
            y: item.j,
            z: item.k,
        }
    }
}

/// Implements the [`Transform`] message of ROS2.
///
/// [`Transform`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/Transform.msg
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Transform {
    pub translation: Vector3,
    pub rotation: Quaternion,
}

impl From<Transform> for nalgebra::Isometry3<f64> {
    fn from(item: Transform) -> Self {
        Self::from_parts(item.translation.into(), item.rotation.into())
    }
}

impl From<nalgebra::Isometry3<f64>> for Transform {
    fn from(item: nalgebra::Isometry3<f64>) -> Self {
        Self {
            translation: item.translation.into(),
            rotation: item.rotation.into(),
        }
    }
}

/// Implements the [`TransformStamped`] message of ROS2.
///
/// [`TransformStamped`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/TransformStamped.msg
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransformStamped {
    pub header: std_msgs::Header,
    pub child_frame_id: String,
    pub transform: Transform,
}

impl From<&TransformStamped> for ecoord::Transform {
    fn from(item: &TransformStamped) -> Self {
        Self::new(
            item.header.stamp.into(),
            // None,
            item.transform.translation.into(),
            item.transform.rotation.into(),
        )
    }
}

impl From<(&ecoord::TransformId, &ecoord::Transform)> for TransformStamped {
    fn from(item: (&ecoord::TransformId, &ecoord::Transform)) -> Self {
        let header = std_msgs::Header {
            stamp: item.1.timestamp.into(),
            frame_id: item.0.frame_id.clone().into(),
        };

        Self {
            header,
            child_frame_id: item.0.child_frame_id.clone().into(),
            transform: item.1.isometry().into(),
        }
    }
}

/// Implements the [`Twist`] message of ROS2.
///
/// [`Twist`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/Twist.msg
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Twist {
    pub linear: Vector3,
    pub angular: Vector3,
}

/// Implements the [`TwistWithCovariance`] message of ROS2.
///
/// [`TwistWithCovariance`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/TwistWithCovariance.msg
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct TwistWithCovariance {
    pub twist: Twist,
    #[serde(with = "BigArray")]
    pub covariance: [f64; 36],
}

/// Implements the [`Vector3`] message of ROS2.
///
/// [`Vector3`]: https://github.com/ros2/common_interfaces/blob/rolling/geometry_msgs/msg/Vector3.msg
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<Vector3> for nalgebra::Vector3<f64> {
    fn from(item: Vector3) -> Self {
        Self::new(item.x, item.y, item.z)
    }
}

impl From<Vector3> for nalgebra::Translation3<f64> {
    fn from(item: Vector3) -> Self {
        Self::new(item.x, item.y, item.z)
    }
}

impl From<nalgebra::Translation3<f64>> for Vector3 {
    fn from(item: nalgebra::Translation3<f64>) -> Self {
        Self {
            x: item.x,
            y: item.y,
            z: item.z,
        }
    }
}

impl From<nalgebra::Vector3<f64>> for Vector3 {
    fn from(item: nalgebra::Vector3<f64>) -> Self {
        Self {
            x: item.x,
            y: item.y,
            z: item.z,
        }
    }
}
