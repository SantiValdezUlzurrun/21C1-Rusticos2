use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::ComandoHandler;
use crate::comando_info::ComandoInfo;
use crate::canal::Canal;
use crate::cliente::Cliente;
use std::sync::{Arc, Mutex};

pub type ComandoConCliente =
    Box<dyn FnOnce(&mut ComandoInfo, Cliente, Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis + 'static>;

pub struct ComandoPubSubHandler {
    cliente: Cliente,
    comando: ComandoInfo,
    a_ejecutar: ComandoConCliente,
}

impl ComandoPubSubHandler {
    pub fn new(comando: ComandoInfo, cliente: Cliente) -> Self {
        let a_ejecutar = match comando.get_nombre().as_str() {
            _ => subscribe,
        };
        ComandoPubSubHandler {
            cliente,
            comando,
            a_ejecutar: Box::new(a_ejecutar),
        }
    }
}

impl ComandoHandler for ComandoPubSubHandler {
    fn ejecutar(mut self: Box<Self>, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&mut self.comando, self.cliente, bdd)
    }
}

pub fn es_comando_pubsub(comando: &str) -> bool {
    let comandos = vec!["SUBSCRIBE"];
    comandos.iter().any(|&c| c == comando)
}

fn subscribe(comando: &mut ComandoInfo, cliente: Cliente, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {

    let mut resultado = ResultadoRedis::Int(0);

    while let Some(clave) = comando.get_parametro() {
        let mut canal = match bdd.lock().unwrap().obtener_valor(&clave) {
            Some(TipoRedis::Canal(c)) => c.clone(),
            None => Canal::new(),
            _ => return ResultadoRedis::Error("WrongType tipo de dato no es un canal".to_string()),
        };

        canal.suscribirse(cliente.clone());

        bdd.lock().unwrap().guardar_valor(clave,TipoRedis::Canal(canal));
        resultado = ResultadoRedis::Int(1);
    }
    resultado
}
