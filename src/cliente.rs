
fn main() -> redis::RedisResult<()> {

    let client = redis::Client::open("redis://127.0.0.1:8080/")?;
    let mut con = client.get_connection()?;
    redis::cmd("SET").arg("key").arg("foo").query(&mut con)?; 
    let result = redis::cmd("GET").arg("key").query(&mut con);
    
    
    assert_eq!(result, Ok("foo".to_string()));
    Ok(())
}
