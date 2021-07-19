const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

use std::thread;
use std::thread::JoinHandle;

pub fn string_tests(handles: &mut Vec<JoinHandle<()>>) {
    test_set_y_get(handles);
}

fn test_set_y_get(handles: &mut Vec<JoinHandle<()>>) {
    let handle = thread::spawn(move || {
        let client = match redis::Client::open(REDIS_SERVER_IP) {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_set"),
        };
        let mut con = match client.get_connection() {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_set"),
        };

        match redis::cmd("SET").arg("key").arg("foo").query(&mut con) {
            Ok(a) => a,
            Err(_) => return println!("Error en el comando set"),
        };

        let result = redis::cmd("GET").arg("key").query(&mut con);

        assert_eq!(result, Ok("foo".to_string()));
    });
    handles.push(handle);
}
