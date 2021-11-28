//! Instruction fields of [ArgApp] and its Configuration
//!
//! # `config` Configuration
//!
//! Some fields are required, some are not Configurable, some are optional, The following fields
//! are configurable
//!
//! ## ArgApp
//!
//! Argument | Type | Description
//! --- | --- | ---
//! **`skip`** | [bool] | `true` as default, use the files stroed in `data_dir` or not, if not a new
//! **`spawn_task_max`** | [usize] | `100` as default, the maximal length of spawned tasks
//! **`buf_task`** | [usize] | `10000` as default, the length of `Task` collected by `parser`s, exceeding which all `Task` will be stored into `data_dir/tasks/` for memory saving
//! **`round_entity`** | [usize] | `10` as default, the number of entities exceed which `process_entity` is called to consume them session will started all older files will be truncated.
//! **`data_dir`** | [String] | `data/` as default, the place to store or load files of `App` when reaching` rate.cycle`
//! **`nap`** | [f64] | `15.0` as default, the duration after which generated `Task` or `Affix` or recycled `Affix` become availible
//! **`join_gap`** | [f64] | `7.0` as default, the duration which the spawned task exceeds the executor is called to forcefully join it
//! **`round_req`** | [usize] | `10`, for more to see [ArgApp]
//! **`round_req_min`** | [usize] | `5`, for more to see [ArgApp]
//! **`round_req_max`** | [usize] | `77`, for more to see [ArgApp]
//! **`round_task`** | [usize] | `10` as default, for more to see [ArgApp]
//! **`round_task_min`** | [usize] | `7`, for more to see [ArgApp]
//! **`round_res`** | [usize] | `10` as default, for more to see [ArgApp]
//! **`round_errs`** | [usize] | `10` as default, the number of `Response` cannot be parsed, exceed which `process_entity` is called to consume them,
//!
//! ## ArgAffix
//!
//! Argument | Type | Description
//! --- | --- | ---
//! **`arg_affix.is_on`** | [bool] | `false` as defalut, enable affix customization or not, when true, `Affixor` must be implemented
//! **`arg_affix.affix_min`** | [usize] | `0` as default the minimal length of affix( including these in use or in future )
//! **`arg_affix.affix_max`** | [usize] | `0` as default the minimal length of affix( including, these in use or in future )
//!
//! ## ArgRate
//!
//! Argument | Type | Description
//! --- | --- | ---
//! **`rate.cycle`** | [f64] | 600.0 as default, the duration after which backup files of `App`
//! **`rate.load`** | [f64] | 99.0 as default, the load to be spawned in each `interval`,
//! **`rate.rate_low`** | [f64] | 0.333 as dafault, a value between 0-1.0 that lower the taks to be spawned, eg. the oringnal value is 12, rate_low is 0.33, the tasks to be spawned is 12.0 * 0.33 ~ 4.
//! **`rate.err`** | [usize] | the nubmer that erros of `Response` occurs, the default value is 0,
//! **`rate.interval`** | [f64] | the duration of time after which updating `ArgRate` `ArgApp`, the default value is 30.0,
//!
//! [ArgApp]: crate::engine::arg::ArgApp
//!
use crate::engine::vault::Vault;
use crate::utils;
use std::io::{BufRead, BufReader};

/// Arguments that control the [App] at runtime, including using history or not,  
/// [Task] [Affix] [Request] [Response] entities consuming and generating
/// There shall be an introduction to every member(maybe coming soon).
///
/// [Task]: crate::Task
/// [Affix]: crate::Affix
/// [Request]: crate::Request
/// [Response]: crate::Response
/// [App]: crate::App
#[derive(std::fmt::Debug)]
pub struct ArgApp {
    /// time tap added to created Tasks or Affixs
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
    /// minimal task(affix) consumed per round
    pub round_task_min: usize,
    /// consume response once upon a time
    pub round_res: usize,
    ///consume errs once upon a time
    pub round_errs: usize,
    ///consume Entity once upon a time
    pub round_entity: usize,
    /// use files in directory `data/` or not,
    /// set true as default
    pub skip: bool,
    /// control the task speed runtime
    pub(crate) rate: Vault<ArgRate>,
    /// control the affix workflow
    pub arg_affix: Option<ArgAffix>,
    /// directory that store history file
    pub data_dir: String,
}

