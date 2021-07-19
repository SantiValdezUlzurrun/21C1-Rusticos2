use crate::comando_info::ComandoInfo;
use crate::redis::RedisError;
use std::fs::OpenOptions;
use std::io::Write;

use std::sync::mpsc::{Receiver, Sender};

pub enum Mensaje {
    InfoComando(String, ComandoInfo),
    InfoError(String, RedisError),
    InfoConeccion(String, String),
    Cerrar,
}

pub fn aplicar_funcion_log(
    ruta: &str,
    receptor: &std::sync::mpsc::Receiver<Mensaje>,
    funcion_log: fn(&str, String) -> Result<(), String>,
) {
    while let Ok(mensaje) = receptor.recv() {
        let a_logear = match mensaje {
            Mensaje::InfoComando(addr, comando_info) => addr + " " + &comando_info.descripcion(),

            Mensaje::InfoError(addr, error) => addr + " " + &error.to_string(),

            Mensaje::InfoConeccion(addr, mensaje) => addr + " " + &mensaje,

            Mensaje::Cerrar => break,
        };

        match funcion_log(ruta, a_logear) {
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

fn f_log_escritor(ruta: &str, a_logear: String) -> Result<(), String> {
    let mut archivo = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(ruta)
    {
        Ok(archivo) => archivo,
        Err(_) => return Err("ERROR al abrir el archivo".to_string()),
    };

    match writeln!(archivo, "{}", a_logear.as_str()) {
        Ok(_) => return Ok(()),
        Err(_) => return Err("ERROR al escribir en el archivo".to_string()),
    }
}

fn f_log_verbose(ruta: &str, a_logear: String) -> Result<(), String> {
    let mut archivo = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(ruta)
    {
        Ok(archivo) => archivo,
        Err(_) => return Err("ERROR al abrir el archivo".to_string()),
    };

    println!("{}", a_logear);

    match writeln!(archivo, "{}", a_logear.as_str()) {
        Ok(_) => return Ok(()),
        Err(_) => return Err("ERROR al escribir en el archivo".to_string()),
    }
}
pub trait LogHandler {
    fn logear(&mut self);
}

pub struct LogHandlerEscritor {
    ruta: String,
    receptor: Receiver<Mensaje>,
}

impl LogHandlerEscritor {
    pub fn new(ruta: String, receptor: Receiver<Mensaje>) -> LogHandlerEscritor {
        LogHandlerEscritor { ruta, receptor }
    }
}

impl LogHandler for LogHandlerEscritor {
    fn logear(&mut self) {
        aplicar_funcion_log(&self.ruta, &mut self.receptor, f_log_escritor);
    }
}

pub struct LogHandlerVerbose {
    ruta: String,
    receptor: Receiver<Mensaje>,
}

impl LogHandlerVerbose {
    pub fn new(ruta: String, receptor: Receiver<Mensaje>) -> LogHandlerVerbose {
        LogHandlerVerbose { ruta, receptor }
    }
}

impl LogHandler for LogHandlerVerbose {
    fn logear(&mut self) {
        aplicar_funcion_log(&self.ruta, &mut self.receptor, f_log_verbose);
    }
}

pub struct Logger {
    log: Sender<Mensaje>,
}

impl Logger {
    pub fn new(log: Sender<Mensaje>) -> Self {
        Logger { log }
    }

    pub fn log_comando(&self, socket_addr: String, comando_info: ComandoInfo) {
        self.log
            .send(Mensaje::InfoComando(socket_addr, comando_info))
            .unwrap();
    }

    pub fn log_error(&self, socket_addr: String, error: RedisError) {
        self.log
            .send(Mensaje::InfoError(socket_addr, error))
            .unwrap();
    }

    pub fn log_coneccion(&self, socket_addr: String, mensaje: String) {
        self.log
            .send(Mensaje::InfoConeccion(socket_addr, mensaje))
            .unwrap();
    }
}
