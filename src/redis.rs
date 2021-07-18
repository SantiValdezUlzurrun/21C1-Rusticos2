use crate::redis_error::RedisError;
use crate::base_de_datos::{BaseDeDatos, ResultadoRedis};
use crate::comando::crear_comando_handler;
use crate::comando_info::ComandoInfo;
use crate::log_handler::Mensaje;
use crate::log_handler::{LogHandler, Logger};
use crate::parser::parsear_respuesta;
use crate::parser::Parser;
use crate::Config;
use crate::cliente::{Token, Cliente};

use std::net::TcpListener;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
extern crate redis;


pub struct Redis {
    direccion: String,
    bdd: Arc<Mutex<BaseDeDatos>>,
    _verbose: bool,
    _timeout: u32,
    tx: Sender<Mensaje>,
    hilo_log: Option<JoinHandle<()>>,
    hilos_clientes: Vec<Option<JoinHandle<()>>>,
    siguiente_id: Token,
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
            siguiente_id: 0,
        }
    }

    pub fn iniciar(&mut self) -> Result<(), RedisError> {
        let listener = match TcpListener::bind(&self.direccion) {
            Ok(l) => l,
            Err(_) => return Err(RedisError::InicializacionError),
        };

        for stream in listener.incoming().flatten() {
            let clon_tabla = Arc::clone(&self.bdd);
            let logger = Logger::new(self.tx.clone());

            let mut cliente = Cliente::new(self.siguiente_id, stream);
            self.siguiente_id += 1;

            let handle = thread::spawn(move || {
                logger.log_coneccion(cliente.obtener_addr(), "Se conecto usario".to_string());
                match manejar_cliente(&mut cliente, clon_tabla, &logger) {
                    Ok(()) => (),
                    Err(e) => manejar_error(&logger, e, cliente.obtener_addr()),
                };

                logger.log_coneccion(cliente.obtener_addr(), "se desconecto usuario".to_string());
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
    cliente: &mut Cliente,
    tabla: Arc<Mutex<BaseDeDatos>>,
    logger: &Logger,
) -> Result<(), RedisError> {

    loop {
        if cliente.envio_informacion() {
            let stream = match cliente.obtener_socket() {
                Some(s) => s,
                None => return Err(RedisError::ConeccionError),
            };

            let parser = Parser::new(stream);

            let comando = match parser.parsear_stream() {
                Ok(orden) => orden,
                Err(_) => return Err(RedisError::ServerError),
            };

            logger.log_comando(cliente.obtener_addr(), comando.clone());

            let resultado = manejar_comando(comando, cliente.clone(), Arc::clone(&tabla));

            let respuesta = parsear_respuesta(&resultado);

            match cliente.enviar(respuesta) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }

        } else if !cliente.esta_conectado() {
            break;
        }
    }
    Ok(())
}

fn manejar_comando(entrada: ComandoInfo, cliente: Cliente, tabla: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let handler = crear_comando_handler(entrada, cliente);
    handler.ejecutar(tabla)
}

fn manejar_error(logger: &Logger, error: RedisError, cliente_addr: String) {
    logger.log_error(cliente_addr, error);
}
