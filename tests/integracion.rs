mod string_test;
mod list_test;
use crate::string_test::string_tests;
use crate::list_test::list_tests;
use std::thread::JoinHandle;

pub const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

fn main() {
    let mut handles: Vec<JoinHandle<()>> = vec![];

    string_tests(&mut handles);
    list_tests(&mut handles);

    for handle in handles {
        handle.join().unwrap();
    }
}
