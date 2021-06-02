use std::thread;

fn main() -> redis::RedisResult<()> {
   
    let mut handles = vec![];

    for _ in 0..10 {
        let handle = thread::spawn(move || {
            
            let client = match redis::Client::open("redis://127.0.0.1:8080/") {
                Ok(a) => a,
                Err(_) => return (),
            };
            let mut con = match client.get_connection() {
                Ok(a) => a,
                Err(_) => return (),
            };

            match redis::cmd("SET").arg("key").arg("foo").query(&mut con) {
                Ok(a) => a,
                Err(_) => return (),
            };

            let result = redis::cmd("GET").arg("key").query(&mut con);

            assert_eq!(result, Ok("foo".to_string()));
        });
        handles.push(handle);
    }
   
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
