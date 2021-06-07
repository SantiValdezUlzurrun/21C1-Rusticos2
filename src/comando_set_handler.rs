use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use crate::comando::{ComandoHandler, Comando, ResultadoRedis, TipoRedis};


pub struct ComandoSetHandler {
    comando: Vec<String>,
    a_ejecutar: Comando,
}

impl ComandoSetHandler {
    
    pub fn new(comando: Vec<String>) -> Self {

        let a_ejecutar = match comando[0].as_str() {
            "SADD" => sadd,
            "SCARD" => scard,
            "SISMEMBER" => sismember,
            "SMEMBERS" => smembers,
            _ => srem,
        };
        ComandoSetHandler {
            comando,
            a_ejecutar: Box::new(a_ejecutar),

        }
    }
}

impl ComandoHandler for ComandoSetHandler {

    fn ejecutar(self: Box<Self>, hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
        (self.a_ejecutar)(&self.comando, hash_map)
    }
}

pub fn es_comando_set(comando: &String) -> bool {
    
    let comandos = vec!["SADD", "SCARD"];
    comandos.iter().any(|&c| c == comando)
}


fn sadd(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    let a_agregar = match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Set(set)) => aggregar_al_set(&comando[2..], &mut set.clone()),
        None => aggregar_al_set(&comando[2..], &mut HashSet::new()),
        _ => return ResultadoRedis::Error("WrongTypeError error al obtener el set, valor guardado en la clave no es un Set".to_string()),
    };
    hash_map.lock().unwrap().insert(comando[1].clone(), TipoRedis::Set(a_agregar));
    ResultadoRedis::Int(comando[2..].len())
}

fn aggregar_al_set(valores: &[String], set : &mut HashSet<String>) -> HashSet<String>{

    for valor in valores.iter() {
       set.insert(valor.clone()); 
    }
    set.clone()
}


fn scard(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Set(set)) => ResultadoRedis::Int(set.len()),
        None => ResultadoRedis::Int(0),
        _ => ResultadoRedis::Error("WrongTypeError error al obtener el set, valor guardado en la clave no es un Set".to_string()),
    }
}

fn sismember(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Set(set)) => ResultadoRedis::Int(if set.contains(&comando[2]) {1} else {0}),
        None => ResultadoRedis::Int(0),
        _ => ResultadoRedis::Error("WrongTypeError error al obtener el set, valor guardado en la clave no es un Set".to_string()),
    }
}

fn smembers(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Set(set)) => {
            let mut vector = vec![];
            for valor in set.iter() {
                vector.push(ResultadoRedis::BulkStr(valor.clone()));
            }
            ResultadoRedis::Vector(vector)
        },
        None => ResultadoRedis::Vector(vec![]),
        _ => ResultadoRedis::Error("WrongTypeError error al obtener el set, valor guardado en la clave no es un Set".to_string()),
    }
}

fn srem(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis { 
    let a_agregar = match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Set(set)) => eliminar_del_set(&comando[2..], &mut set.clone()),
        None => return ResultadoRedis::Int(0),
        _ => return ResultadoRedis::Error("WrongTypeError error al obtener el set, valor guardado en la clave no es un Set".to_string()),
    };

    hash_map.lock().unwrap().insert(comando[1].clone(), TipoRedis::Set(a_agregar));
    ResultadoRedis::Int(comando[2..].len())
}


fn eliminar_del_set(valores: &[String], set : &mut HashSet<String>) -> HashSet<String>{

    for valor in valores.iter() {
       set.remove(valor); 
    }
    set.clone()
}
