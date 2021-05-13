//! Instruction fields of [ArgApp] and its Configuration
//!
//! # `config` Configuration
//!
//! Some fields are required, some are not Configurable, some are optional, The following fields
//! are configurable 
//!
//! ## ArgApp
//!
//! **`is_skip`**: `bool`, `true` as default, use the files stroed in `data_dir` or not, if not a new
//!
//! **`spawn_task_max`**: `usize`, `100` as default, the maximal length of spawned tasks
//!
//! **`buf_task`**: `usize`, `10000` as default, the length of `Task` collected by `parser`s, exceeding
//! which all `Task` will be stored into `data_dir/tasks/` for memory saving
//!
//! **`round_entity`**: `usize`, `10` as default, the number of entities exceed which `process_entity`
//! is called to consume them
//! session will started all older files will be truncated.
//!
//! **`data_dir`**: `string`, `data/` as default, the place to store or load files of `App` when
//! reaching` rate.cycle`
//!
//! **`nap`**: `f64`, `15.0` as default, the duration after which generated `Task` or `Profile` or recycled `Profile` become
//! availible
//!
//! **`join_gap`**: `f64`, `7.0` as default, the duration which the spawned task exceeds the executor
//! is called to forcefully join it
//!
//! **`round_req`**: `usize`, `10`, for more to see [ArgApp]
//!
//! **`round_req_min`**: `usize`, `5`, for more to see [ArgApp]
//!
//! **`round_req_max`**: `uize`, `77`, for more to see [ArgApp]
//!
//! **`round_task`**: `usize`, `10` as default, for more to see [ArgApp]
//!
//! **`round_task_min`**: `usize`, `7`, for more to see [ArgApp]
//!
//! **`round_res`**: `usize`, `10` as default, for more to see [ArgApp]
//!
//! **`round_yield_err`**: `usize`, `10` as default, the number of `Response` cannot be parsed, exceed which `process_entity`
//! is called to consume them,
//! 
//! ## ArgProfile
//! 
//! **`arg_profile.is_on`**: `bool`, `false` as defalut, enable profile customization or not, when
//! true, `ProfileInfo.req` cannot be None, 
//!
//! **`arg_profile.profile_min`**: `usize` `0` as default, the minimal length of profile( including
//! these in use or in future )
//!
//! **`arg_profile.profile_max`**: `usize` `0` as default, the minimal length of profile( including,
//! these in use or in future )
//! 
//! ## ArgRate
//! 
//! **`rate.cycle`**: `f64`, 600.0 as default, the duration after which backup files of `App`
//!
//! **`rate.load`**: `f64`, 99.0 as default, the load to be spawned in each `interval`,
//!
//! **`rate.rate_low`**: `f64`, 0.333 as dafault, a value between 0-1.0 that lower the taks to be spawned, eg. the oringnal
//! value is 12, rate_low is 0.33, the tasks to be spawned is 12.0 * 0.33 ~ 4. 
//!
//! **`rate.err`**: `usize`, the nubmer that erros of `Response` occurs, the default value is 0,
//!
//! **`rate.interval`**: `f64`, the duration of time after which updating `ArgRate` `ArgApp`, the default
//! value is 30.0,
//!
//! [ArgApp]: crate::engine::arg::ArgApp
//!
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use crate::utils;

/// Arguments that control the `App` at runtime, including using history or not,  
/// `Task` `Profile` `Request` `Response` `Entity` consuming and generating
/// There shall be an introduction to every member(maybe coming soon).
#[derive(std::fmt::Debug)]
pub struct ArgApp {
    /// time tap added to created Tasks or Profiles
    pub nap: f64,
    /// gap to forcefully join the spawned task
    pub join_gap: f64,
    /// number that once for a concurrent future poll
    pub round_req: usize,
    /// cache request minimal length
    pub round_req_min: usize,
    /// cache request maximal length
    pub round_req_max: usize,
    /// buffer length for the created task.
    pub buf_task: usize,
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
    pub round_entity: usize,
    /// use files in directory `data/` or not,
    /// set true as default
    pub is_skip: bool,
    /// control the task speed runtime
    pub rate: Arc<Mutex<ArgRate>>,
    /// control the profile workflow
    pub arg_profile: Option<ArgProfile>,
    /// directory that store history file
    pub data_dir: String,
}

impl ArgApp {
    /// create an instance of `ArgApp`
    pub fn new() -> Self {
        let mut arg = ArgApp {
            nap: 17.0,
            join_gap: 7.0,
            round_req: 10,
            round_req_min: 3,
            round_req_max: 70,
            buf_task: 1000,
            spawn_task_max: 100,
            round_task: 10,
            round_task_min: 7,
            round_res: 10,
            round_yield_err: 10,
            round_entity: 10,
            is_skip: true,
            rate: Arc::new(Mutex::new(ArgRate::new())),
            arg_profile: None,
            data_dir: "data/".into(),
        };
        arg.parse_config(None, false);
        arg
    }

