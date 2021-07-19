use crate::cliente::Cliente;
use crate::comando_info::ComandoInfo;
use std::sync::{Arc, Mutex};

use crate::base_de_datos::{BaseDeDatos, ResultadoRedis};
use crate::comando_key_handler::{es_comando_key, ComandoKeyHandler};
use crate::comando_list_handler::{es_comando_list, ComandoListHandler};
use crate::comando_pubsub_handler::ComandoPubSubHandler;
use crate::comando_set_handler::{es_comando_set, ComandoSetHandler};
use crate::comando_string_handler::{es_comando_string, ComandoStringHandler};

pub trait ComandoHandler {
    fn ejecutar(self: Box<Self>, hash_map: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis;
}

pub fn crear_comando_handler(comando: ComandoInfo, cliente: Cliente) -> Box<dyn ComandoHandler> {
    if es_comando_string(comando.get_nombre().as_str()) {
        Box::new(ComandoStringHandler::new(comando))
    } else if es_comando_set(comando.get_nombre().as_str()) {
        Box::new(ComandoSetHandler::new(comando))
    } else if es_comando_key(comando.get_nombre().as_str()) {
        Box::new(ComandoKeyHandler::new(comando))
    } else if es_comando_list(comando.get_nombre().as_str()) {
        Box::new(ComandoListHandler::new(comando))
    } else {
        Box::new(ComandoPubSubHandler::new(comando, cliente))
    }
}

pub type Comando =
    Box<dyn FnOnce(&mut ComandoInfo, Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis + 'static>;
