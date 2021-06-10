use crate::comando::{Comando, ComandoHandler, ResultadoRedis, TipoRedis};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub struct ComandoSetHandler {
    comando: Vec<String>,
    a_ejecutar: Comando,
}

impl ComandoSetHandler {
    pub fn new(comando: Vec<String>) -> Self {
        let a_ejecutar = match comando[0].as_str() {
            "SADD" => sadd,
            "SCARD" => scard,
            "SISMEMBER" => sismember,
            "SMEMBERS" => smembers,
            _ => srem,
        };
        ComandoSetHandler {
            comando,
            a_ejecutar: Box::new(a_ejecutar),
        }
    }
}

impl ComandoHandler for ComandoSetHandler {
    fn ejecutar(
        self: Box<Self>,
        hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>,
    ) -> ResultadoRedis {
        (self.a_ejecutar)(&self.comando, hash_map)
    }
}
#[allow(dead_code)]
pub fn es_comando_set(comando: &str) -> bool {
    let comandos = vec!["SADD", "SCARD"];
    comandos.iter().any(|&c| c == comando)
}

fn sadd(comando: &[String], hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    let (a_agregar, cantidad_ingresada) =
        match hash_map.lock().unwrap().get(&comando[1]) {
            Some(TipoRedis::Set(set)) => aggregar_al_set(&comando[2..], &mut set.clone()),
            None => aggregar_al_set(&comando[2..], &mut HashSet::new()),
            _ => return ResultadoRedis::Error(
                "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                    .to_string(),
            ),
        };
    hash_map
        .lock()
        .unwrap()
        .insert(comando[1].clone(), TipoRedis::Set(a_agregar));
    ResultadoRedis::Int(cantidad_ingresada)
}

fn aggregar_al_set(valores: &[String], set: &mut HashSet<String>) -> (HashSet<String>, usize) {
    let mut cantidad_ingresada = 0;
    for valor in valores.iter() {
        if !set.contains(valor) {
            set.insert(valor.clone());
            cantidad_ingresada += 1;
        }
    }
    (set.clone(), cantidad_ingresada)
}

fn scard(comando: &[String], hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Set(set)) => ResultadoRedis::Int(set.len()),
        None => ResultadoRedis::Int(0),
        _ => ResultadoRedis::Error(
            "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                .to_string(),
        ),
    }
}

fn sismember(
    comando: &[String],
    hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>,
) -> ResultadoRedis {
    match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Set(set)) => {
            ResultadoRedis::Int(if set.contains(&comando[2]) { 1 } else { 0 })
        }
        None => ResultadoRedis::Int(0),
        _ => ResultadoRedis::Error(
            "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                .to_string(),
        ),
    }
}

fn smembers(
    comando: &[String],
    hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>,
) -> ResultadoRedis {
    match hash_map.lock().unwrap().get(&comando[1]) {
        Some(TipoRedis::Set(set)) => {
            let mut vector = vec![];
            for valor in set.iter() {
                vector.push(ResultadoRedis::BulkStr(valor.clone()));
            }
            ResultadoRedis::Vector(vector)
        }
        None => ResultadoRedis::Vector(vec![]),
        _ => ResultadoRedis::Error(
            "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                .to_string(),
        ),
    }
}

fn srem(comando: &[String], hash_map: Arc<Mutex<HashMap<String, TipoRedis>>>) -> ResultadoRedis {
    let (a_agregar, cantidad_eliminada) =
        match hash_map.lock().unwrap().get(&comando[1]) {
            Some(TipoRedis::Set(set)) => eliminar_del_set(&comando[2..], &mut set.clone()),
            None => return ResultadoRedis::Int(0),
            _ => return ResultadoRedis::Error(
                "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                    .to_string(),
            ),
        };

    hash_map
        .lock()
        .unwrap()
        .insert(comando[1].clone(), TipoRedis::Set(a_agregar));
    ResultadoRedis::Int(cantidad_eliminada)
}

fn eliminar_del_set(valores: &[String], set: &mut HashSet<String>) -> (HashSet<String>, usize) {
    let mut cantidad_eliminada = 0;
    for valor in valores.iter() {
        if set.contains(valor) {
            set.remove(valor);
            cantidad_eliminada += 1;
        }
    }
    (set.clone(), cantidad_eliminada)
}

#[cfg(test)]
mod tests {
    use super::*;