impl ArgApp {
    /// create an instance of [ArgApp]
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
            round_errs: 10,
            round_entity: 10,
            skip: true,
            rate: Vault::new(ArgRate::new()),
            arg_affix: None,
            data_dir: "data/".into(),
        };
        arg.parse_config(None, false);
        arg
    }

    /// use [Affixor] or not
    ///
    /// [Affixor]: crate::ArgAffix
    pub fn affix_on(&self) -> bool {
        if let Some(ArgAffix { is_on: true, .. }) = self.arg_affix {
            return true;
        }
        false
    }

    /// set key-value pairs in [ArgApp]
    fn set(&mut self, key: &str, value: &str, fail_safe: bool) {
        match key {
            "nap" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.nap = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for nap: {}", value);
                } else {
                    panic!("Update Failed, invalid value for nap: {}", value);
                }
            }
            "join_gap" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.join_gap = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for join_gap: {}", value);
                } else {
                }
            }
            "round_req" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_req = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for round_req: {}", value);
                } else {
                    panic!("Update Failed, invalid value for round_req: {}", value);
                }
            }
            "round_req_min" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_req_min = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for round_req_min: {}", value);
                } else {
                    panic!("Update Failed, invalid value for round_req_min: {}", value);
                }
            }
            "round_req_max" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_req_max = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for round_req_max: {}", value);
                } else {
                    panic!("Update Failed, invalid value for round_req_max: {}", value);
                }
            }
            "buf_task" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.buf_task = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for buf_task: {}", value);
                } else {
                    panic!("Update Failed, invalid value for buf_task: {}", value);
                }
            }
            "spawn_task_max" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.spawn_task_max = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for spawn_task_max: {}", value);
                } else {
                    panic!("Update Failed, invalid value for spawn_task_max: {}", value);
                }
            }
            "round_task" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_task = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for round_task: {}", value);
                } else {
                    panic!("Update Failed, invalid value for round_task: {}", value);
                }
            }
            "round_task_min" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_task_min = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for round_task_min: {}", value);
                } else {
                    panic!("Update Failed, invalid value for round_task_min: {}", value);
                }
            }
            "round_res" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_res = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for round_res: {}", value);
                } else {
                    panic!("Update Failed, invalid value for round_res: {}", value);
                }
            }
            "round_errs" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_errs = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for round_errs: {}", value);
                } else {
                    panic!("Update Failed, invalid value for round_errs: {}", value);
                }
            }
            "round_entity" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.round_entity = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for round_entity: {}", value);
                } else {
                    panic!("Update Failed, invalid value for round_entity: {}", value);
                }
            }
            "skip" => {
                if let Ok(v) = value.parse::<bool>() {
                    self.skip = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for skip: {}", value);
                } else {
                    panic!("Update Failed, invalid value for skip: {}", value);
                }
            }
            "data_dir" => {
                if let Ok(v) = value.parse::<String>() {
                    self.data_dir = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for data_dir: {}", value);
                } else {
                    panic!("Update Failed, invalid value for data_dir: {}", value);
                }
            }
            "rate.cycle" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rate.as_mut().cycle = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for rate.cycle: {}", value);
                } else {
                    panic!("Update Failed, invalid value for rate.cycle: {}", value);
                }
            }
            "rate.interval" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rate.as_mut().interval = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for rate.interval: {}", value);
                } else {
                    panic!("Update Failed, invalid value for rate.interval: {}", value);
                }
            }
            "rate.load" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rate.as_mut().load = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for rate.load: {}", value);
                } else {
                    panic!("Update Failed, invalid value for rate.load: {}", value);
                }
            }
            "rate.remains" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.rate.as_mut().remains = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for rate.remains: {}", value);
                } else {
                    panic!("Update Failed, invalid value for rate.remains: {}", value);
                }
            }
            "rate.rate_low" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rate.as_mut().rate_low = v;
                } else if fail_safe {
                    log::error!("Update Failed, invalid value for rate.rate_low: {}", value);
                } else {
                    panic!("Update Failed, invalid value for rate.rate_low: {}", value);
                }
            }
            "arg_affix.is_on" => {
                if self.arg_affix.is_some() {
                    if let Ok(v) = value.parse::<bool>() {
                        self.arg_affix.as_mut().unwrap().is_on = v;
                    } else if fail_safe {
                        log::error!(
                            "Update Failed, invalid value for arg_affix.is_on: {}",
                            value
                        );
                    } else {
                        panic!(
                            "Update Failed, invalid value for arg_affix.is_on: {}",
                            value
                        );
                    }
                } else {
                    let mut arg = ArgAffix::new();
                    if let Ok(v) = value.parse::<bool>() {
                        arg.is_on = v;
                    } else if fail_safe {
                        log::error!(
                            "Update Failed, invalid value for arg_affix.is_on: {}",
                            value
                        );
                    } else {
                        panic!(
                            "Update Failed, invalid value for arg_affix.is_on: {}",
                            value
                        );
                    }
                    self.arg_affix = Some(arg);
                }
            }
            "arg_affix.affix_min" => {
                if self.arg_affix.is_some() {
                    if let Ok(v) = value.parse::<usize>() {
                        self.arg_affix.as_mut().unwrap().affix_min = v;
                    } else if fail_safe {
                        log::error!(
                            "Update Failed, invalid value for arg_affix.affix_min: {}",
                            value
                        );
                    } else {
                        panic!(
                            "Update Failed, invalid value for arg_affix.affix_in: {}",
                            value
                        );
                    }
                } else {
                    let mut arg = ArgAffix::new();
                    if let Ok(v) = value.parse::<usize>() {
                        arg.affix_min = v;
                    } else if fail_safe {
                        log::error!(
                            "Update Failed, invalid value for arg_affix.affix_min: {}",
                            value
                        );
                    } else {
                        panic!(
                            "Update Failed, invalid value for arg_affix.affix_in: {}",
                            value
                        );
                    }
                    self.arg_affix = Some(arg);
                }
            }
            "arg_affix.affix_max" => {
                if self.arg_affix.is_some() {
                    if let Ok(v) = value.parse::<usize>() {
                        self.arg_affix.as_mut().unwrap().affix_max = v;
                    } else if fail_safe {
                        log::error!(
                            "Update Failed, invalid value for arg_affix.affix_max : {}",
                            value
                        );
                    } else {
                        panic!(
                            "Update Failed, invalid value for arg_affix.affix_max: {}",
                            value
                        );
                    }
                } else {
                    let mut arg = ArgAffix::new();
                    if let Ok(v) = value.parse::<usize>() {
                        arg.affix_max = v;
                    } else if fail_safe {
                        log::error!(
                            "Update Failed, invalid value for arg_affix.affix_max : {}",
                            value
                        );
                    } else {
                        panic!(
                            "Update Failed, invalid value for arg_affix.affix_max: {}",
                            value
                        );
                    }
                    self.arg_affix = Some(arg);
                }
            }
            _ => {
                eprintln!("Unrecognizable or unnecessary variable: {}", key);
            }
        }
    }

    /// parse the dyer.cfg file and update the [ArgApp]
    /// not fail safe for the first time call in [ArgApp::new]
    /// fail safe after that
    pub fn parse_config(&mut self, path: Option<&str>, fail_safe: bool) {
        let fields = [
            "arg_affix.is_on",
            "arg_affix.affix_min",
            "arg_affix.affix_max",
            "rate.cycle",
            "rate.interval",
            "rate.load",
            "rate.remains",
            "rate.rate_low",
            "data_dir",
            "skip",
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
            "round_errs",
            "round_entity",
        ];
        let file = std::fs::File::open(path.unwrap_or("dyer.cfg")).unwrap();
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
    }

    fn init(&mut self) {
        if self.arg_affix.is_some() {
            if self.arg_affix.as_ref().unwrap().affix_min
                >= self.arg_affix.as_ref().unwrap().affix_max
            {
                self.arg_affix.as_mut().unwrap().affix_max =
                    self.arg_affix.as_ref().unwrap().affix_min * 3 + 1;
            }
        }
        if self.round_req_min >= self.round_req_max {
            self.round_req_max = self.round_req_min * 3 + 1;
        }
    }
}

