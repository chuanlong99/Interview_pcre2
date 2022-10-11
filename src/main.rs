#![allow(non_camel_case_types)]
mod ffi_wrapper;

const UDP_CLIENT_ADDR :&str = "127.0.0.1:20000";
const UDP_SERVER_ADDR :&str = "127.0.0.1:20080";
const UDP_SERVER_PORT :&str = "20080";

use ffi_wrapper::{pcre2_code_free_8, pcre2_compile_8, pcre2_get_ovector_pointer_8, pcre2_match_8,
                      pcre2_match_data_create_from_pattern_8, pcre2_match_data_free_8, PCRE2_UCP, PCRE2_UTF,
};
use std::net::UdpSocket;
use std::{ptr, thread, time};
use std::process::{Command};


fn match_with_regular<'a>(pattern: &'a str, strings: &'a str) -> Option<&'a str>{

    let mut error_code = 0;
    let mut error_offset = 0;

    //compile the pattern
    let code = unsafe {
        pcre2_compile_8(
            pattern.as_ptr(),
            pattern.len(),
            PCRE2_UCP | PCRE2_UTF,
            &mut error_code,
            &mut error_offset,
            ptr::null_mut(),
        )
    };
    if code.is_null() {
        println!(
            "compilation failed; error code: {:?}, offset: {:?}",
            error_code,
            error_offset
        );
        return None;
    }

    let match_data = unsafe {
        pcre2_match_data_create_from_pattern_8(code, ptr::null_mut())
    };
    if match_data.is_null() {
        unsafe {
            pcre2_code_free_8(code);
        }
        println!("could not allocate match_data");
        return None;

    }

    let ovector = unsafe { pcre2_get_ovector_pointer_8(match_data) };
    if ovector.is_null() {
        unsafe {
            pcre2_match_data_free_8(match_data);
            pcre2_code_free_8(code);
        }
        println!("could not get ovector");
        return None;

    }

    let rc = unsafe {
        pcre2_match_8(
            code,
            strings.as_ptr(),
            strings.len(),
            0,
            0,
            match_data,
            ptr::null_mut(),
        )
    };
    if rc <= 0 {
        unsafe {
            pcre2_match_data_free_8(match_data);
            pcre2_code_free_8(code);
        }
        println!("error executing match");
        return None;

    }

    let (s, e) = unsafe { (*ovector.offset(0), *ovector.offset(1)) };
    unsafe {
        pcre2_match_data_free_8(match_data);
        pcre2_code_free_8(code);
    }

    let s = s + 4;
    let e = e - 1;
    //println!("match result: {:?}", &strings[s..e]);
    Some(&strings[s..e])
}


fn main() {
    let pattern = r"\d{4}\D{3,11}\w";
    let strings = "a;jhgoqoghqoj0329 u0tyu10hg0h9Y0Y9827342482y(Y0y(G)_)lajf;lqjfgqhgpqjopjqa=)*(^!@#$%^&*())9999999";
    let expected_result = "y(Y";

    let result = match_with_regular(pattern, strings);
    let s = match result {
        None => {
            println!("cannot find any string match the pattern");
            return;
        }
        Some(s) => {
            s
        }
    };

    //run a udp server
    let handle = thread::spawn(move ||{
        let cmd = format!("{} {}", "nc -w 1 -lu", UDP_SERVER_PORT);
        let udp_server = Command::new("bash").arg("-c").arg(cmd).output().expect("cannot run udp server");
        let cmd_stdout = std::str::from_utf8(&udp_server.stdout).expect("cannot get stdout of udp server");
        println!("udp server received: {}", &cmd_stdout);
        assert_eq!(expected_result, cmd_stdout);
    });

    //sleeping to wait udp server start
    thread::sleep(time::Duration::from_secs(1));

    //send str to udp server
    let udp_client = UdpSocket::bind(String::from(UDP_CLIENT_ADDR));
    if let Ok(client) = udp_client {
        let _ = client.send_to(&s.as_bytes(), UDP_SERVER_ADDR).expect("cannot send data to udp server");
    }else {
        println!("something wrong with udp bind");
    }

    let _ = handle.join();
}
