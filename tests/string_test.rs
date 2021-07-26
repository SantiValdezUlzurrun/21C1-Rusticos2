const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

use redis::Value::Nil;

pub fn string_tests() {
    test_guardo_un_string_le_appendeo_otro_string_y_le_pregunto_su_logitud();
    test_set_y_get();
    test_almaceno_numeros_y_opero_sobre_ellos();
    test_almaceno_varios_string_simultaneamente();
    test_utilizo_getset_getdel_para_manejar_strings();
}

fn test_set_y_get() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_set"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_set"),
    };

    match redis::cmd("DEL").arg("key").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando del"),
    };

    match redis::cmd("SET").arg("key").arg("foo").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando set"),
    };

    let result = redis::cmd("GET").arg("key").query(&mut con);

    assert_eq!(result, Ok("foo".to_string()));
}

fn test_guardo_un_string_le_appendeo_otro_string_y_le_pregunto_su_logitud() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_set"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_set"),
    };

    match redis::cmd("SET")
        .arg("nombre")
        .arg("Mauricio")
        .query(&mut con)
    {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando set"),
    };

    let mut len = redis::cmd("STRLEN").arg("nombre").query(&mut con);
    assert_eq!(len, Ok(8));

    match redis::cmd("APPEND")
        .arg("nombre")
        .arg(" Buzzone")
        .query(&mut con)
    {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando append"),
    };

    let result = redis::cmd("GET").arg("nombre").query(&mut con);
    assert_eq!(result, Ok("Mauricio Buzzone".to_string()));

    len = redis::cmd("STRLEN").arg("nombre").query(&mut con);
    assert_eq!(len, Ok(16));
}

fn test_almaceno_numeros_y_opero_sobre_ellos() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_incrby_decrby"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_incrby_decrby"),
    };

    match redis::cmd("SET").arg("contador").arg("0").query(&mut con) {
        Ok(a) => a,
        Err(_) => return println!("Error en el comando set"),
    };

    let mut contador = redis::cmd("INCRBY")
        .arg("contador")
        .arg("1")
        .query(&mut con);
    assert_eq!(contador, Ok(1));

    contador = redis::cmd("INCRBY")
        .arg("contador")
        .arg("3")
        .query(&mut con);
    assert_eq!(contador, Ok(4));

    contador = redis::cmd("DECRBY")
        .arg("contador")
        .arg("1")
        .query(&mut con);
    assert_eq!(contador, Ok(3));
}

fn test_almaceno_varios_string_simultaneamente() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_mset_mget"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_mset_mget"),
    };

    match redis::cmd("MSET")
        .arg("es1")
        .arg("HOLA")
        .arg("es2")
        .arg("MUNDO")
        .arg("ig1")
        .arg("HELLO")
        .query(&mut con)
    {
        Ok(a) => a,
        Err(e) => return println!("{:?}", e.detail()), //println!("Error en el comando MSET"),
    };

    let result = redis::cmd("MGET")
        .arg("es1")
        .arg("es2")
        .arg("ig1")
        .arg("AAA")
        .query(&mut con);
    assert_eq!(
        result,
        Ok((
            "HOLA".to_string(),
            "MUNDO".to_string(),
            "HELLO".to_string(),
            Nil
        ))
    );
}

fn test_utilizo_getset_getdel_para_manejar_strings() {
    let client = match redis::Client::open(REDIS_SERVER_IP) {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_getset_getdel"),
    };
    let mut con = match client.get_connection() {
        Ok(a) => a,
        Err(_) => return println!("No hubo conneccion test_getset_getdel"),
    };

    match redis::cmd("SET")
        .arg("catedra")
        .arg("TallerDeProgramacion")
        .query(&mut con)
    {
        Ok(a) => a,
        Err(_) => return println!("ERROR EN EL SET"),
    }

    match redis::cmd("APPEND")
        .arg(&["catedra", "-CatedraDeymonnaz"])
        .query(&mut con)
    {
        Ok(a) => a,
        Err(_e) => return println!("Error en la funcion de append"),
    };

    let valor = redis::cmd("STRLEN").arg("catedra").query(&mut con);
    assert_eq!(valor, Ok(37));

    let valor = redis::cmd("GETDEL").arg("catedra").query(&mut con);
    assert_eq!(
        valor,
        Ok("TallerDeProgramacion-CatedraDeymonnaz".to_string())
    );
}
