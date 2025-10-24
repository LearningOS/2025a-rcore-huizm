use alloc::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

/// Bankers
pub struct Bankers {
    available: BTreeMap<usize, usize>,            // rid to num
    allocation: BTreeMap<usize, BTreeSet<usize>>, // tid to list of rid
    need: BTreeMap<usize, usize>,                 // tid to rid
    finish: BTreeMap<usize, bool>,                // tid to is_finished
}

impl Bankers {
    /// New
    pub fn new() -> Self {
        Self {
            available: BTreeMap::new(),
            allocation: BTreeMap::new(),
            need: BTreeMap::new(),
            finish: BTreeMap::new(),
        }
    }

    /// Increase resource
    pub fn increase_resource(&mut self, rid: usize, num: usize) {
        if let Some(v) = self.available.get_mut(&rid) {
            *v += num;
        } else {
            self.available.insert(rid, num);
        };
    }

    /// Is safe
    pub fn is_safe(&mut self, tid: usize, rid: usize, enabled: bool) -> Option<bool> {
        self.need.insert(tid, rid);
        self.finish.insert(tid, false);

        if !enabled || *self.available.get(&rid).unwrap() > 0 {
            Some(true)
        } else {
            // if no satisfying threads, deadlock, otherwise reject current request
            for (t, _) in self
                .finish
                .iter()
                .filter(|(t, r)| **t != tid && **r == false)
            {
                if let Some(r) = self.need.get(t) {
                    if *self.available.get(r).unwrap() > 0 {
                        return Some(false);
                    };
                } else {
                    // not waiting, can continue running
                    return Some(false);
                };
            }
            None
        }
    }

    /// Request grant
    pub fn request_grant(&mut self, tid: usize) {
        // remove from need
        let rid = self.need.remove(&tid).unwrap();

        // decrease from available
        *self.available.get_mut(&rid).unwrap() -= 1;

        // increase in allocation
        if let Some(ll) = self.allocation.get_mut(&tid) {
            ll.insert(rid);
        } else {
            let mut ll = BTreeSet::new();
            ll.insert(rid);
            self.allocation.insert(tid, ll);
        };
    }

    /// Release resource
    pub fn release_resource(&mut self, tid: usize, rid: usize) {
        // remove from allocation
        self.allocation.get_mut(&tid).unwrap_or(&mut BTreeSet::new()).remove(&rid);

        // put back to available
        *self.available.get_mut(&rid).unwrap() += 1;

        // remove from unfinished list if not contending anymore
        if self.need.get(&tid).is_none()
            && self.allocation.get(&tid).unwrap_or(&BTreeSet::new()).len() == 0
        {
            self.finish.remove(&tid);
        };
    }
}
