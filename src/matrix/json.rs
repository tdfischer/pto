use rustc_serialize::json::{Json,Array};

pub fn path<'a>(json: &'a Json, path: &str) -> &'a Json {
    let parts = path.split(".");
    let mut cur = json;
    for p in parts {
        cur = cur.as_object().unwrap().get(p).unwrap();
    }
    cur
}

pub fn array<'a>(json: &'a Json, path: &str) -> &'a Array {
    self::path(json, path).as_array().unwrap()
}

pub fn string<'a>(json: &'a Json, path: &str) -> &'a str{
    self::path(json, path).as_string().unwrap()
}
