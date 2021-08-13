use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::canal::Canal;
use crate::cliente::Cliente;
use crate::comando::ComandoHandler;
use crate::comando_info::ComandoInfo;
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
            "UNSUBSCRIBE" => unsubscribe,
            "PUBLISH" => publish,
            "PUBSUB" => pubsub,
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
    let comandos = vec!["SUBSCRIBE", "UNSUBSCRIBE", "PUBLISH", "PUBSUB"];
    comandos.iter().any(|&c| c == comando)
}

fn subscribe(
    comando: &mut ComandoInfo,
    cliente: Cliente,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    let mut resultado = ResultadoRedis::Int(0);

    while let Some(clave) = comando.get_parametro() {
        let mut canal = match bdd.lock() {
            Ok(bdd) => match bdd.obtener_valor(&clave) {
                Some(TipoRedis::Canal(c)) => c.clone(),
                None => Canal::new(),
                _ => {
                    return ResultadoRedis::Error(
                        "WRONGTYPE Operation against a key holding the wrong kind of value"
                            .to_string(),
                    )
                }
            },
            Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
        };

        canal.suscribirse(cliente.clone());

        match bdd.lock() {
            Ok(mut bdd) => bdd.guardar_valor(clave, TipoRedis::Canal(canal)),
            Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
        }
        resultado = ResultadoRedis::Int(1);
    }
    resultado
}

fn unsubscribe(
    comando: &mut ComandoInfo,
    cliente: Cliente,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    let mut resultado = ResultadoRedis::Int(0);

    while let Some(clave) = comando.get_parametro() {
        let mut canal = match bdd.lock() {
            Ok(bdd) => match bdd.obtener_valor(&clave) {
                Some(TipoRedis::Canal(c)) => c.clone(),
                _ => {
                    return ResultadoRedis::Error(
                        "WRONGTYPE Operation against a key holding the wrong kind of value"
                            .to_string(),
                    )
                }
            },
            Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
        };

        canal.desuscribirse(cliente.clone());

        match bdd.lock() {
            Ok(mut bdd) => bdd.guardar_valor(clave, TipoRedis::Canal(canal)),
            Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
        }
        resultado = ResultadoRedis::Int(1);
    }
    resultado
}

fn publish(
    comando: &mut ComandoInfo,
    _cliente: Cliente,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    let mensaje = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'publish' command".to_string(),
            )
        }
    };

    let mut canal = match bdd.lock() {
        Ok(bdd) => match bdd.obtener_valor(&clave) {
            Some(TipoRedis::Canal(c)) => c.clone(),
            _ => {
                return ResultadoRedis::Error(
                    "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
                )
            }
        },
        Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
    };
    ResultadoRedis::Int(canal.publicar(mensaje) as isize)
}

fn pubsub(
    comando: &mut ComandoInfo,
    _cliente: Cliente,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        _ => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'pubsub' command".to_string(),
            )
        }
    };

    match clave.as_str() {
        "CHANNELS" => channels(comando, _cliente, bdd),
        "NUMSUB" => numsub(comando, _cliente, bdd),
        _ => {
            ResultadoRedis::Error("ERR wrong number of arguments for 'pubsub' command".to_string())
        }
    }
}

fn channels(
    comando: &mut ComandoInfo,
    _cliente: Cliente,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'channels' command".to_string(),
            )
        }
    };
    let canales: Vec<String> = match bdd.lock() {
        Ok(bdd) => bdd.canales_activos(&parametro),
        Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
    };
    ResultadoRedis::Vector(
        canales
            .iter()
            .map(|s| ResultadoRedis::BulkStr(s.to_string()))
            .collect(),
    )
}

fn numsub(
    comando: &mut ComandoInfo,
    _cliente: Cliente,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    let mut cantidades = Vec::new();
    while let Some(clave) = comando.get_parametro() {
        let canal = match bdd.lock() {
            Ok(bdd) => match bdd.obtener_valor(&clave) {
                Some(TipoRedis::Canal(c)) => c.clone(),
                _ => {
                    return ResultadoRedis::Error(
                        "WRONGTYPE Operation against a key holding the wrong kind of value"
                            .to_string(),
                    )
                }
            },
            Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
        };

        cantidades.push(canal.len() as isize);
    }
    ResultadoRedis::Vector(cantidades.iter().map(|i| ResultadoRedis::Int(*i)).collect())
}
