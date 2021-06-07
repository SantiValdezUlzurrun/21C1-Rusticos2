use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::comando::{ComandoHandler, Comando, ResultadoRedis, TipoRedis};

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
            comando ,
            a_ejecutar: Box::new(a_ejecutar),

        }
    }
}

impl ComandoHandler for ComandoStringHandler {

    fn ejecutar(self: Box<Self>, hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
        (self.a_ejecutar)(&self.comando, hash_map)
    }
}

pub fn es_comando_string(comando: &String) -> bool {
    
    let comandos = vec!["GET", "SET", "APPEND"];
    comandos.iter().any(|&c| c == comando)
}


fn get(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Str(valor)) => ResultadoRedis::BulkStr(valor.to_string()),
        _ => ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
    }
}

fn set(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
        hash_map
            .lock()
            .unwrap()
            .insert(comando[1].clone(), TipoRedis::Str(comando[2].clone()));
        ResultadoRedis::StrSimple("OK".to_string())
}

fn append(comando: &Vec<String>,  hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
   
    if hash_map.lock().unwrap().contains_key(&comando[1]){
        let valor = match get(comando,hash_map.clone()){
            ResultadoRedis::BulkStr(valor) => valor + &comando[2],
            _ => return ResultadoRedis::Error("GetError error al obtener la clave".to_string()) 
        };
    
        hash_map.lock().unwrap().insert(comando[1].clone(), TipoRedis::Str(valor.clone()));
        return ResultadoRedis::Int(valor.len())
    };
    hash_map.lock().unwrap().insert(comando[1].clone(), TipoRedis::Str(comando[2].clone()));
    ResultadoRedis::Int(comando[2].len())
}


#[cfg(test)]
mod tests {
    use std::collections::LinkedList;
    use super::*;

    #[test]
    fn get_devuelve_el_valor_almacenado_en_el_hash(){
        let mut hash : HashMap<String, TipoRedis>= HashMap::new();
        hash.insert("miClave".to_string(),TipoRedis::Str("miValor".to_string()));
        let vector = vec!["get".to_string(),"miClave".to_string()];

        assert_eq!( ResultadoRedis::BulkStr("miValor".to_string()),get(&vector,Arc::new(Mutex::new(hash))));
    }

    #[test]
    fn get_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista(){
        let mut hash : HashMap<String, TipoRedis>= HashMap::new();
        hash.insert("miClave".to_string(),TipoRedis::Lista(LinkedList::new()));
        let vector = vec!["get".to_string(),"miClave".to_string()];

        assert_eq!( ResultadoRedis::Error("GetError error al obtener la clave".to_string()),get(&vector,Arc::new(Mutex::new(hash))));
    }

    #[test]
    fn set_almacena_un_valor_en_el_hash(){
        let hash : HashMap<String, TipoRedis>= HashMap::new();
        let vector = vec!["get".to_string(),"miClave".to_string(),"miValor".to_string()];

        assert_eq!(ResultadoRedis::StrSimple("OK".to_string()),set(&vector,Arc::new(Mutex::new(hash))));
    }

    #[test]
    fn append_agrega_el_string_enviado_al_final_del_string_guardado_con_la_misma_clave(){
        let mut hash : HashMap<String, TipoRedis>= HashMap::new();
        hash.insert("miClave".to_string(),TipoRedis::Str("miValor".to_string()));
        let ptr_hash = Arc::new(Mutex::new(hash));
        let ptr_hash1= Arc::clone(&ptr_hash);

        let vector = vec!["APPEND".to_string(),"miClave".to_string(),"conAlgoAppendeado".to_string()];

        assert_eq!( ResultadoRedis::Int(24), append(&vector,ptr_hash1));
        assert_eq!( ResultadoRedis::BulkStr("miValorconAlgoAppendeado".to_string()),get(&vector,ptr_hash));
    }

    #[test]
    fn append_agrega_un_string_al_hash_porque_no_hay_un_elemento_guarado_con_esa_clave(){
        let hash : HashMap<String, TipoRedis>= HashMap::new();
        let ptr_hash = Arc::new(Mutex::new(hash));
        let ptr_hash1= Arc::clone(&ptr_hash);

        let vector = vec!["APPEND".to_string(),"miClave".to_string(),"conAlgoAppendeado".to_string()];

        assert_eq!( ResultadoRedis::Int(17), append(&vector,ptr_hash1));
        assert_eq!( ResultadoRedis::BulkStr("conAlgoAppendeado".to_string()),get(&vector,ptr_hash));
    }

    #[test]
    fn append_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista(){
        let mut hash : HashMap<String, TipoRedis>= HashMap::new();
        hash.insert("miClave".to_string(),TipoRedis::Lista(LinkedList::new()));
        let vector = vec!["get".to_string(),"miClave".to_string()];

        assert_eq!( ResultadoRedis::Error("GetError error al obtener la clave".to_string()),append(&vector,Arc::new(Mutex::new(hash))));
    }
}
