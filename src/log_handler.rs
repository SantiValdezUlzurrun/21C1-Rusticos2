use crate::canal::Canal;
use crate::cliente::Cliente;
use crate::comando_info::ComandoInfo;
use crate::redis_error::RedisError;

use std::fs::OpenOptions;
use std::io::{Result, Write};
use std::sync::mpsc::{Receiver, Sender};

/// Representa un mensaje que puede enviar el Logger al LogHandler
pub enum Mensaje {
    /// El mensaje a loggear es el address del usuario y el comando
    InfoComando(String, ComandoInfo),
    /// El mensaje a loggear es el address del usuario y el error produciodo
    InfoError(String, RedisError),
    /// El mensaje a loggear es el address del usuario y su status de coneccion
    InfoConeccion(String, String),
    /// Setea en verbose al Manejador
    SetVerbose(bool),
    /// Suscribe al cliente en modo monitor
    Monitor(Cliente),
    /// Cambia el archivo donde se esta loggeando
    ArchivoALogear(String),
    /// Cierra el hilo donde corre el  manejador
    Cerrar,
}

/// Entidad que se encarga de correr en un hilo y loggear mensajes enviados por el logger
pub struct LogHandler {
    ruta: String,
    receptor: Receiver<Mensaje>,
    canal: Canal,
    tipo: Box<dyn TipoLog + Send>,
}

impl LogHandler {
    /// Instancia un manejador listo para recibir mensajes
    ///
    /// # Argumentos
    ///
    /// * `ruta` - string donde se va a loggear
    /// * `verbose` - define el tipo de logger
    /// * `receptor` - Receiver de mensajes asociado al channel del Logger
    pub fn new(ruta: String, receptor: Receiver<Mensaje>, verbose: bool) -> Self {
        LogHandler {
            ruta,
            receptor,
            canal: Canal::new("monitor".to_string()),
            tipo: set_verbose(verbose),
        }
    }

    /// Ejecuta al manejador esperando mensajes
    ///
    /// ```no_run
    /// let (tx_log, rx_log) = channel();
    ///
    /// let mut log_handler: LogHandler =
    ///    LogHandler::new(config.logfile(), rx_log, config.verbose());
    ///
    /// let hilo_log = thread::spawn(move || {
    ///      log_handler.logear();
    /// });
    /// ```
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

/// Entidad que se encarga de escribir en un archivo
pub struct LogEscritor;

impl TipoLog for LogEscritor {
    fn logear(&self, ruta: String, a_logear: String) -> Result<()> {
        escribir(&ruta, a_logear)
    }
}

/// Entidad que se encarga de escribir y mostrar por stdout
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

/// Representa al mensajero que se comunica con el manejador para loggear
pub struct Logger {
    log: Sender<Mensaje>,
}

impl Logger {

    /// Instancia un Logger para enviar mensajes
    ///
    /// # Argumentos
    ///
    /// * `log` - Sender de Mensaje asociado al channel de LogHandler
    pub fn new(log: Sender<Mensaje>) -> Self {
        Logger { log }
    }

    /// Envia el mensaje para logear el comando
    pub fn log_comando(&self, socket_addr: String, comando_info: ComandoInfo) {
        if self
            .log
            .send(Mensaje::InfoComando(socket_addr, comando_info))
            .is_ok()
        {}
    }

    /// Envia el mensaje para logear el error
    pub fn log_error(&self, socket_addr: String, error: RedisError) {
        if self
            .log
            .send(Mensaje::InfoError(socket_addr, error))
            .is_ok()
        {}
    }

    /// Envia el mensaje para logear la coneccion
    pub fn log_coneccion(&self, socket_addr: String, mensaje: String) {
        if self
            .log
            .send(Mensaje::InfoConeccion(socket_addr, mensaje))
            .is_ok()
        {}
    }

    /// Envia el mensaje para setear el manejador en verbose
    pub fn verbose(&self, verbose: bool) {
        if self.log.send(Mensaje::SetVerbose(verbose)).is_ok() {}
    }

    /// Envia el mensaje para monitorear al cliente
    pub fn monitorear(&self, cliente: Cliente) {
        if self.log.send(Mensaje::Monitor(cliente)).is_ok() {}
    }

    /// Envia el mensaje para cambiar el archivo donde se loggea
    pub fn archivo(&self, ruta_nueva: String) {
        if self.log.send(Mensaje::ArchivoALogear(ruta_nueva)).is_ok() {}
    }
}
