const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

//use redis::Value::Nil;
use std::thread;
use std::thread::JoinHandle;

pub fn list_tests(_handles: &mut Vec<JoinHandle<()>>) {
	//test_lpush(handles);
}

#[allow(dead_code)]
fn test_lpush(handles: &mut Vec<JoinHandle<()>>){
	let handle = thread::spawn(move || {
        let client = match redis::Client::open(REDIS_SERVER_IP) {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_lpush"),
        };
        let mut con = match client.get_connection() {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_lpush"),
        };	

        match redis::cmd("DEL").arg("lista").query(&mut con) {
            Ok(a) => a,
            Err(_) => return println!("Error en el comando del"),
        };

        let pusheados = redis::cmd("LPUSH").arg("lista").arg("valor1").arg("valor2").arg("valor3").query(&mut con);
        assert_eq!(pusheados, Ok(3));

        let pusheados = redis::cmd("LLEN").arg("lista").query(&mut con);
        assert_eq!(pusheados, Ok(3));

        let result = match redis::cmd("LRANGE").arg("lista").arg(0).arg(-1).query(&mut con){
        	Ok(a) => a,
        	Err(e) => return println!("{:?}",e.detail())
        };
        println!("result {:?}",result)
        //assert_eq!(result, Ok(("valor1".to_string(),"valor2".to_string(),"valor3".to_string())));
    });
    handles.push(handle);
}