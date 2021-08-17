use crate::cliente::Cliente;
use crate::comando_info::ComandoInfo;
use crate::config::Config;
use crate::base_de_datos::{BaseDeDatos, ResultadoRedis};
use crate::comando_key_handler::{es_comando_key, ComandoKeyHandler};
use crate::comando_list_handler::{es_comando_list, ComandoListHandler};
use crate::comando_nulo_handler::ComandoNuloHandler;
use crate::comando_pubsub_handler::{es_comando_pubsub, ComandoPubSubHandler};
use crate::comando_server_handler::{es_comando_server, ComandoServerHandler};
use crate::comando_set_handler::{es_comando_set, ComandoSetHandler};
use crate::comando_string_handler::{es_comando_string, ComandoStringHandler};

use std::sync::{Arc, Mutex};

/// Interfaz publica que todos los manejadores de comando deben implementar
pub trait ComandoHandler {
    fn ejecutar(self: Box<Self>, hash_map: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis;
}

/// Instancia al manejador especifico con los parametros adecuados dependiendo del tipo de comando
pub fn crear_comando_handler(
    comando: ComandoInfo,
    cliente: Cliente,
    config: Arc<Mutex<Config>>,
) -> Box<dyn ComandoHandler> {
    if !cliente.soporta_comando(comando.get_nombre().as_str()) {
        Box::new(ComandoNuloHandler::new(comando))
    } else if es_comando_string(comando.get_nombre().as_str()) {
        Box::new(ComandoStringHandler::new(comando))
    } else if es_comando_set(comando.get_nombre().as_str()) {
        Box::new(ComandoSetHandler::new(comando))
    } else if es_comando_key(comando.get_nombre().as_str()) {
        Box::new(ComandoKeyHandler::new(comando))
    } else if es_comando_list(comando.get_nombre().as_str()) {
        Box::new(ComandoListHandler::new(comando))
    } else if es_comando_pubsub(comando.get_nombre().as_str()) {
        Box::new(ComandoPubSubHandler::new(comando, cliente))
    } else if es_comando_server(comando.get_nombre().as_str()) {
        Box::new(ComandoServerHandler::new(comando, config))
    } else {
        Box::new(ComandoNuloHandler::new(comando))
    }
}

/// Interfaz publica de como debe ser un comando redis
pub type Comando =
    Box<dyn FnOnce(&mut ComandoInfo, Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis + 'static>;
