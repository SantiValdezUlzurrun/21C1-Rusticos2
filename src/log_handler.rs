use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};

pub enum Mensaje {
    Info(String),
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
        while let Ok(mensaje) = self.receptor.recv() {
            match mensaje {
                Mensaje::Info(a_logear) => {
                    let mut archivo = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&self.ruta)
                        .unwrap();
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

    pub fn log(&self, mensaje: String) {
        self.log.send(Mensaje::Info(mensaje)).unwrap();
    }
}
