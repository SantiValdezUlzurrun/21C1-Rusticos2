use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{Comando, ComandoHandler};
use std::sync::{Arc, Mutex};

/*
Comando Lista faltantes:
+ decrby
+ getset
+ incrby
+ mget
+ mset
*/
pub struct ComandoStringHandler {
    comando: Vec<String>,
    a_ejecutar: Comando,
}

impl ComandoStringHandler {
    pub fn new(comando: Vec<String>) -> Self {
        let a_ejecutar = match comando[0].as_str() {
            "GET" => get,
            _ => set,
        };
        ComandoStringHandler {
            comando,
            a_ejecutar: Box::new(a_ejecutar),
        }
    }
}

impl ComandoHandler for ComandoStringHandler {
    fn ejecutar(self: Box<Self>, hash_map: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&self.comando, hash_map)
    }
}

pub fn es_comando_string(comando: &str) -> bool {
    let comandos = vec!["GET", "SET", "APPEND"];
    comandos.iter().any(|&c| c == comando)
}

fn get(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    match bdd.lock().unwrap().obtener_valor(&comando[1]) {
        Some(TipoRedis::Str(valor)) => ResultadoRedis::BulkStr(valor.to_string()),
        _ => ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
    }
}

fn set(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    bdd.lock()
        .unwrap()
        .guardar_valor(comando[1].clone(), TipoRedis::Str(comando[2].clone()));
    ResultadoRedis::StrSimple("OK".to_string())
}

#[allow(dead_code)]
fn append(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    if bdd.lock().unwrap().existe_clave(&comando[1]) {
        let valor = match get(comando, bdd.clone()) {
            ResultadoRedis::BulkStr(valor) => valor + &comando[2],
            _ => return ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
        };

        bdd.lock()
            .unwrap()
            .guardar_valor(comando[1].clone(), TipoRedis::Str(valor.clone()));
        return ResultadoRedis::Int(valor.len());
    };
    bdd.lock()
        .unwrap()
        .guardar_valor(comando[1].clone(), TipoRedis::Str(comando[2].clone()));
    ResultadoRedis::Int(comando[2].len())
}
#[allow(dead_code)]
fn getdel(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let bdd_clon = Arc::clone(&bdd);
    let resultado = get(comando, bdd_clon);
    bdd.lock().unwrap().eliminar_clave(&comando[1]);
    resultado
}
#[allow(dead_code)]
fn strlen(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    match bdd.lock().unwrap().obtener_valor(&comando[1]) {
        Some(TipoRedis::Str(valor)) => ResultadoRedis::Int(valor.len()),
        _ => ResultadoRedis::Error("StrLen error al obtener la clave".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::LinkedList;

    #[test]
    fn get_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));
        let vector = vec!["get".to_string(), "miClave".to_string()];

        assert_eq!(
            ResultadoRedis::BulkStr("miValor".to_string()),
            get(&vector, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn get_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(LinkedList::new()));
        let vector = vec!["get".to_string(), "miClave".to_string()];

        assert_eq!(
            ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
            get(&vector, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn set_almacena_un_valor_en_el_hash() {
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        let vector = vec![
            "get".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        assert_eq!(
            ResultadoRedis::StrSimple("OK".to_string()),
            set(&vector, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn append_agrega_el_string_enviado_al_final_del_string_guardado_con_la_misma_clave() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let vector = vec![
            "APPEND".to_string(),
            "miClave".to_string(),
            "conAlgoAppendeado".to_string(),
        ];

        assert_eq!(ResultadoRedis::Int(24), append(&vector, ptr_hash1));
        assert_eq!(
            ResultadoRedis::BulkStr("miValorconAlgoAppendeado".to_string()),
            get(&vector, ptr_hash)
        );
    }

    #[test]
    fn append_agrega_un_string_al_hash_porque_no_hay_un_elemento_guarado_con_esa_clave() {
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let vector = vec![
            "APPEND".to_string(),
            "miClave".to_string(),
            "conAlgoAppendeado".to_string(),
        ];

        assert_eq!(ResultadoRedis::Int(17), append(&vector, ptr_hash1));
        assert_eq!(
            ResultadoRedis::BulkStr("conAlgoAppendeado".to_string()),
            get(&vector, ptr_hash)
        );
    }

    #[test]
    fn append_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(LinkedList::new()));
        let vector = vec!["get".to_string(), "miClave".to_string()];

        assert_eq!(
            ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
            append(&vector, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn getdel_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));

        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash_clone = Arc::clone(&ptr_hash);

        let vector = vec!["get".to_string(), "miClave".to_string()];

        assert_eq!(
            ResultadoRedis::BulkStr("miValor".to_string()),
            getdel(&vector, ptr_hash_clone)
        );
        assert_eq!(
            ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
            getdel(&vector, ptr_hash)
        );
    }

    #[test]
    fn getdel_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(LinkedList::new()));
        let vector = vec!["get".to_string(), "miClave".to_string()];

        assert_eq!(
            ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
            get(&vector, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn strlen_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));
        let vector = vec!["get".to_string(), "miClave".to_string()];

        assert_eq!(
            ResultadoRedis::Int(7),
            strlen(&vector, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn strlen_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(LinkedList::new()));
        let vector = vec!["get".to_string(), "miClave".to_string()];

        assert_eq!(
            ResultadoRedis::Error("StrLen error al obtener la clave".to_string()),
            strlen(&vector, Arc::new(Mutex::new(bdd)))
        );
    }
}