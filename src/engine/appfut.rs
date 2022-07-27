use crate::utils;
use crate::Client;
use std::collections::{HashMap, LinkedList};
use tokio::task::JoinHandle as Handle;

// H hash of T serve as index for T
// and linkedlist provodes positional infomation for certain hash H
pub(crate) struct AppFut {
    /// hash-value pairs
    pub data: HashMap<u64, Handle<()>>,
    /// an sorted(ascending) list storing hash and time stamp
    pub index: LinkedList<(u64, f64)>,
}

impl AppFut {
    /// create an instance
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
            index: LinkedList::new(),
        }
    }

    /// directly take an value out and feed it to Client
    /// and update `index`
    pub(crate) async fn direct_join(&mut self, mut ids: Vec<u64>) {
        let mut raw_results = Vec::with_capacity(ids.len());
        ids.iter().for_each(|id| {
            if let Some(item) = self.data.remove(id) {
                raw_results.push(item);
            }
        });
        let mut item_cached = LinkedList::new();
        while let Some(item) = self.index.pop_front() {
            if ids.contains(&item.0) {
                ids.retain(|&en| en != item.0);
            } else {
                item_cached.push_back(item);
            }
            if ids.is_empty() {
                break;
            }
        }
        item_cached.append(&mut self.index);
        self.index = item_cached;
        Client::join_all(raw_results).await;
    }

    /// execute results from `get_idel` and feed it to a callback
    /// and update `index`
    pub(crate) fn cancell(&mut self, gap: f64, capacity: usize) {
        let idels = self.get_idel(gap, capacity);
        if !idels.is_empty() {
            log::info!(
                "cancelling {} / {} for Response.",
                idels.len(),
                self.index.len() + idels.len(),
            );
            idels.into_iter().for_each(|idel| (&idel.2).abort());
            //.collect::<Vec<Handle<()>>>();
            //Client::join_all(tasks).await;
        }
    }

    /// execute results from `get_idel` and feed it to a callback
    /// and update `index`
    pub(crate) async fn all(&mut self, gap: f64, capacity: usize) {
        let idels = self.get_idel(gap, capacity);
        if !idels.is_empty() {
            log::info!(
                "cancelling {} / {} for Response.",
                idels.len(),
                self.index.len() + idels.len(),
            );
            let tasks = idels
                .into_iter()
                .map(|idel| idel.2)
                .collect::<Vec<Handle<()>>>();
            Client::join_all(tasks).await;
        }
    }

    /// inset an item and update `data` and `index`
    pub(crate) fn insert(&mut self, item: Handle<()>, hash: u64, stamp: f64) {
        self.data.insert(hash, item);
        let now = utils::now();
        self.index.push_back((hash, stamp));
        assert!(self.index.front().unwrap_or(&(0, 0.0)).1 < now);
    }

    /// get no more than `capacity`s idels that longer than `gap`
    pub(crate) fn get_idel(&mut self, gap: f64, capacity: usize) -> Vec<(u64, f64, Handle<()>)> {
        let now = utils::now();
        let mut items = Vec::with_capacity(capacity);
        while let Some(item) = self.index.pop_front() {
            if item.1 + gap < now && items.len() < capacity {
                let ele = (item.0, item.1, self.data.remove(&item.0).unwrap());
                items.push(ele);
            } else {
                self.index.push_front(item);
                break;
            }
        }
        if !items.is_empty() {
            log::debug!("Availible response to parse: {}", items.len());
        }
        items
    }
}
