use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{Comando, ComandoHandler};
use crate::comando_info::ComandoInfo;
use std::sync::{Arc, Mutex};

pub struct ComandoListHandler {
    comando: ComandoInfo,
    a_ejecutar: Comando,
}

impl ComandoListHandler {
    pub fn new(comando: ComandoInfo) -> Self {
        let a_ejecutar = match comando.get_nombre().as_str() {
            //"LINDEX"
            _ => lindex,
        };
        ComandoListHandler {
            comando,
            a_ejecutar: Box::new(a_ejecutar),
        }
    }
}

impl ComandoHandler for ComandoListHandler {
    fn ejecutar(mut self: Box<Self>, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&mut self.comando, bdd)
    }
}

pub fn lindex(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    let lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        _ => return ResultadoRedis::Error("WRONGTYPE".to_string()),
    };
    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };
    let indice: i32 = match parametro.parse() {
        Ok(v) => v,
        Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
    };
    let tamanio = lista.len() as i32;

    if 0 < indice && indice < tamanio {
        ResultadoRedis::BulkStr(lista[indice as usize].clone())
    } else if 0 > indice && tamanio + indice >= 0 {
        ResultadoRedis::BulkStr(lista[(tamanio + indice) as usize].clone())
    } else {
        ResultadoRedis::BulkStr("nil".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lindex_busca_un_indice_positivo_en_una_clave_valor_de_la_base_de_datos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "clave".to_string(),
            TipoRedis::Lista(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lindex".to_string(),
            "clave".to_string(),
            "1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("2".to_string()),
            lindex(&mut comando, ptr)
        );
    }

    #[test]
    fn lindex_busca_un_indice_negativo_en_una_clave_valor_de_la_base_de_datos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "clave".to_string(),
            TipoRedis::Lista(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lindex".to_string(),
            "clave".to_string(),
            "-2".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("2".to_string()),
            lindex(&mut comando, ptr)
        );
    }

    #[test]
    fn lindex_busca_un_indice_fuera_de_rango_en_una_clave_valor_de_la_base_de_datos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "clave".to_string(),
            TipoRedis::Lista(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lindex".to_string(),
            "clave".to_string(),
            "65".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("nil".to_string()),
            lindex(&mut comando, ptr)
        );
    }
}
