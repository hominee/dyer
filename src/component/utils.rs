use std::collections::HashMap;

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
