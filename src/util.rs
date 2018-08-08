// SPDX-License-Identifier:  MIT

use std;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::path::PathBuf;
use libudev;
use regex::Regex;

use sema::*;

pub fn hwaddr_valid<T: ToString>(hwaddr: &T) -> bool {
    use std::num::ParseIntError;

    let hwaddr_length_as_str = 17;
    let addr = hwaddr.to_string();

    if ! addr.is_ascii() {
        return false;
    }

    if addr.len() != hwaddr_length_as_str {
        return false;
    }

    let bytes: Vec<Result<u8, ParseIntError>> = addr.split(|c| c == ':' || c == '-').map(|s| u8::from_str_radix(s, 16)).collect();

    for b in bytes {
        if b.is_err() {
            return false;
        }
    }

    return true;
}

pub fn hwaddr_normalize<T: ToString>(hwaddr: &T) -> Result<String, Box<Error>> {
    let mut addr = hwaddr.to_string();

    if ! hwaddr_valid(&addr) {
        return Err(From::from("Failed to parse MAC address"));
    }

    if addr.find("-").is_some() {
        addr = addr.replace("-", ":")
    }

    addr.make_ascii_uppercase();
    Ok(addr)
}

pub fn hwaddr_from_event_device() -> Result<String, Box<Error>> {
    let udev = libudev::Context::new()?;
    let devpath = env::var("DEVPATH")?;
    let mut syspath = "/sys".to_string();

    syspath.push_str(&devpath);

    let attr = udev.device_from_syspath(&PathBuf::from(syspath))?.attribute_value("address").ok_or("Failed to get MAC Address")?.to_owned();
    let addr = hwaddr_normalize(&attr.to_str().ok_or("Failed to convert OsStr to String")?.to_string())?;

    Ok(addr)
}

pub fn get_prefix_from_file(path: &str) -> Result<String, Box<Error>> {
    let mut f = File::open(path)?;
    let mut content = String::new();

    f.read_to_string(&mut content)?;

    let re = Regex::new(r"net.ifnames.prefix=([[:alpha:]]+)")?;
    let prefix = match re.captures(&content) {
        Some(c) => c[1].to_string(),
        None => "".to_string()
    };

    if prefix == "eth" {
        return Err(From::from("Use of prefix \"eth\" is not allowed because it is the prefix used by the kernel"));
    }

    if prefix.len() > 14 {
        return Err(From::from("Prefix too long, maximum length of prefix is 14 characters"));
    }

    Ok(prefix)
}

pub fn exit_maybe_unlock(sema: Option<&mut Semaphore>, exit_code: i32) -> ! {
    if let Some(s) = sema {
        s.unlock();
    }

    std::process::exit(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hwaddr_valid_ok() {
        assert!(hwaddr_valid(&"11:22:33:44:55:66"));
    }

    #[test]
    fn hwaddr_valid_ok_dashed() {
        assert!(hwaddr_valid(&"11-22-33-44-55-66"));
    }

    #[test]
    #[should_panic]
    fn hwaddr_valid_invalid_chars() {
        assert!(hwaddr_valid(&"11-22-33-44-55-xx");)
    }

    #[test]
    #[should_panic]
    fn hwaddr_valid_invalid_range() {
        assert!(hwaddr_valid(&"ffff-33-44-55-66");)
    }

    #[test]
    #[should_panic]
    fn hwaddr_valid_invalid_long() {
        assert!(hwaddr_valid(&"11-22-33-44-55-66-77"));
    }

    #[test]
    #[should_panic]
    fn hwaddr_valid_invalid_short() {
        assert!(hwaddr_valid(&"52:54:00:52:1f"));
    }

    #[test]
    fn hwaddr_normalize_ok() {
        assert_eq!(hwaddr_normalize(&"52:54:00:52:1f:93").unwrap(), "52:54:00:52:1F:93");
    }

    #[test]
    fn hwaddr_normalize_ok_dashed() {
        assert_eq!(hwaddr_normalize(&"52-54-00-52-1f-93").unwrap(), "52:54:00:52:1F:93");
    }

    #[test]
    #[should_panic]
    fn hwaddr_normalize_invalid() {
        assert_eq!(hwaddr_normalize(&"xx:54:00:52:1f:93").unwrap(), "52:54:00:52:1F:93");
    }
}