mod comando;
mod log_handler;
mod parser;
mod redis;
mod persistencia;

use crate::log_handler::{LogHandler, Logger};
use crate::parser::parsear_int;
use crate::redis::Redis;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::mpsc::channel;
use std::thread;

pub enum ArchivoError {
    ArchivoInexistenteError,
    ArchivoIncompletoError,
}

#[derive(Debug)]
pub struct Config {
    verbose: bool,
    port: String,
    timeout: u32,
    dbfilename: String,
    logfile: String,
}

impl Config {
    fn new() -> Self {
        Config {
            verbose: false,
            port: "8080".to_string(),
            timeout: 0,
            dbfilename: "dump.rb".to_string(),
            logfile: "redis.log".to_string(),
        }
    }
}

fn obtener_configuracion(ruta_archivo: String) -> Result<Config, ArchivoError> {
    let archivo = match File::open(ruta_archivo) {
        Ok(archivo) => archivo,
        Err(_) => return Err(ArchivoError::ArchivoInexistenteError),
    };

    let lector = BufReader::new(archivo);
    let mut lineas = lector.lines();
    let mut config = Config::new();

    while let Some(Ok(linea)) = lineas.next() {
        let argumento: Vec<&str> = linea.split(": ").collect();

        if argumento[0] == "dbfilename" {
            config.dbfilename = argumento[1].to_string();
        } else if argumento[0] == "logfile" {
            config.logfile = argumento[1].to_string();
        } else if argumento[0] == "port" {
            config.port = argumento[1].to_string();
        } else {
            let valor = match parsear_int(argumento[1].to_string()) {
                Some(v) => v,
                None => return Err(ArchivoError::ArchivoIncompletoError),
            };
            if argumento[0] == "verbose" {
                config.verbose = valor == 1;
            } else {
                config.timeout = valor;
            }
        }
    }

    Ok(config)
}

fn main() {
    let config = match env::args().last() {
        Some(ruta) => match obtener_configuracion(ruta) {
            Ok(config) => config,
            Err(_) => return,
        },
        None => Config::new(),
    };

    let (tx, rx) = channel();
    let mut handler = LogHandler::new(config.logfile, rx);

    let logger = Logger::new(tx);

    let handle_log_handler = thread::spawn(move || {
        handler.logear();
    });

    let host: &str = "127.0.0.1";

    let mut redis: Redis = Redis::new(host, &config.port, config.verbose, config.timeout, logger);
    match redis.iniciar() {
        Ok(_) => (),
        Err(_) => println!("Error al iniciar"),
    };

    handle_log_handler.join().unwrap();
}
