mod comando;
mod parser;
mod redis;
mod log_handle;
use crate::redis::Redis;


fn main() {
    let host: &str = "127.0.0.1";
    let port: &str = "8080";
    let mut redis: Redis = Redis::new(host, port);
    match redis.iniciar() {
        Ok(_) => (),
        Err(_) => println!("Error al iniciar"),
    }
}
