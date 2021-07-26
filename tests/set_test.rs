const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

pub fn set_tests() {
    test01();
}

fn test01() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_set"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test01_set"),
    };

    match redis::cmd("DEL").arg("miset").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando del"),
    };

    let seteados = redis::cmd("SADD")
        .arg("miset")
        .arg("Juan")
        .arg("Pablo")
        .arg("Santiago")
        .arg("Federico")
        .query(&mut con);

    assert_eq!(seteados, Ok(4));

    let len = redis::cmd("SCARD").arg("miset").query(&mut con);
    assert_eq!(len, Ok(4));

    let seteados = redis::cmd("SADD")
        .arg("miset")
        .arg("otroNombre")
        .query(&mut con);
    assert_eq!(seteados, Ok(1));

    let pertenece = redis::cmd("SISMEMBER")
        .arg("miset")
        .arg("otroNombre")
        .query(&mut con);
    assert_eq!(pertenece, Ok(1));

    let eliminado = redis::cmd("SREM")
        .arg("miset")
        .arg("otroNombre")
        .query(&mut con);
    assert_eq!(eliminado, Ok(1));
}
