mod redis;
mod parser;
use crate::redis::Redis;

fn main() -> std::io::Result<()>  {
    
    let host : &str = "127.0.0.1";
    let port : &str = "8080";
    let redis : Redis = Redis::new(host, port);
    redis.iniciar();
	Ok(())
}


