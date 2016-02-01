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

use std::io::Read;
use std::str;

#[derive(Debug)]
pub struct LineReader {
    linebuf: String
}

impl LineReader {
    pub fn new() -> Self {
        LineReader {
            linebuf: String::new()
        }
    }

    pub fn read(&mut self, stream: &mut Read) -> Option<String> {
        match self.split_next_line() {
            None => self.read_and_split(stream),
            Some(line) => Some(line)
        }
    }

    fn read_and_split(&mut self, stream: &mut Read) -> Option<String> {
        let mut buf = [0; 1024];
        let next_msg = stream.read(&mut buf);
        match next_msg {
            Ok(count) => {
                self.linebuf.push_str(str::from_utf8(&buf[0..count]).unwrap());
                self.split_next_line()
            }
            Err(_) => None
        }
    }

    fn split_next_line(&mut self) -> Option<String> {
        let new_str;
        let split;
        match self.linebuf.find("\r\n") {
            Some(idx) => {
                new_str = self.linebuf.clone();
                split = new_str .split_at(idx);
            },
            None =>
                return None

        }
        self.linebuf = split.1[2..].to_string().clone();
        Some(split.0.to_string())
    }
}

