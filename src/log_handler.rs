use crate::canal::Canal;
use crate::cliente::Cliente;
use crate::comando_info::ComandoInfo;
use crate::redis_error::RedisError;
use std::fs::OpenOptions;
use std::io::Result;
use std::io::Write;

use std::sync::mpsc::{Receiver, Sender};

pub enum Mensaje {
    InfoComando(String, ComandoInfo),
    InfoError(String, RedisError),
    InfoConeccion(String, String),
    #[allow(dead_code)]
    SetVerbose(bool),
    Monitor(Cliente),
    ArchivoALogear(String),
    Cerrar,
}

pub struct LogHandler {
    ruta: String,
    receptor: Receiver<Mensaje>,
    canal: Canal,
    tipo: Box<dyn TipoLog + Send>,
}

impl LogHandler {
    pub fn new(ruta: String, receptor: Receiver<Mensaje>, verbose: bool) -> Self {
        LogHandler {
            ruta,
            receptor,
            canal: Canal::new(),
            tipo: set_verbose(verbose),
        }
    }

    pub fn logear(&mut self) {
        while let Ok(mensaje) = self.receptor.recv() {
            let a_logear = match mensaje {
                Mensaje::InfoComando(addr, comando_info) => {
                    addr + " " + &comando_info.descripcion()
                }

                Mensaje::InfoError(addr, error) => addr + " " + &error.to_string(),

                Mensaje::InfoConeccion(addr, mensaje) => addr + " " + &mensaje,

                Mensaje::SetVerbose(b) => {
                    self.tipo = set_verbose(b);
                    continue;
                }

                Mensaje::Monitor(c) => {
                    self.canal.suscribirse(c);
                    continue;
                }

                Mensaje::ArchivoALogear(a) => {
                    self.ruta = a;
                    continue;
                }

                Mensaje::Cerrar => break,
            };

            self.canal.publicar(a_logear.clone());

            match self.tipo.logear(self.ruta.clone(), a_logear) {
                Ok(_) => (),
                Err(_) => break,
            }
        }
    }
}

fn set_verbose(verbose: bool) -> Box<dyn TipoLog + Send> {
    match verbose {
        true => Box::new(LogVerbose),
        false => Box::new(LogEscritor),
    }
}

trait TipoLog {
    fn logear(&self, ruta: String, a_logear: String) -> Result<()>;
}

pub struct LogEscritor;

impl TipoLog for LogEscritor {
    fn logear(&self, ruta: String, a_logear: String) -> Result<()> {
        escribir(&ruta, a_logear)
    }
}

pub struct LogVerbose;

impl TipoLog for LogVerbose {
    fn logear(&self, ruta: String, a_loguear: String) -> Result<()> {
        imprimir(a_loguear.clone());
        escribir(&ruta, a_loguear)
    }
}

fn escribir(ruta: &str, a_logear: String) -> Result<()> {
    let mut archivo = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(ruta)
    {
        Ok(archivo) => archivo,
        Err(e) => return Err(e),
    };

    writeln!(archivo, "{}", a_logear.as_str())
}

fn imprimir(a_logear: String) {
    println!("{}", a_logear);
}

pub struct Logger {
    log: Sender<Mensaje>,
}

impl Logger {
    pub fn new(log: Sender<Mensaje>) -> Self {
        Logger { log }
    }

    pub fn log_comando(&self, socket_addr: String, comando_info: ComandoInfo) {
        if self
            .log
            .send(Mensaje::InfoComando(socket_addr, comando_info))
            .is_ok()
        {}
    }

    pub fn log_error(&self, socket_addr: String, error: RedisError) {
        if self
            .log
            .send(Mensaje::InfoError(socket_addr, error))
            .is_ok()
        {}
    }

    pub fn log_coneccion(&self, socket_addr: String, mensaje: String) {
        if self
            .log
            .send(Mensaje::InfoConeccion(socket_addr, mensaje))
            .is_ok()
        {}
    }

    #[allow(dead_code)]
    pub fn verbose(&self, verbose: bool) {
        if self.log.send(Mensaje::SetVerbose(verbose)).is_ok() {}
    }

    pub fn monitorear(&self, cliente: Cliente) {
        if self.log.send(Mensaje::Monitor(cliente)).is_ok() {}
    }

    pub fn archivo(&self, ruta_nueva: String) {
        if self.log.send(Mensaje::ArchivoALogear(ruta_nueva)).is_ok() {}
    }
}
