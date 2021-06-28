use crate::comando_info::ComandoInfo;
use crate::redis::RedisError;
use std::fs::OpenOptions;
use std::io::Write;

use std::sync::mpsc::{Receiver, Sender};

pub enum Mensaje {
    InfoComando((String,ComandoInfo)),
    InfoError((String,RedisError)),
    InfoConeccion((String,String)),
    Cerrar,
}

pub struct LogHandler {
    ruta: String,
    receptor: Receiver<Mensaje>,
}

impl LogHandler {
    pub fn new(ruta: String, receptor: Receiver<Mensaje>) -> LogHandler {
        LogHandler { ruta, receptor }
    }

    pub fn logear(&mut self) {
        
        let mut archivo = match OpenOptions::new().write(true).append(true).create(true).open(&self.ruta){
            Ok(archivo) => archivo,
            Err(_) => return,// Para pensar :(
        };
                        
        while let Ok(mensaje) = self.receptor.recv() {
            match mensaje {
                Mensaje::InfoComando((addr,comando_info)) => {
                    let a_logear = addr +" "+ &comando_info.descripcion();

                    if let Err(e) = writeln!(archivo, "{}", a_logear.as_str()) {
                        println!("{:?}", e);
                    }
                }
                Mensaje::InfoError((addr,error)) => {

                    let a_logear = addr +" "+ &error.to_string();
                     if let Err(e) = writeln!(archivo, "{}", a_logear.as_str()) {
                        println!("{:?}", e);
                    }
                }

                Mensaje::InfoConeccion((addr,mensaje)) => {

                    let a_logear = addr +" "+ &mensaje;
                     if let Err(e) = writeln!(archivo, "{}", a_logear.as_str()) {
                        println!("{:?}", e);
                    }
                }

                Mensaje::Cerrar => break,
            };
        }
    }
}

pub struct Logger {
    log: Sender<Mensaje>,
}

impl Logger {
    pub fn new(log: Sender<Mensaje>) -> Self {
        Logger { log }
    }

    pub fn log_comando(&self, socket_addr: String,comando_info: ComandoInfo){
        self.log.send(Mensaje::InfoComando((socket_addr,comando_info))).unwrap();
    }

    pub fn log_error(&self, socket_addr: String,error: RedisError){
        self.log.send(Mensaje::InfoError((socket_addr,error))).unwrap();
    }

    pub fn log_coneccion(&self, socket_addr: String,mensaje: String){
        self.log.send(Mensaje::InfoConeccion((socket_addr,mensaje))).unwrap();
    }
}
