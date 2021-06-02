use crate::comando::crear_comando;
use crate::comando::ResultadoRedis;
use crate::log_handler::Logger;
use crate::parser::parsear_respuesta;
use crate::parser::Parser;
use crate::log_handler::Mensaje;

use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
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
    tabla: Arc<Mutex<HashMap<String, String>>>,
    _verbose: bool,
    _timeout: u32,
    tx: Sender<Mensaje>,
}

impl Redis {
    pub fn new(host: &str, port: &str, _verbose: bool, _timeout: u32, tx: Sender<Mensaje>) -> Self {
        Redis {
            direccion: host.to_string() + ":" + port,
            tabla: Arc::new(Mutex::new(HashMap::new())),
            _verbose,
            _timeout,
            tx,
        }
    }

    pub fn iniciar(&mut self) -> Result<(), RedisError> {
        let listener = match TcpListener::bind(&self.direccion) {
            Ok(l) => l,
            Err(_) => return Err(RedisError::InicializacionError),
        };

        for mut stream in listener.incoming().flatten() {
            
            let clon_tabla = Arc::clone(&self.tabla);
            let logger = Logger::new(self.tx.clone());
            logger.log("Se conecto usario".to_string());
            thread::spawn(move || {
                match manejar_cliente(&mut stream, clon_tabla) {
                    Ok(()) => (),
                    Err(e) => manejar_error(&logger,e),
                };
                logger.log("se desconecto usuario".to_string());
            });
            
        }
        Ok(())
    }

  }

fn manejar_cliente(socket: &mut TcpStream, tabla: Arc<Mutex<HashMap<String, String>>>) -> Result<(), RedisError> {
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

fn manejar_comando(entrada: Vec<String>, tabla: Arc<Mutex<HashMap<String, String>>>) -> ResultadoRedis {
    let comando = crear_comando(&entrada);
    comando.ejecutar(tabla)
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