/// To control the workflow of engine in dealing with [Affix]
/// including using affix or not, the amount to use/generate
///
/// [Affix]: crate::Affix
#[derive(std::fmt::Debug)]
pub struct ArgAffix {
    /// use affix customization or not
    pub is_on: bool,
    /// minimal cached affix number(including affixs used in `Request` that to be executed)
    pub affix_min: usize,
    /// maximal cached affix number(including affixs used in `Request` that to be executed)
    pub affix_max: usize,
}

impl ArgAffix {
    /// create an instance of [ArgAffix]
    pub fn new() -> Self {
        ArgAffix {
            is_on: false,
            affix_min: 0,
            affix_max: 0,
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

    /// backup the [Task] [Affix] [Request] for some time in case of interupt
    ///
    /// [Task]: crate::Task
    /// [Affix]: crate::Affix
    /// [Request]: crate::Request
    pub fn backup(&mut self) -> bool {
        if self.cycle_usage >= self.cycle {
            self.cycle_usage = self.cycle_usage.rem_euclid(self.cycle);
            return true;
        }
        false
    }

    /// decide the length of [Task] to be spawned
    ///
    /// [Task]: crate::Task
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
        log::trace!("Remains:{}, Delta: {}, Len: {}", self.remains, delta, len);
        self.remains = self.remains - (len as usize) + 1;
        if len > 0.0 {
            log::trace!("Only {} tasks are valid by rate control.", len);
        }
        if self.anchor_low <= now {
            len.ceil() as usize
        } else {
            (self.rate_low * len).ceil() as usize
        }
    }
}
