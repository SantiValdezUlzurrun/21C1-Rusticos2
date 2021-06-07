use std::collections::{HashMap, HashSet, LinkedList};
use std::sync::{Arc, Mutex};

use crate::comando_string_handler::{es_comando_string, ComandoStringHandler};
use crate::comando_set_handler::{es_comando_set, ComandoSetHandler};

#[derive(Debug, PartialEq)]
pub enum ResultadoRedis {
    StrSimple(String),
    BulkStr(String),
    Int(usize),
    Vector(Vec<ResultadoRedis>),
    Error(String),
}

pub enum TipoRedis {
    Str(String),
    Lista(LinkedList<String>),
    Set(HashSet<String>),
}

pub trait ComandoHandler {
    fn ejecutar(
        self: Box<Self>,
        hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>,
    ) -> ResultadoRedis;
}

pub fn crear_comando_handler(comando: Vec<String>) -> Box<dyn ComandoHandler> {
    if es_comando_string(&comando[0]) {
        Box::new(ComandoStringHandler::new(comando))
    } else { //if es_comando_set(&comando[0]) {
        Box::new(ComandoSetHandler::new(comando))
    }/*else {
        ComandoListaHandler::new(comando);
    }*/
}

pub type Comando = Box<
    dyn FnOnce(&Vec<String>, Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis + 'static,
>;

/*
pub struct RedisLista {
    lista: LinkedList<RedisString>,
}
*/
