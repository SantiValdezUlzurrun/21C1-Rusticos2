use crate::persistencia::levantar_tabla;
use crate::observer::Observable;
use crate::persistencia::Persistidor;
use crate::persistencia::MensajePersistencia;
use crate::persistencia::PersistidorHandler;
use crate::base_de_datos::{BaseDeDatos, ResultadoRedis};
use crate::cliente::crear_cliente;
use crate::cliente::{Cliente, Token};
use crate::comando::crear_comando_handler;
use crate::comando_info::ComandoInfo;
use crate::log_handler::Mensaje;
use crate::log_handler::{LogHandler, Logger};
use crate::redis_error::RedisError;
use crate::Config;

use std::net::TcpListener;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
extern crate redis;

pub struct Redis {
    config: Arc<Mutex<Config>>,
    bdd: Arc<Mutex<BaseDeDatos>>,
    siguiente_id: Token,
    tx_log: Sender<Mensaje>,
    hilo_log: Option<JoinHandle<()>>,
    tx_pers: Sender<MensajePersistencia>,
    hilo_pers: Option<JoinHandle<()>>,
    hilos_clientes: Vec<Option<JoinHandle<()>>>,
}

impl Redis {
    pub fn new(mut config: Config) -> Self {
        let (tx_log, rx_log) = channel();

        let mut log_handler: LogHandler = LogHandler::new(config.logfile(), rx_log, config.verbose());

        let hilo_log = thread::spawn(move || {
            log_handler.logear();
        });

        let (tx_pers, rx_pers) = channel();
        let mut pers_handler = PersistidorHandler::new(config.dbfilename(), 1, rx_pers);

        let hilo_pers = thread::spawn(move || {
            pers_handler.persistir();
        });

        let mut bdd = BaseDeDatos::new_con(levantar_tabla(config.dbfilename()));
        bdd.agregar_observador(Box::new(Persistidor::new(tx_pers.clone())));
        config.set_persistidor(Persistidor::new(tx_pers.clone()));

        Redis {
            config: Arc::new(Mutex::new(config)),
            bdd: Arc::new(Mutex::new(bdd)),
            siguiente_id: 0,
            tx_log,
            hilo_log: Some(hilo_log),
            tx_pers,
            hilo_pers: Some(hilo_pers),
            hilos_clientes: Vec::new(),
        }
    }

    pub fn iniciar(&mut self) -> Result<(), RedisError> {
        let direccion = match self.config.lock() {
            Ok(c) => c.direccion(),
            Err(_) => return Err(RedisError::Server),
        };

        let listener = match TcpListener::bind(direccion) {
            Ok(l) => l,
            Err(_) => return Err(RedisError::Inicializacion),
        };

        for stream in listener.incoming().flatten() {
            let clon_tabla = Arc::clone(&self.bdd);
            let clon_config = Arc::clone(&self.config);
            let logger = Logger::new(self.tx_log.clone());
            let timeout = match self.config.lock() {
                Ok(c) => c.timeout(),
                Err(_) => continue,
            };

            let mut cliente = crear_cliente(self.siguiente_id, timeout, stream);
            self.siguiente_id += 1;

            let handle = thread::spawn(move || {
                logger.log_coneccion(cliente.obtener_addr(), "Se conecto usario".to_string());
                match manejar_cliente(&mut cliente, clon_tabla, clon_config, &logger) {
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

        if self.tx_log.send(Mensaje::Cerrar).is_ok() {}

        if let Some(hilo) = self.hilo_log.take() {
            if hilo.join().is_ok() {}
        }

        if self.tx_pers.send(MensajePersistencia::Cerrar).is_ok() {}

        if let Some(hilo) = self.hilo_pers.take() {
            if hilo.join().is_ok() {}
        }
    }
}

fn manejar_cliente(
    cliente: &mut Cliente,
    tabla: Arc<Mutex<BaseDeDatos>>,
    config: Arc<Mutex<Config>>,
    logger: &Logger,
) -> Result<(), RedisError> {
    loop {
        if cliente.envio_informacion() {
            let comando = match cliente.obtener_comando() {
                Ok(Some(c)) => c,
                Ok(None) => continue,
                Err(e) => return Err(e),
            };
            logger.log_comando(cliente.obtener_addr(), comando.clone());

            let resultado = manejar_comando(
                comando,
                cliente.clone(),
                Arc::clone(&tabla),
                Arc::clone(&config),
            );

            match config.lock() {
                Ok(mut c) => c.actualizar(logger, cliente.clone()),
                Err(_) => return Err(RedisError::Server),
            }

            match cliente.enviar_resultado(&resultado) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        } else if !cliente.esta_conectado() {
            break;
        }
    }
    Ok(())
}

fn manejar_comando(
    entrada: ComandoInfo,
    cliente: Cliente,
    tabla: Arc<Mutex<BaseDeDatos>>,
    config: Arc<Mutex<Config>>,
) -> ResultadoRedis {
    let handler = crear_comando_handler(entrada, cliente, config);
    handler.ejecutar(tabla)
}

fn manejar_error(logger: &Logger, error: RedisError, cliente_addr: String) {
    logger.log_error(cliente_addr, error);
}
