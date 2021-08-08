mod base_de_datos;
mod canal;
mod cliente;
mod cliente_http;
mod cliente_redis;
mod comando;
mod comando_http;
mod comando_info;
mod comando_key_handler;
mod comando_list_handler;
mod comando_pubsub_handler;
mod comando_server_handler;
mod comando_set_handler;
mod comando_string_handler;
mod config;
mod http_parser;
mod log_handler;
mod parser;
mod persistencia;
mod redis;
mod redis_error;
mod valor;

use std::env;

use crate::config::{obtener_configuracion, Config};
use crate::redis::Redis;

fn main() {
    let config = match env::args().last() {
        Some(ruta) => match obtener_configuracion(ruta) {
            Ok(config) => config,
            Err(_) => return,
        },
        None => Config::new(),
    };

    let mut redis: Redis = Redis::new(config);
    match redis.iniciar() {
        Ok(_) => (),
        Err(_) => println!("Error al iniciar"),
    };
}
