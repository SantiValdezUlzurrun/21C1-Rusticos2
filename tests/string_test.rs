const REDIS_SERVER_IP: &str = "redis://127.0.0.1:8080/";

//use std::time::Duration;
use std::thread;
use std::thread::JoinHandle;

pub fn string_tests(handles: &mut Vec<JoinHandle<()>>) {
    test_set_y_get(handles);
    test_guardo_un_string_le_appendeo_otro_string_y_le_pregunto_su_logitud(handles);
    test_almaceno_numeros_y_opero_sobre_ellos(handles);
    test_almaceno_varios_string_simultaneamente(handles);

}

fn test_set_y_get(handles: &mut Vec<JoinHandle<()>>) {
    let handle = thread::spawn(move || {
        let client = match redis::Client::open(REDIS_SERVER_IP) {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_set"),
        };
        let mut con = match client.get_connection() {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_set"),
        };

        match redis::cmd("SET").arg("key").arg("foo").query(&mut con) {
            Ok(a) => a,
            Err(_) => return println!("Error en el comando set"),
        };

        let result = redis::cmd("GET").arg("key").query(&mut con);

        assert_eq!(result, Ok("foo".to_string()));
    });
    handles.push(handle);
}

fn test_guardo_un_string_le_appendeo_otro_string_y_le_pregunto_su_logitud(
    handles: &mut Vec<JoinHandle<()>>,
) {
    let handle = thread::spawn(move || {
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
            Err(_) => return println!("Error en el comando set"),
        };

        let result = redis::cmd("GET").arg("nombre").query(&mut con);
        assert_eq!(result, Ok("Mauricio Buzzone".to_string()));

        len = redis::cmd("STRLEN").arg("nombre").query(&mut con);
        assert_eq!(len, Ok(16));
    });
    handles.push(handle);
}

fn test_almaceno_numeros_y_opero_sobre_ellos(handles: &mut Vec<JoinHandle<()>>) {
    let handle = thread::spawn(move || {
        let client = match redis::Client::open(REDIS_SERVER_IP) {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_set"),
        };
        let mut con = match client.get_connection() {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_set"),
        };

        let mut contador = redis::cmd("INCRBY").arg("contador").arg("1").query(&mut con);
        assert_eq!(contador, Ok(1));

        for i in 2..=10 {
            contador = redis::cmd("INCRBY").arg("contador").arg("1").query(&mut con);
            assert_eq!(contador, Ok(i));
        }

        for i in 1..10 {
            contador = redis::cmd("DECRBY").arg("contador").arg("1").query(&mut con);
            assert_eq!(contador, Ok(10 - i));
        }
    });
    handles.push(handle);
}

fn test_almaceno_varios_string_simultaneamente(handles: &mut Vec<JoinHandle<()>>) {
    let handle = thread::spawn(move || {
        let client = match redis::Client::open(REDIS_SERVER_IP) {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_set"),
        };
        let mut con = match client.get_connection() {
            Ok(a) => a,
            Err(_) => return println!("No hubo conneccion test_set"),
        };

        match redis::cmd("MSET")
            .arg("hola_es").arg("HOLA")
            .arg("mundo_es").arg("MUNDO")
            .arg("hola_ig").arg("HELLO")
            .arg("mundo_ig").arg("WORD")
            .arg("hola_al").arg("HALLO")
            .arg("mundo_al").arg("WELT")
            .query(&mut con)
        {
            Ok(a) => a,
            Err(_) => return println!("Error en el comando set"),
        };
        

        let result = redis::cmd("MGET")
            .arg("hola_es")
            .arg("mundo_es")
            .arg("hola_ig")
            .arg("mundo_ig")
            .arg("hola_al")
            .arg("mundo_al")
            .query(&mut con);
        assert_eq!(result, Ok("Mauricio Buzzone".to_string()));

    });
    handles.push(handle);
}
