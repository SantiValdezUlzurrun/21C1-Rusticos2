use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::comando::{ComandoHandler, Comando, ResultadoRedis, TipoRedis};

pub struct ComandoStringHandler {
    comando: Vec<String>,
    a_ejecutar: Comando,
}

impl ComandoStringHandler {
    
    pub fn new(comando: Vec<String>) -> Self {

        let a_ejecutar = match comando[0].as_str() {
            "GET" => get,
            _ => set,
        };
        ComandoStringHandler {
            comando ,
            a_ejecutar: Box::new(a_ejecutar),

        }
    }
}

impl ComandoHandler for ComandoStringHandler {

    fn ejecutar(self: Box<Self>, hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
        (self.a_ejecutar)(&self.comando, hash_map)
    }
}

pub fn es_comando_string(comando: &String) -> bool {
    
    let comandos = vec!["GET", "SET"];
    comandos.iter().any(|&c| c == comando)
}


fn get(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Str(valor)) => ResultadoRedis::BulkStr(valor.to_string()),
        _ => ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
    }
}

fn set(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
        hash_map
            .lock()
            .unwrap()
            .insert(comando[1].clone(), TipoRedis::Str(comando[2].clone()));
        ResultadoRedis::StrSimple("OK".to_string())
}
