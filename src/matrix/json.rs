/*
 * Copyright 2015-2016 Torrie Fischer <tdfischer@hackerbots.net>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use rustc_serialize::json::{Json,Array};

pub fn path<'a>(json: &'a Json, path: &str) -> &'a Json {
    let parts = path.split(".");
    let mut cur = json;
    for p in parts {
        cur = match cur.as_object().unwrap().get(p) {
            Some(c) => c,
            None => panic!("Could not find {} in {} (lost at {})", path, json.pretty(), p)
        }
    }
    cur
}

pub fn array<'a>(json: &'a Json, path: &str) -> &'a Array {
    match self::path(json, path).as_array() {
        Some(p) => p,
        None => panic!("{} in {} is not an array", path, json.pretty())
    }
}

pub fn string<'a>(json: &'a Json, path: &str) -> &'a str{
    match self::path(json, path).as_string() {
        Some(p) => p,
        None => panic!("{} in {} is not an array", path, json.pretty())
    }
}
