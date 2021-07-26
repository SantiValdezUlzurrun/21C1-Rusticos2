use std::thread;
use std::time::Duration;

const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

pub fn key_tests() {
    test01_key();
    test02_key();
    test03_key();
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

    let copiado = redis::cmd("EXISTS").arg("key").query(&mut con);
    assert_eq!(copiado, Ok(1));

    let copiado = redis::cmd("EXISTS").arg("AAA").query(&mut con);
    assert_eq!(copiado, Ok(0));

    let copiado = redis::cmd("RENAME")
        .arg("key")
        .arg("nueva_key")
        .query(&mut con);
    assert_eq!(copiado, Ok("Ok".to_string()));

    let copiado = redis::cmd("EXISTS").arg("nueva_key").query(&mut con);
    assert_eq!(copiado, Ok(1));
}

fn test02_key() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_set"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_set"),
    };

    match redis::cmd("DEL").arg("valores").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando del"),
    };

    let pusheados = redis::cmd("RPUSH")
        .arg("valores")
        .arg("5")
        .arg("4")
        .arg("1")
        .arg("2")
        .arg("3")
        .query(&mut con);
    assert_eq!(pusheados, Ok(5));

    let ordenados = redis::cmd("SORT").arg("valores").query(&mut con);
    assert_eq!(ordenados, Ok((1, 2, 3, 4, 5)));

    let ordenados_des = redis::cmd("SORT")
        .arg("valores")
        .arg("DESC")
        .query(&mut con);
    assert_eq!(ordenados_des, Ok((5, 4, 3, 2, 1)));

    let expiracion = redis::cmd("EXPIRE").arg("valores").arg(2).query(&mut con);
    assert_eq!(expiracion, Ok(1));

    thread::sleep(Duration::from_secs(3));

    let copiado = redis::cmd("EXISTS").arg("valores").query(&mut con);
    assert_eq!(copiado, Ok(0));
}

fn test03_key() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_set"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_set"),
    };

    match redis::cmd("DEL").arg("clave").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando del"),
    };

    let pusheados = redis::cmd("SET").arg("clave").arg("valor").query(&mut con);
    assert_eq!(pusheados, Ok("OK".to_string()));

    let expiracion = redis::cmd("EXPIRE").arg("clave").arg(3).query(&mut con);
    assert_eq!(expiracion, Ok(1));

    let copiado = redis::cmd("TTL").arg("clave").query(&mut con);
    assert_eq!(copiado, Ok(3));

    let copiado = redis::cmd("PERSIST").arg("clave").query(&mut con);
    assert_eq!(copiado, Ok(1));

    thread::sleep(Duration::from_secs(3));

    let copiado = redis::cmd("EXISTS").arg("clave").query(&mut con);
    assert_eq!(copiado, Ok(1));
}
