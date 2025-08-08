use crate::TopicId;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopicPage {
    pub topics: HashMap<TopicId, Topic>,
}

impl TopicPage {
    pub fn new(topics: Vec<Topic>) -> Self {
        Self {
            topics: topics.into_iter().map(|x| (x.id, x)).collect(),
        }
    }

    /// Returns the total number of topics in this page.
    pub fn len(&self) -> usize {
        self.topics.len()
    }

    /// Checks if the page is empty.
    pub fn is_empty(&self) -> bool {
        self.topics.is_empty()
    }

    pub fn get_topic_ids(&self) -> HashSet<TopicId> {
        self.topics.keys().cloned().collect()
    }

    pub fn total_messages(&self) -> u64 {
        self.topics.values().map(|x| x.message_count).sum()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Topic {
    pub id: TopicId,
    pub name: String,
    pub message_count: u64,
}
