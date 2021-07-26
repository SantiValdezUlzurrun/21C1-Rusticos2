use crate::base_de_datos::BaseDeDatos;
use crate::cliente::Cliente;
use crate::log_handler::Logger;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::Mutex;

use regex::Regex;

pub enum ArchivoError {
    ArchivoInexistenteError,
}

#[derive(Debug)]
pub struct Config {
    mapa_config: HashMap<String, String>,
    monitorear_ultimo_cliente: bool,
}

impl Config {
    pub fn new() -> Self {
        let mut mapa_config = HashMap::new();
        mapa_config.insert("verbose".to_string(), "0".to_string());
        mapa_config.insert("host".to_string(), "127.0.0.1".to_string());
        mapa_config.insert("port".to_string(), "8080".to_string());
        mapa_config.insert("timeout".to_string(), "0".to_string());
        mapa_config.insert("dbfilename".to_string(), "dump.rb".to_string());
        mapa_config.insert("logfile".to_string(), "redis.log".to_string());

        Config {
            mapa_config,
            monitorear_ultimo_cliente: false,
        }
    }

    pub fn direccion(&self) -> String {
        let host = match self.mapa_config.get("host") {
            Some(h) => h.to_string(),
            None => "127.0.0.1".to_string(),
        };

        let port = match self.mapa_config.get("port") {
            Some(p) => p.to_string(),
            None => "8080".to_string(),
        };

        host + ":" + &port
    }

    pub fn timeout(&self) -> u64 {
        match self.mapa_config.get("timeout") {
            Some(t) => t.parse().unwrap_or(0),
            None => 0,
        }
    }

    pub fn dbfilename(&self) -> String {
        match self.mapa_config.get("dbfilename") {
            Some(d) => d.to_string(),
            None => "dump.rb".to_string(),
        }
    }

    pub fn logfile(&self) -> String {
        match self.mapa_config.get("logfile") {
            Some(l) => l.to_string(),
            None => "redis.log".to_string(),
        }
    }

    pub fn verbose(&self) -> bool {
        match self.mapa_config.get("verbose") {
            Some(t) => match t.parse::<u32>() {
                Ok(t) => t == 1,
                Err(_) => false,
            },
            None => false,
        }
    }

    pub fn get(&self, re: &str) -> Vec<String> {
        let regex = match Regex::new(re) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut vec = Vec::new();
        for (clave, valor) in &self.mapa_config {
            if regex.is_match(clave) {
                vec.push(clave.clone());
                vec.push(valor.clone());
            }
        }
        vec
    }

    pub fn set(&mut self, parametro: String, valor: String) {
        self.mapa_config.insert(parametro, valor);
    }

    pub fn info(&self) -> Vec<String> {
        let mut info = vec!["# Config".to_string(), "".to_string()];

        for (clave, valor) in &self.mapa_config {
            info.push(format!("{}:{}", clave, valor));
        }
        info
    }

    pub fn monitor(&mut self) {
        self.monitorear_ultimo_cliente = true;
    }

    pub fn actualizar(
        &mut self,
        logger: &Logger,
        cliente: Cliente,
        tabla: Arc<Mutex<BaseDeDatos>>,
    ) {
        self.actualizar_log(logger, cliente);
        self.actualizar_bdd(tabla);
    }

    pub fn actualizar_log(&mut self, logger: &Logger, cliente: Cliente) {
        if self.monitorear_ultimo_cliente {
            logger.monitorear(cliente);
            self.monitorear_ultimo_cliente = false;
        }
        logger.verbose(self.verbose());
        logger.archivo(self.logfile());
    }

    pub fn actualizar_bdd(&self, tabla: Arc<Mutex<BaseDeDatos>>) {
        if let Ok(b) = tabla.lock() {
            b.cambiar_archivo(self.dbfilename())
        }
    }
}

pub fn obtener_configuracion(ruta_archivo: String) -> Result<Config, ArchivoError> {
    let archivo = match File::open(ruta_archivo) {
        Ok(archivo) => archivo,
        Err(_) => return Err(ArchivoError::ArchivoInexistenteError),
    };

    let lector = BufReader::new(archivo);
    let mut lineas = lector.lines();
    let mut mapa = HashMap::new();

    while let Some(Ok(linea)) = lineas.next() {
        let argumento: Vec<&str> = linea.split(": ").collect();
        mapa.insert(argumento[0].to_string(), argumento[1].to_string());
    }

    Ok(Config {
        mapa_config: mapa,
        monitorear_ultimo_cliente: false,
    })
}
