use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// some utilities that useful and convenience for dealing with data flow.

pub fn get_cookie(data: &str) -> HashMap<String, String> {
    let stop_word = ["path", "expires", "domain", "httponly"];
    let mut cookie = HashMap::new();
    let vals = data.split("::").collect::<Vec<&str>>();
    vals.into_iter().for_each(|val| {
        let v_str: Vec<&str> = val.split(";").map(|s| s.trim()).collect();
        v_str.into_iter().for_each(|pair| {
            let mut ind = true;
            let tmp: Vec<&str> = pair
                .split("=")
                .filter(|c| {
                    if !stop_word.contains(&c.to_lowercase().trim()) {
                        true
                    } else {
                        ind = false;
                        false
                    }
                })
                .collect();
            if ind {
                cookie.insert(tmp[0].to_string(), tmp[1..].join("="));
            }
        });
    });
    cookie
}

pub fn now() -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    
}


///generate hash 
pub(crate) fn hash<I>(salt: I) -> u64
where 
    I: std::iter::IntoIterator,
    I::Item: Hash,
{
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    salt.into_iter().for_each(|ele| ele.hash(&mut hasher));
     hasher.finish()
    
}

