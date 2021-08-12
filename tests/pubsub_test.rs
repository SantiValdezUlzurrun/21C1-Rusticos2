const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

pub fn pubsub_tests() {
    test01_pubsub();
}

fn test01_pubsub() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_pubsub"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_pubsub"),
    };

    let mut pubsub = con.as_pubsub();
    pubsub.subscribe("channel_1").unwrap();

    redis::cmd("PUBLISH").arg("channel_1").arg("HOLA");

    let msg = pubsub.get_message().unwrap();
    let payload: String = msg.get_payload().unwrap();
    println!("channel '{}': {}", msg.get_channel_name(), payload);
}
