use std::fmt;

/// Dedicated type for an identifier of a topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TopicId(i32);

impl fmt::Display for TopicId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TopicId> for i32 {
    fn from(item: TopicId) -> Self {
        item.0
    }
}

impl From<i32> for TopicId {
    fn from(item: i32) -> Self {
        Self(item)
    }
}

/// Dedicated type for an identifier of a channel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileName(String);

impl fmt::Display for FileName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<FileName> for String {
    fn from(item: FileName) -> Self {
        item.0
    }
}

impl From<String> for FileName {
    fn from(item: String) -> Self {
        Self(item)
    }
}

impl From<&str> for FileName {
    fn from(item: &str) -> Self {
        Self(item.to_string())
    }
}

/// Dedicated type for an identifier of a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkId(usize);

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ChunkId> for usize {
    fn from(item: ChunkId) -> Self {
        item.0
    }
}

impl From<usize> for ChunkId {
    fn from(item: usize) -> Self {
        Self(item)
    }
}

/// Dedicated type for an identifier of a channel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelTopic(String);

impl fmt::Display for ChannelTopic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ChannelTopic> for String {
    fn from(item: ChannelTopic) -> Self {
        item.0
    }
}

impl From<String> for ChannelTopic {
    fn from(item: String) -> Self {
        Self(item)
    }
}

impl From<&str> for ChannelTopic {
    fn from(item: &str) -> Self {
        Self(item.to_string())
    }
}

/// Dedicated type for an identifier of a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelId(u16);

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ChannelId> for u16 {
    fn from(item: ChannelId) -> Self {
        item.0
    }
}

impl From<u16> for ChannelId {
    fn from(item: u16) -> Self {
        Self(item)
    }
}

impl From<&u16> for ChannelId {
    fn from(item: &u16) -> Self {
        Self(*item)
    }
}

/// Dedicated type for an identifier of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageId(u16);

impl MessageId {
    /// The minimum possible value for a `MessageId`.
    pub const MIN: Self = MessageId(u16::MIN);

    /// The maximum possible value for a `MessageId`.
    pub const MAX: Self = MessageId(u16::MAX);
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<MessageId> for u16 {
    fn from(item: MessageId) -> Self {
        item.0
    }
}

impl From<u16> for MessageId {
    fn from(item: u16) -> Self {
        Self(item)
    }
}

impl From<MessageId> for usize {
    fn from(item: MessageId) -> Self {
        item.0 as usize
    }
}

impl From<usize> for MessageId {
    fn from(item: usize) -> Self {
        Self(item as u16)
    }
}