    /// set key-value pairs in `ArgApp`
    fn set(&mut self, key: &str, value: &str, fail_safe: bool) {
        match key {
            "nap" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.nap = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for nap: {}", value);
                }else {
                    panic!("update failed, invalid value for nap: {}", value);
                }
            }
            "join_gap" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.join_gap = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for join_gap: {}", value);
                }else {
                    panic!("update failed, invalid value for join_gap: {}", value);
                }
                
            }
            "round_req" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_req = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for round_req: {}", value);
                }else {
                    panic!("update failed, invalid value for round_req: {}", value);
                }
            }
            "round_req_min" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_req_min = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for round_req_min: {}", value);
                }else {
                    panic!("update failed, invalid value for round_req_min: {}", value);
                }
            }
            "round_req_max" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_req_max = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for round_req_max: {}", value);
                }else {
                    panic!("update failed, invalid value for round_req_max: {}", value);
                }
            }
            "buf_task" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.buf_task = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for buf_task: {}", value);
                }else {
                    panic!("update failed, invalid value for buf_task: {}", value);
                }
            }
            "spawn_task_max" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.spawn_task_max = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for spawn_task_max: {}", value);
                }else {
                    panic!("update failed, invalid value for spawn_task_max: {}", value);
                }
            }
            "round_task" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_task = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for round_task: {}", value);
                }else {
                    panic!("update failed, invalid value for round_task: {}", value);
                }
            }
            "round_task_min" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_task_min = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for round_task_min: {}", value);
                }else {
                    panic!("update failed, invalid value for round_task_min: {}", value);
                }
            }
            "round_res" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_res = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for round_res: {}", value);
                }else {
                    panic!("update failed, invalid value for round_res: {}", value);
                }
            }
            "round_yield_err" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_yield_err = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for round_yield_err: {}", value);
                }else {
                    panic!("update failed, invalid value for round_yield_err: {}", value);
                }
            }
            "round_entity" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_entity = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for round_entity: {}", value);
                }else {
                    panic!("update failed, invalid value for round_entity: {}", value);
                }
            }
            "is_skip" => {
                if let Ok(v) = value.parse::<bool>() {
                    self.is_skip = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for is_skip: {}", value);
                }else {
                    panic!("update failed, invalid value for is_skip: {}", value);
                }
            }
            "data_dir" => {
                if let Ok(v) = value.parse::<String>() {
                    self.data_dir = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for data_dir: {}", value);
                }else {
                    panic!("update failed, invalid value for data_dir: {}", value);
                }
            }
            "rate.cycle" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rate.lock().unwrap().cycle = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for rate.cycle: {}", value);
                }else {
                    panic!("update failed, invalid value for rate.cycle: {}", value);
                }
            }
            "rate.interval" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rate.lock().unwrap().interval = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for rate.interval: {}", value);
                }else {
                    panic!("update failed, invalid value for rate.interval: {}", value);
                }
            }
            "rate.load" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rate.lock().unwrap().load = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for rate.load: {}", value);
                }else {
                    panic!("update failed, invalid value for rate.load: {}", value);
                }
            }
            "rate.remains" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.rate.lock().unwrap().remains = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for rate.remains: {}", value);
                }else {
                    panic!("update failed, invalid value for rate.remains: {}", value);
                }
            }
            "rate.rate_low" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rate.lock().unwrap().rate_low = v;
                }else if fail_safe {
                    log::error!("update failed, invalid value for rate.rate_low: {}", value);
                }else {
                    panic!("update failed, invalid value for rate.rate_low: {}", value);
                }
            }
            "arg_profile.is_on" => {
                if self.arg_profile.is_some() {
                    if let Ok(v) = value.parse::<bool>() {
                        self.arg_profile.as_mut().unwrap().is_on = v;
                    }else if fail_safe {
                        log::error!("update failed, invalid value for arg_profile.is_on: {}", value);
                    }else {
                        panic!("update failed, invalid value for arg_profile.is_on: {}", value);
                    }
                } else {
                    let mut arg = ArgProfile::new();
                    if let Ok(v) = value.parse::<bool>() {
                        arg.is_on = v;
                    }else if fail_safe {
                        log::error!("update failed, invalid value for arg_profile.is_on: {}", value);
                    }else {
                        panic!("update failed, invalid value for arg_profile.is_on: {}", value);
                    }
                    self.arg_profile = Some(arg);
                }
            }
            "arg_profile.profile_min" => {
                if self.arg_profile.is_some() {
                    if let Ok(v) = value.parse::<usize>() {
                        self.arg_profile.as_mut().unwrap().profile_min = v;
                    }else if fail_safe {
                        log::error!("update failed, invalid value for arg_profile.profile_min: {}", value);
                    }else {
                        panic!("update failed, invalid value for arg_profile.profile_in: {}", value);
                    }
                } else {
                    let mut arg = ArgProfile::new();
                    if let Ok(v) = value.parse::<usize>() {
                        arg.profile_min = v;
                    }else if fail_safe {
                        log::error!("update failed, invalid value for arg_profile.profile_min: {}", value);
                    }else {
                        panic!("update failed, invalid value for arg_profile.profile_in: {}", value);
                    }
                    self.arg_profile = Some(arg);
                }
            }
            "arg_profile.profile_max" => {
                if self.arg_profile.is_some() {
                    if let Ok(v) = value.parse::<usize>() {
                        self.arg_profile.as_mut().unwrap().profile_max = v;
                    }else if fail_safe {
                        log::error!("update failed, invalid value for arg_profile.profile_max : {}", value);
                    }else {
                        panic!("update failed, invalid value for arg_profile.profile_max: {}", value);
                    }
                } else {
                    let mut arg = ArgProfile::new();
                    if let Ok(v) = value.parse::<usize>() {
                        arg.profile_max = v;
                    }else if fail_safe {
                        log::error!("update failed, invalid value for arg_profile.profile_max : {}", value);
                    }else {
                        panic!("update failed, invalid value for arg_profile.profile_max: {}", value);
                    }
                    self.arg_profile = Some(arg);
                }
            }
            _ => {
                eprintln!("Unrecognizable or unnecessary variable: {}", key);
            }
        }
    }

    /// parse the config file and update the `ArgApp`
    /// not fail safe for the first time call in `ArgApp::new`
    /// fail safe after that
    pub fn parse_config(&mut self, path: Option<&str>, fail_safe: bool) {
        let fields = [
            "arg_profile.is_on",
            "arg_profile.profile_min",
            "arg_profile.profile_max",
            "rate.cycle",
            "rate.interval",
            "rate.load",
            "rate.remains",
            "rate.rate_low",
            "data_dir",
            "is_skip",
            "nap",
            "join_gap",
            "round_req",
            "round_req_min",
            "round_req_max",
            "buf_task",
            "spawn_task_max",
            "round_task",
            "round_task_min",
            "round_res",
            "round_yield_err",
            "round_entity",
        ];
        let file = std::fs::File::open(path.unwrap_or("./config")).unwrap();
        let reader = BufReader::new(file);
        reader.lines().filter(|line| line.is_ok()).for_each(|line| {
            let pairs = line
                .unwrap()
                .split(":")
                .map(|ele| ele.to_string())
                .collect::<Vec<String>>();
            if pairs.len() == 2 {
                let key = pairs[0].trim();
                if fields.contains(&key) {
                    let value = pairs[1].trim().trim_end_matches(|c| c == ',');
                    self.set(key, value, fail_safe);
                }
            }
        });
        self.init();
        //println!("{:?}", self);
    }

    fn init(&mut self) {
        if self.arg_profile.is_some() {
            if self.arg_profile.as_ref().unwrap().profile_min
                >= self.arg_profile.as_ref().unwrap().profile_max
            {
                self.arg_profile.as_mut().unwrap().profile_max =
                    self.arg_profile.as_ref().unwrap().profile_min * 3 + 1;
            }
        }
        if self.round_req_min >= self.round_req_max {
            self.round_req_max = self.round_req_min * 3 + 1;
        }
    }
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
    /// create an instance of `ArgProfile`
    pub fn new() -> Self {
        ArgProfile {
            is_on: false,
            profile_min: 0,
            profile_max: 0,
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
    /// a time gap when updating some infomation
    pub interval: f64,
    /// normally the speed that the app spawns tasks in the whole interval
    pub load: f64,
    /// failed tasks in each interval
    pub err: usize,
    /// remaining jobs to do in each cycle in each interval
    pub remains: usize,
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
        let now = utils::now();
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
            stamps: Vec::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        let now = utils::now();
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
                self.remains = self.load as usize;
            }
            return true;
        }
        false
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
            None => utils::now(),
        };
        let delta = self.load * (self.anchor - now) / self.interval;
        let len = if self.remains as f64 >= delta + 0.5 && delta >= 0.0 {
            self.remains as f64 - delta
        } else if (self.remains as f64) < delta + 0.5 && delta >= 0.0 {
            self.remains = delta as usize;
            0.0
        } else {
            self.remains as f64
        };
        log::trace!("remains:{}, delta: {}, len: {}", self.remains, delta, len);
        self.remains = self.remains - (len as usize) + 1;
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
