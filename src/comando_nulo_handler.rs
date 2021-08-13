use crate::base_de_datos::{BaseDeDatos, ResultadoRedis};
use crate::comando::{Comando, ComandoHandler};
use crate::comando_info::ComandoInfo;
use std::sync::{Arc, Mutex};

pub struct ComandoNuloHandler {
    comando: ComandoInfo,
    a_ejecutar: Comando,
}

impl ComandoNuloHandler {
    pub fn new(comando: ComandoInfo) -> Self {
        let a_ejecutar = comando_nulo;
        ComandoNuloHandler {
            comando,
            a_ejecutar: Box::new(a_ejecutar),
        }
    }
}

impl ComandoHandler for ComandoNuloHandler {
    fn ejecutar(mut self: Box<Self>, hash_map: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&mut self.comando, hash_map)
    }
}

fn comando_nulo(comando: &mut ComandoInfo, _bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    ResultadoRedis::Error(format!(
        "ComandoError '{}' no existe",
        comando.descripcion()
    ))
}
