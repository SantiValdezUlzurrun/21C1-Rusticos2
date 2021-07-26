const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

pub fn key_tests() {
    test01_key();
}

fn test01_key() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_set"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_set"),
    };

    match redis::cmd("DEL").arg("key").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando del"),
    };

    match redis::cmd("SET").arg("key").arg("miValor").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando set"),
    };

    let copiado = redis::cmd("COPY")
        .arg("key")
        .arg("key_copia")
        .query(&mut con);
    assert_eq!(copiado, Ok(1));
}
