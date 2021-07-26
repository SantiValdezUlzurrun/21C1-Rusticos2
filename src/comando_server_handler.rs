use crate::base_de_datos::{BaseDeDatos, ResultadoRedis};
use crate::comando::ComandoHandler;
use crate::comando_info::ComandoInfo;
use crate::config::Config;
use std::sync::{Arc, Mutex};

pub type ComandoConConfig = Box<
    dyn FnOnce(&mut ComandoInfo, Arc<Mutex<BaseDeDatos>>, Arc<Mutex<Config>>) -> ResultadoRedis
        + 'static,
>;

pub struct ComandoServerHandler {
    comando: ComandoInfo,
    config: Arc<Mutex<Config>>,
    a_ejecutar: ComandoConConfig,
}

impl ComandoServerHandler {
    pub fn new(comando: ComandoInfo, config: Arc<Mutex<Config>>) -> Self {
        let a_ejecutar = match comando.get_nombre().as_str() {
            "DBSIZE" => dbsize,
            "CONFIG" => fconfig,
            "INFO" => info,
            "MONITOR" => monitor,
            _ => flushdb,
        };
        ComandoServerHandler {
            comando,
            config,
            a_ejecutar: Box::new(a_ejecutar),
        }
    }
}

impl ComandoHandler for ComandoServerHandler {
    fn ejecutar(mut self: Box<Self>, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&mut self.comando, bdd, self.config)
    }
}

#[allow(dead_code)]
pub fn es_comando_pubsub(comando: &str) -> bool {
    let comandos = vec!["FLUSHDB", "DBSIZE", "CONFIG"];
    comandos.iter().any(|&c| c == comando)
}

fn flushdb(
    _comando: &mut ComandoInfo,
    bdd: Arc<Mutex<BaseDeDatos>>,
    _config: Arc<Mutex<Config>>,
) -> ResultadoRedis {
    match bdd.lock() {
        Ok(mut b) => b.borrar_claves(),
        Err(_) => return ResultadoRedis::Error("Error al acceder a la base de datos".to_string()),
    };
    ResultadoRedis::StrSimple("OK".to_string())
}

fn dbsize(
    _comando: &mut ComandoInfo,
    bdd: Arc<Mutex<BaseDeDatos>>,
    _config: Arc<Mutex<Config>>,
) -> ResultadoRedis {
    let cantidad = match bdd.lock() {
        Ok(b) => b.cantidad_claves(),
        Err(_) => return ResultadoRedis::Error("Error al acceder a la base de datos".to_string()),
    };
    ResultadoRedis::Int(cantidad as isize)
}

fn fconfig(
    comando: &mut ComandoInfo,
    bdd: Arc<Mutex<BaseDeDatos>>,
    config: Arc<Mutex<Config>>,
) -> ResultadoRedis {
    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => return ResultadoRedis::Error("Parametro no encontrado".to_string()),
    };

    match parametro.as_str() {
        "GET" => config_get(comando, bdd, config),
        "SET" => config_set(comando, bdd, config),
        _ => ResultadoRedis::Error("Opcion de config no encontrada".to_string()),
    }
}

fn config_get(
    comando: &mut ComandoInfo,
    _bdd: Arc<Mutex<BaseDeDatos>>,
    config: Arc<Mutex<Config>>,
) -> ResultadoRedis {
    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => return ResultadoRedis::Error("Parametro no encontrado".to_string()),
    };

    let valores = match config.lock() {
        Ok(c) => c.get(&parametro),
        Err(_) => return ResultadoRedis::Error("Error al acceder a la configuracion".to_string()),
    };

    ResultadoRedis::Vector(
        valores
            .iter()
            .map(|x| ResultadoRedis::BulkStr(x.to_string()))
            .collect(),
    )
}

fn config_set(
    comando: &mut ComandoInfo,
    _bdd: Arc<Mutex<BaseDeDatos>>,
    config: Arc<Mutex<Config>>,
) -> ResultadoRedis {
    let (parametro, valor) = match (comando.get_parametro(), comando.get_parametro()) {
        (Some(p), Some(v)) => (p, v),
        _ => return ResultadoRedis::Error("Parametro no encontrado".to_string()),
    };

    match config.lock() {
        Ok(mut c) => c.set(parametro, valor),
        Err(_) => return ResultadoRedis::Error("Error al acceder a la configuracion".to_string()),
    };

    ResultadoRedis::StrSimple("Ok".to_string())
}

fn info(
    _comando: &mut ComandoInfo,
    bdd: Arc<Mutex<BaseDeDatos>>,
    config: Arc<Mutex<Config>>,
) -> ResultadoRedis {
    let info = match (config.lock(), bdd.lock()) {
        (Ok(c), Ok(b)) => {
            let mut v = c.info();
            v.append(&mut b.info());
            v
        }
        _ => return ResultadoRedis::Error("Error al acceder a la informacion".to_string()),
    };

    ResultadoRedis::Vector(
        info.iter()
            .map(|s| ResultadoRedis::BulkStr(s.to_string()))
            .collect(),
    )
}

fn monitor(
    _comando: &mut ComandoInfo,
    _bdd: Arc<Mutex<BaseDeDatos>>,
    config: Arc<Mutex<Config>>,
) -> ResultadoRedis {
    match config.lock() {
        Ok(mut c) => c.monitor(),
        Err(_) => return ResultadoRedis::Error("Error al acceder a la configuracion".to_string()),
    };
    ResultadoRedis::StrSimple("Ok".to_string())
}
