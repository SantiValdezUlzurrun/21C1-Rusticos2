use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{crear_comando_handler};
use crate::log_handler::Mensaje;
use crate::log_handler::{LogHandler, Logger};
use crate::parser::parsear_respuesta;
use crate::parser::Parser;
use crate::Config;

use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
extern crate redis;

pub enum RedisError {
    ServerError,
    ConeccionError,
    InicializacionError,
}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
           RedisError::ServerError => write!(f, "ServerError error del servidor"),
           RedisError::ConeccionError => write!(f, "ConeccionError no se ha podido establecer conexion"),
           RedisError::InicializacionError => write!(f, "InicializacionError no se ha podido inicializar el servidor en el puerto especificado"),
       }
    }
}

pub struct Redis {
    direccion: String,
    bdd: Arc<Mutex<BaseDeDatos>>,
    _verbose: bool,
    _timeout: u32,
    tx: Sender<Mensaje>,
    hilo_log: Option<JoinHandle<()>>,
    hilos_clientes: Vec<Option<JoinHandle<()>>>,
}

impl Redis {
    pub fn new(config: Config) -> Self {
        let (tx, rx) = channel();
        let mut handler = LogHandler::new(config.logfile, rx);

        let hilo_log = thread::spawn(move || {
            handler.logear();
        });

        Redis {
            direccion: config.host + ":" + config.port.as_str(),
            bdd: Arc::new(Mutex::new(BaseDeDatos::new(config.dbfilename))),
            _verbose: config.verbose,
            _timeout: config.timeout,
            tx,
            hilo_log: Some(hilo_log),
            hilos_clientes: Vec::new(),
        }
    }

    pub fn iniciar(&mut self) -> Result<(), RedisError> {
        let listener = match TcpListener::bind(&self.direccion) {
            Ok(l) => l,
            Err(_) => return Err(RedisError::InicializacionError),
        };

        for mut stream in listener.incoming().flatten() {
            let clon_tabla = Arc::clone(&self.bdd);
            let logger = Logger::new(self.tx.clone());
            let handle = thread::spawn(move || {
                logger.log("Se conecto usario".to_string());
                match manejar_cliente(&mut stream, clon_tabla) {
                    Ok(()) => (),
                    Err(e) => manejar_error(&logger, e),
                };
                logger.log("se desconecto usuario".to_string());
            });
            self.hilos_clientes.push(Some(handle));
        }
        Ok(())
    }
}

impl Drop for Redis {
    fn drop(&mut self) {
        for cliente in &mut self.hilos_clientes {
            if let Some(hilo_cliente) = cliente.take() {
                if hilo_cliente.join().is_ok() {}
            }
        }

        self.tx.send(Mensaje::Cerrar).unwrap();

        if let Some(hilo) = self.hilo_log.take() {
            if hilo.join().is_ok() {}
        }
    }
}

fn manejar_cliente(
    socket: &mut TcpStream,
    tabla: Arc<Mutex<BaseDeDatos>>,
) -> Result<(), RedisError> {
    let socket_clon = match socket.try_clone() {
        Ok(sock) => sock,
        _ => return Err(RedisError::ServerError),
    };
    loop {
        if cliente_envio_informacion(socket) {
            let parser = Parser::new(&socket_clon);

            let comando = match parser.parsear_stream() {
                Ok(orden) => orden,
                Err(_) => return Err(RedisError::ServerError),
            };

            let resultado = manejar_comando(comando, Arc::clone(&tabla));

            let respuesta = parsear_respuesta(&resultado);

            match socket.write(respuesta.as_bytes()) {
                Ok(_) => (),
                Err(_) => return Err(RedisError::ConeccionError),
            }
        } else if !cliente_esta_conectado(socket) {
            break;
        }
    }
    Ok(())
}

fn manejar_comando(
    entrada: Vec<String>,
    tabla: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    let handler = crear_comando_handler(entrada);
    handler.ejecutar(tabla)
}

fn cliente_envio_informacion(socket: &TcpStream) -> bool {
    match socket.peek(&mut [0; 128]) {
        Ok(len) => len > 0,
        Err(_) => false,
    }
}

fn cliente_esta_conectado(socket: &TcpStream) -> bool {
    match socket.peek(&mut [0; 128]) {
        Ok(len) => len != 0,
        Err(_) => false,
    }
}

fn manejar_error(logger: &Logger, error: RedisError) {
    logger.log(error.to_string());
}
