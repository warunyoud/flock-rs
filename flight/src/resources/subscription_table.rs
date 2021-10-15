use std::collections::{HashMap, BTreeSet};

#[derive(Debug)]
struct SubscriptionHashMap {
    subscriptions: HashMap<String, BTreeSet<String>>,    
}

impl SubscriptionHashMap {
    fn new() -> SubscriptionHashMap {
        SubscriptionHashMap {
            subscriptions: HashMap::new()
        }
    }

    fn get(&self, key: &str) -> Vec<&String> {
        match self.subscriptions.get(key) {
            Some(set) => set.into_iter().collect(),
            None => vec![]
        }
    }

    fn insert(&mut self, key: &str, value: &str) -> bool {
        let mut row_change = false;
        if !self.subscriptions.contains_key(key) {
            row_change = true;
            self.subscriptions.insert(key.to_string(), BTreeSet::new());

        }

        if let Some(subscription) = self.subscriptions.get_mut(key) {
            subscription.insert(value.to_string());
        }
        row_change
    }

    fn remove(&mut self, key: &str, value: &str) -> bool {
        let mut row_change = false;
        if let Some(set) = self.subscriptions.get_mut(key) {
            set.remove(value);

            if set.is_empty() {
                self.subscriptions.remove(key);
                row_change = true;
            }
        }
        row_change
    }

    fn remove_all(&mut self, key: &str) {
        self.subscriptions.remove(key);
    }
}

#[derive(Debug)]
pub struct SubscriptionTable {
    topic_subscriptions: SubscriptionHashMap,
    socket_subscriptions: SubscriptionHashMap
}

impl SubscriptionTable {
    pub fn new() -> SubscriptionTable {
        SubscriptionTable {
            topic_subscriptions: SubscriptionHashMap::new(),
            socket_subscriptions: SubscriptionHashMap::new()   
        }
    }

    pub fn get(&self, topic: &str) -> Vec<&String> {
        self.topic_subscriptions.get(topic)
    }

    pub fn insert(&mut self, socket_id: &str, topic: &str) -> bool {
        self.socket_subscriptions.insert(socket_id, topic);
        self.topic_subscriptions.insert(topic, socket_id)
    }

    pub fn remove(&mut self, socket_id: &str, topic: &str) -> bool {
        self.socket_subscriptions.remove(socket_id, topic);
        self.topic_subscriptions.remove(topic, socket_id)
    }

    pub fn remove_all(&mut self, socket_id: &str) -> Vec<String> {
        let mut completely_remove_topics = Vec::new();

        let topics_to_remove = self.socket_subscriptions.get(socket_id);
        for topic in topics_to_remove {
            if self.topic_subscriptions.remove(&topic, socket_id) {
                completely_remove_topics.push(topic.to_string());
            }
        }
        self.socket_subscriptions.remove_all(socket_id);
        completely_remove_topics
    }
}
