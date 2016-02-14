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
                if self.linebuf.len() + count >= 2048 {
                    // FIXME: Return an error instead?
                    panic!("Too much buffer used.");
                }
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
            None => {
                match self.linebuf.find("\n") {
                    Some(idx) => {
                        new_str = self.linebuf.clone();
                        split = new_str .split_at(idx);
                    },
                    None =>
                        return None
                }
            }
        }
        self.linebuf = split.1[1..].to_string().clone();
        Some(split.0.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path;
    use std::fs;
    use std::io;
    use std::io::BufRead;
    use std::io::Read;

    #[test]
    fn compare_to_bufreader() {
        let mut line_lines: Vec<String> = vec![];
        {
            let path = path::PathBuf::from("src/irc/test-fixtures/").join("irssi.log");
            let mut file = fs::File::open(path.as_path()).unwrap();
            let mut line_reader = LineReader::new();
            loop {
                match line_reader.read(&mut file) {
                    None => break,
                    Some(line) =>
                        line_lines.push(line)
                }
            }
        }
        let mut std_lines: Vec<String> = vec![];
        {
            let path = path::PathBuf::from("src/irc/test-fixtures/").join("irssi.log");
            let file = fs::File::open(path.as_path()).unwrap();
            let line_reader = io::BufReader::new(file);
            for line in line_reader.lines() {
                std_lines.push(line.unwrap());
            }
        }
        assert_eq!(line_lines, std_lines);
    }

    struct ArrayReader<'a> {
        d: &'a [u8],
        pos: usize
    }

    impl<'a> Read for ArrayReader<'a> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let start = self.pos;
            self.pos += buf.len();
            let mut c = 0;
            for (d, s) in buf.iter_mut().zip(self.d[start..self.pos].iter()) {
                *d = *s;
                c += 1;
            }
            Ok(c)
        }
    }

    #[test]
    #[should_panic]
    fn full_buffer() {
        let mut data = ArrayReader {
            d: &[0; 2048*10],
            pos: 0
        };
        let mut reader = LineReader::new();
        loop {
            reader.read(&mut data as &mut Read);
        }
    }
}
