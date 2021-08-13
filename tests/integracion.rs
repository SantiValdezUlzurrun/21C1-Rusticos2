mod key_test;
mod list_test;
mod set_test;
mod string_test;
mod pubsub_test;

use crate::key_test::key_tests;
use crate::list_test::list_tests;
use crate::set_test::set_tests;
use crate::string_test::string_tests;
use crate::pubsub_test::pubsub_tests;

pub const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

fn main() {
    string_tests();
    list_tests();
    key_tests();
    set_tests();
    pubsub_tests();
}