    //sadd
    #[test]
    fn sadd_cuando_se_envia_una_clave_que_no_esta_esta_se_crea_y_se_almacena_correctamente() {
        let hash: HashMap<String, TipoRedis> = HashMap::new();
        let vector = vec![
            "SADD".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = sadd(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(1), resultado,);

        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        assert_eq!(
            h.lock().unwrap().get(&"miClave".to_string()).unwrap(),
            &TipoRedis::Set(set),
        );
    }

    #[test]
    fn sadd_cuando_se_envia_un_valor_que_esta_repetido_esta_no_se_almacena() {
        let hash: HashMap<String, TipoRedis> = HashMap::new();
        let vector = vec![
            "SADD".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        sadd(&vector, Arc::clone(&h));
        let resultado = sadd(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(0), resultado,);
    }

    #[test]
    fn sadd_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();
        hash.insert(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec![
            "SADD".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = sadd(&vector, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                    .to_string()
            ),
            resultado,
        );
    }

    //scard
    #[test]
    fn scard_cuando_se_envia_una_clave_que_no_esta_se_devuelve_0_cardinalidad() {
        let hash: HashMap<String, TipoRedis> = HashMap::new();
        let vector = vec!["SCARD".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(hash));

        let resultado = scard(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(0), resultado,);
    }

    #[test]
    fn scard_cuando_se_envia_una_clave_que_posee_dos_elementos_se_devuelve_2_de_cardinalidad() {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();

        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        set.insert("otroValor".to_string());

        hash.insert("miClave".to_string(), TipoRedis::Set(set));
        let vector = vec!["SCARD".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(hash));

        let resultado = scard(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(2), resultado,);
    }

    #[test]
    fn scard_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();
        hash.insert(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec!["SCARD".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(hash));

        let resultado = scard(&vector, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                    .to_string()
            ),
            resultado,
        );
    }

    //sismember
    #[test]
    fn sismember_cuando_se_envia_una_clave_que_no_esta_devuelve_0_ya_que_no_pertenece() {
        let hash: HashMap<String, TipoRedis> = HashMap::new();
        let vector = vec![
            "SISMEMBER".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = sismember(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(0), resultado,);
    }

    #[test]
    fn sismember_cuando_se_envia_una_clave_que_posee_dos_elementos_y_se_pregunta_si_uno_de_ellos_pertenece_se_devuelve_1_de_true(
    ) {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();

        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        set.insert("otroValor".to_string());

        hash.insert("miClave".to_string(), TipoRedis::Set(set));
        let vector = vec![
            "SISMEMBER".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = sismember(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(1), resultado,);
    }

    #[test]
    fn sismember_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();
        hash.insert(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec![
            "SISMEMBER".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = sismember(&vector, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                    .to_string()
            ),
            resultado,
        );
    }
    //smembers
    #[test]
    fn smembers_cuando_se_envia_una_clave_que_no_esta_devuelve_un_vector_vacio() {
        let hash: HashMap<String, TipoRedis> = HashMap::new();
        let vector = vec!["SMEMBERS".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(hash));

        let resultado = smembers(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Vector(vec![]), resultado,);
    }

    #[test]
    fn smembers_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();
        hash.insert(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec!["SMEMBERS".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(hash));

        let resultado = smembers(&vector, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                    .to_string()
            ),
            resultado,
        );
    }

    //srem
    #[test]
    fn srem_cuando_se_envia_una_clave_que_no_esta_no_se_elimina_ningun_valor() {
        let hash: HashMap<String, TipoRedis> = HashMap::new();
        let vector = vec![
            "SREM".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = srem(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(0), resultado,);
    }

    #[test]
    fn srem_cuando_se_envia_una_clave_que_posee_dos_elementos_y_se_elimina_uno_este_no_se_encuentra(
    ) {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();

        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        set.insert("otroValor".to_string());

        hash.insert("miClave".to_string(), TipoRedis::Set(set));
        let vector = vec![
            "SREM".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = srem(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(1), resultado,);

        let mut set = HashSet::new();
        set.insert("otroValor".to_string());
        assert_eq!(
            h.lock().unwrap().get(&"miClave".to_string()).unwrap(),
            &TipoRedis::Set(set),
        );
    }

    #[test]
    fn srem_cuando_se_envia_una_clave_que_posee_dos_elementos_y_se_eliminan_solo_uno_dos_veces_no_se_encuentra_el_restante(
    ) {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();

        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        set.insert("otroValor".to_string());

        hash.insert("miClave".to_string(), TipoRedis::Set(set));
        let vector = vec![
            "SREM".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = srem(&vector, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(1), resultado,);

        let mut set = HashSet::new();
        set.insert("otroValor".to_string());
        assert_eq!(
            h.lock().unwrap().get(&"miClave".to_string()).unwrap(),
            &TipoRedis::Set(set),
        );
    }

    #[test]
    fn srem_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut hash: HashMap<String, TipoRedis> = HashMap::new();
        hash.insert(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec![
            "SREM".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(hash));

        let resultado = srem(&vector, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WrongTypeError error al obtener el set, valor guardado en la clave no es un Set"
                    .to_string()
            ),
            resultado,
        );
    }
}
