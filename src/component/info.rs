//! A structure that carries basic meta-data,
//! including its origin, stime stamp, privilege, encoding and so on.
//!
use crate::{plugin::deser::*, utils};
use http::Uri;
use serde::{Deserialize, Serialize};

/// basic meta data related 3 ranging
/// - identifier
///   `id`, `marker`, `from`
/// - time stamp
///    `gap`, `created`, `able`
/// - privilege
///    `rank`, `unique`, `used`
///
/// Some infomation must be specified, such as `marker`, `id`, and so on
///
/// Some can be set from other [Info], like `from`, `used`
///
/// and others are initialized as default, such as `created`, `able`, `encoding`,
#[derive(Deserialize, Debug, Serialize)]
pub struct Info {
    /// the actor it belongs to
    pub marker: String,
    /// identifier of the entity
    pub id: u64,
    /// uri that produces this `entity`
    #[serde(with = "serde_uri")]
    pub from: Uri,
    /// the priority of this entity, the higher the eariler get executed, 0 as default
    pub rank: i16,
    /// time duration to execute the task
    /// it remains 0 until it moves to response,
    pub gap: f64,
    /// the encoding when encoding uri
    // TODO: String is not enough make it enum
    pub encoding: String,
    /// remove duplicate `entity`, `true` as default
    pub unique: bool,
    /// numbers that this `entity` has used, by default,
    /// commonly the threshold is 2 for `Task` beyond which will ignore,
    /// customize it in `ArgApp`
    /// no restriction for `Affix`
    pub used: u32,
    /// meta data that the `entity` is created
    pub created: f64,
    /// timestamp in seconds by which `entity` is allowed to be executed
    /// it is, as default, allowed when created
    pub able: f64,
}

impl Clone for Info {
    fn clone(&self) -> Self {
        Self {
            marker: self.marker.clone(),
            id: self.id,
            from: self.from.clone(),
            rank: self.rank,
            gap: self.gap,
            encoding: self.encoding.clone(),
            unique: self.unique,
            used: self.used,
            created: utils::now(),
            able: self.able,
        }
    }
}

impl Default for Info {
    fn default() -> Self {
        let now = utils::now();
        Self {
            marker: "".into(),
            id: 0,
            from: Uri::default(),
            rank: 0,
            gap: 0.0,
            encoding: "utf-8".into(),
            unique: false,
            used: 0,
            created: now,
            able: now,
        }
    }
}

#[test]
fn test_info() {
    let info = Info::default();
    assert_eq!(info.unique, true);
    assert_eq!(info.used, 0);
    assert_eq!(info.rank, 0);
    assert_eq!(info.encoding, "utf-8".to_string());
}
