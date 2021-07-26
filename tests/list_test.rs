const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

pub fn list_tests() {
    test_creo_una_lista_de_elementos_y_reviso_sus_elementos();
    test_creo_una_lista_y_opero_sobre_ellos();
}

fn test_creo_una_lista_de_elementos_y_reviso_sus_elementos() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_lpush"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_lpush"),
    };

    match redis::cmd("DEL").arg("valores").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando del"),
    };

    let pusheados = redis::cmd("LPUSH")
        .arg("valores")
        .arg("valor1")
        .arg("valor2")
        .arg("valor3")
        .query(&mut con);
    assert_eq!(pusheados, Ok(3));

    let pusheados = redis::cmd("LLEN").arg("valores").query(&mut con);
    assert_eq!(pusheados, Ok(3));

    let result = redis::cmd("LRANGE")
        .arg("valores")
        .arg(0)
        .arg(-1)
        .query(&mut con);
    assert_eq!(
        result,
        Ok((
            "valor3".to_string(),
            "valor2".to_string(),
            "valor1".to_string()
        ))
    );
}

fn test_creo_una_lista_y_opero_sobre_ellos() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_lpush"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_lpush"),
    };

    match redis::cmd("DEL").arg("elementos").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando del"),
    };

    let pusheados = redis::cmd("RPUSH")
        .arg("elementos")
        .arg("elemento1")
        .arg("elemento2")
        .arg("elemento3")
        .arg("elemento4")
        .arg("elemento5")
        .query(&mut con);
    assert_eq!(pusheados, Ok(5));

    let pusheados = redis::cmd("LLEN").arg("elementos").query(&mut con);
    assert_eq!(pusheados, Ok(5));

    let pusheados = redis::cmd("LPOP").arg("elementos").query(&mut con);
    assert_eq!(pusheados, Ok("elemento1".to_string()));

    let pusheados = redis::cmd("LLEN").arg("elementos").query(&mut con);
    assert_eq!(pusheados, Ok(4));

    let pusheados = redis::cmd("RPOP").arg("elementos").query(&mut con);
    assert_eq!(pusheados, Ok("elemento5".to_string()));

    let pusheados = redis::cmd("LLEN").arg("elementos").query(&mut con);
    assert_eq!(pusheados, Ok(3));

    let pusheados = redis::cmd("LSET")
        .arg("elementos")
        .arg(0)
        .arg("elementoA")
        .query(&mut con);
    assert_eq!(pusheados, Ok("OK".to_string()));

    let pusheados = redis::cmd("LINDEX").arg("elementos").arg(0).query(&mut con);
    assert_eq!(pusheados, Ok("elementoA".to_string()));

    let pusheados = redis::cmd("LLEN").arg("elementos").query(&mut con);
    assert_eq!(pusheados, Ok(4));

    let pusheados = redis::cmd("LSET")
        .arg("elementos")
        .arg(3)
        .arg("elementoB")
        .query(&mut con);
    assert_eq!(pusheados, Ok("OK".to_string()));

    let pusheados = redis::cmd("LINDEX").arg("elementos").arg(3).query(&mut con);
    assert_eq!(pusheados, Ok("elementoB".to_string()));

    let len = redis::cmd("RPUSH")
        .arg("elementos")
        .arg("elemento2")
        .arg("elemento2")
        .query(&mut con);
    assert_eq!(len, Ok(7));

    let eliminados = redis::cmd("LREM")
        .arg("elementos")
        .arg("-2")
        .arg("elemento2")
        .query(&mut con);

    let pusheados = redis::cmd("LLEN").arg("elementos").query(&mut con);
    assert_eq!(pusheados, Ok(5));

    assert_eq!(eliminados, Ok(2));
}
