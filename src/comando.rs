use std::sync::{Arc, Mutex};

use crate::base_de_datos::{BaseDeDatos, ResultadoRedis};
use crate::comando_key_handler::ComandoKeyHandler;
use crate::comando_set_handler::{es_comando_set, ComandoSetHandler};
use crate::comando_string_handler::{es_comando_string, ComandoStringHandler};

pub trait ComandoHandler {
    fn ejecutar(self: Box<Self>, hash_map: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis;
}

pub fn crear_comando_handler(comando: Vec<String>) -> Box<dyn ComandoHandler> {
    if es_comando_string(&comando[0]) {
        return Box::new(ComandoStringHandler::new(comando));
    } else if es_comando_set(&comando[0]) {
        return Box::new(ComandoSetHandler::new(comando));
    }
    Box::new(ComandoKeyHandler::new(comando))
    //}
}

pub type Comando = Box<dyn FnOnce(&[String], Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis + 'static>;
