use rustc_serialize::json::{Json,Array};

pub fn path<'a>(json: &'a Json, path: &str) -> &'a Json {
    let parts = path.split(".");
    let mut cur = json;
    for p in parts {
        cur = match cur.as_object().unwrap().get(p) {
            Some(c) => c,
            None => panic!("Could not find {} in {:?} (lost at {})", path, json, p)
        }
    }
    cur
}

pub fn array<'a>(json: &'a Json, path: &str) -> &'a Array {
    match self::path(json, path).as_array() {
        Some(p) => p,
        None => panic!("{} in {:?} is not an array", path, json)
    }
}

pub fn string<'a>(json: &'a Json, path: &str) -> &'a str{
    match self::path(json, path).as_string() {
        Some(p) => p,
        None => panic!("{} in {:?} is not an array", path, json)
    }
}
