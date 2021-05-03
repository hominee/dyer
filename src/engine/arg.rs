use std::sync::{Arc, Mutex};

/// Arguments that control the `App` at runtime, including using history or not,  
/// `Task` `Profile` `Request` `Response` `Entity` consuming and generating
/// There shall be an introduction to every member(maybe coming soon).
pub struct ArgApp {
    /// time tap added to created Tasks or Profiles
    pub gap: u64,
    /// gap to forcefully join the spawned task
    pub join_gap: u64,
    /// gap to forcefully join the spawned task if none of items meeting join_gap
    pub join_gap_emer: f64,
    /// number that once for a concurrent future poll
    pub round_req: usize,
    /// cache request minimal length
    pub round_req_min: usize,
    /// cache request maximal length
    pub round_req_max: usize,
    /// buffer length for the created task.
    pub buf_task_tmp: usize,
    /// maximal spawned task that cached
    pub spawn_task_max: usize,
    /// construct req from task one time
    pub round_task: usize,
    /// minimal task(profile) consumed per round
    pub round_task_min: usize,
    /// consume response once upon a time
    pub round_res: usize,
    ///consume yield_err once upon a time
    pub round_yield_err: usize,
    ///consume Entity once upon a time
    pub round_result: usize,
    pub skip_history: bool,
    /// control the task speed runtime
    pub rate: Arc<Mutex<ArgRate>>,
    /// control the profile workflow
    pub arg_profile: Option<ArgProfile>,
    /// directory that store history file
    pub data_dir: String,
}

/// To control the workflow of engine in dealing with `Profile`
/// including using profile or not, the amount to use/generate
#[derive(std::fmt::Debug)]
pub struct ArgProfile {
    /// use profile customization or not
    pub is_on: bool,
    /// minimal cached profile number(including profiles used in `Request` that to be executed)
    pub profile_min: usize,
    /// maximal cached profile number(including profiles used in `Request` that to be executed)
    pub profile_max: usize,
}

impl ArgProfile {
    pub fn new() -> Self {
        ArgProfile {
            is_on: false,
            profile_min: 3,
            profile_max: 10,
        }
    }
}

impl ArgApp {
    pub fn new() -> Self {
        ArgApp {
            gap: 15,
            join_gap: 7,
            join_gap_emer: 0.1,
            round_req: 10,
            round_req_min: 3,
            round_req_max: 70,
            buf_task_tmp: 10000,
            spawn_task_max: 100,
            round_task: 10,
            round_task_min: 7,
            round_res: 10,
            round_yield_err: 10,
            round_result: 10,
            skip_history: true,
            rate: Arc::new( Mutex::new( ArgRate::new() ) ),
            arg_profile: None,
            data_dir: "data/".to_string(),
        }
    }
}

/// some infomation about `dyer` at rumtime where speed and error-handler based on
#[derive(std::fmt::Debug)]
pub struct ArgRate {
    /// all time the app runs
    pub uptime: f64,
    /// the time that a cycle lasts, backup application history once running out
    pub cycle: f64,
    /// time the app runs in each cycle
    pub cycle_usage: f64,
    /// between 0-1, the rate that low mode lasts in each period
    pub period_threshold: f64,
    /// a time gap when updating some infomation
    pub interval: f64,
    /// normally the speed that the app spawns tasks in the whole interval
    pub load: f64,
    /// failed tasks in each interval
    pub err: u64,
    /// remaining jobs to do in each cycle in each interval
    pub remains: u64,
    /// the rate applied to limit the requests to be spawned in low mode
    pub rate_low: f64,
    /// time anchor by which the mode is low
    pub anchor_low: f64,
    /// time anchor at which update some infomation
    pub anchor: f64,
    /// vector of gap each request takes to receive response header in each interval  
    pub stamps: Vec<f64>,
}

impl ArgRate {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        ArgRate {
            uptime: 0.0,
            cycle: 600.0,
            cycle_usage: 0.0,
            load: 99.0,
            remains: 110,
            rate_low: 0.333,
            anchor_low: 0.0,
            err: 0,
            anchor: now + 30.0,
            interval: 30.0,
            period_threshold: 0.168,
            stamps: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        if now > self.anchor {
            self.cycle_usage += self.interval;
            self.anchor += self.interval;
            self.uptime += self.interval;
            if self.err > 0 {
                self.anchor_low = now + self.err as f64 * 0.333;
            }
            if now >= self.anchor_low {
                log::debug!("active period");
                self.stamps.clear();
                self.remains = self.load as u64;
            }
        }
    }

    /// backup the `Task` `Profile` `Request` for some time in case of interupt
    pub fn backup(&mut self) -> bool {
        if self.cycle_usage >= self.cycle {
            self.cycle_usage = self.cycle_usage.rem_euclid(self.cycle);
            return true;
        }
        false
    }

    /// decide the length of `Task` to be spawned
    pub fn get_len(&mut self, tm: Option<f64>) -> usize {
        let now = match tm {
            Some(now) => now,
            None => std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };
        let delta = self.load * (self.anchor - now) / self.interval;
        let len = if self.remains as f64 >= delta + 0.5 && delta >= 0.0 {
            self.remains as f64 - delta
        } else if (self.remains as f64) < delta + 0.5 && delta >= 0.0 {
            self.remains = delta as u64;
            0.0
        } else {
            self.remains as f64
        };
        log::trace!("remains:{}, delta: {}, len: {}", self.remains, delta, len);
        self.remains = self.remains - (len as u64) + 1;
        if len > 0.0 {
            log::trace!("only {} tasks are valid by rate control.", len);
        }
        if self.anchor_low <= now {
            len.ceil() as usize
        } else {
            (self.rate_low * len).ceil() as usize
        }
    }
}
