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
use c_ares;
use libc;
use hyper;
use std::mem;
use std::sync::{Mutex,Arc};

pub fn resolve_dns(domain: &str) -> Result<Option<(String, u16)>, c_ares::AresError> {
    let options = c_ares::Options::new();
    let running: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
    let mut ares = c_ares::Channel::new(options).expect("Could not create c-ares client");
    let result: Arc<Mutex<Result<Option<(String, u16)>, c_ares::AresError>>> = Arc::new(Mutex::new(Ok(None)));
    {
        let result = result.clone();
        let running = running.clone();
        ares.query_srv(domain, move |r| {
            trace!("Got DNS SRV result: {:?}", r);
            let mut data = result.lock().unwrap();
            *data = r.and_then(|srv| {
                match srv.iter().nth(0) {
                    None => Ok(None),
                    Some(result) =>
                        Ok(Some((result.host().to_string(), result.port())))
                }
            });
            *running.lock().unwrap() = false;
        });
    }
    loop {
        let mut read: c_ares::fd_set = unsafe {mem::transmute([0;32])};
        let mut write: c_ares::fd_set = unsafe {mem::transmute([0;32])};
        let mut err: c_ares::fd_set = unsafe {mem::transmute([0;32])};
        let mut timeout = libc::timeval{ tv_sec: 2, tv_usec: 0};
        ares.fds(&mut read, &mut write);
        unsafe {
            libc::select(1024, &mut read, &mut write, &mut err, &mut timeout);
        }
        ares.process(&mut read, &mut write);
        if !*running.lock().unwrap() {
            break
        }
    }
    let r = result.lock().unwrap();
    (*r).clone()
}

pub fn probe_url(domain: &str) -> Option<hyper::Url> {
    match resolve_dns(&*format!("_matrix._tcp.{}", domain)) {
        Err(_) => None,
        Ok(None) => None,
        Ok(Some((domain, port))) => {
            let url = format!("https://{}:{}/_matrix/", domain, port);
            Some(hyper::Url::parse(&*url).unwrap())
        }
    }
}
