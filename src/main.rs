mod comando;
mod parser;
mod redis;
use crate::parser::parsear_int;
use crate::redis::Redis;
use std::env;
use std::fs::File;
pub enum ArchivoError {
    ArchivoInexistenteError,
    ArchivoIncompletoError,
}
use std::io::prelude::*;

use std::io::BufReader;
#[derive(Debug)]
pub struct Config {
    verbose: bool,
    port: u32,
    timeout: u32,
    dbfilename: String,
    logfile: String,
}

impl Config {
    fn new() -> Self {
        Config {
            verbose: false,
            port: 8080,
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
        } else {
            let valor = match parsear_int(argumento[1].to_string()) {
                Some(v) => v,
                None => return Err(ArchivoError::ArchivoIncompletoError),
            };
            if argumento[0] == "verbose" {
                config.verbose = valor == 1;
            } else if argumento[1] == "port" {
                config.port = valor;
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
    println!("{:?}", config);

    let host: &str = "127.0.0.1";
    let port: &str = "8080";
    let mut redis: Redis = Redis::new(host, port);
    match redis.iniciar() {
        Ok(_) => (),
        Err(_) => println!("Error al iniciar"),
    }
}
