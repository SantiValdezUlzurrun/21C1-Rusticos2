use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{Comando, ComandoHandler};
use crate::comando_info::ComandoInfo;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct ComandoSetHandler {
    comando: ComandoInfo,
    a_ejecutar: Comando,
}

impl ComandoSetHandler {
    pub fn new(comando: ComandoInfo) -> Self {
        let a_ejecutar = match comando.get_nombre().as_str() {
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
    fn ejecutar(mut self: Box<Self>, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&mut self.comando, bdd)
    }
}

pub fn es_comando_set(comando: &str) -> bool {
    let comandos = vec!["SADD", "SCARD", "SISMEMBER", "SMEMBERS", "SREM"];
    comandos.iter().any(|&c| c == comando)
}

fn sadd(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(clave) => clave,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'sadd' command".to_string(),
            )
        }
    };
    match bdd.lock() {
        Ok(mut bdd) => {
            let (a_agregar, cantidad_ingresada) = match bdd.obtener_valor(&clave) {
                Some(TipoRedis::Set(set)) => aggregar_al_set(comando, &mut set.clone()),
                None => aggregar_al_set(comando, &mut HashSet::new()),
                _ => {
                    return ResultadoRedis::Error(
                        "WRONGTYPE Operation against a key holding the wrong kind of value"
                            .to_string(),
                    )
                }
            };
            bdd.guardar_valor(clave, TipoRedis::Set(a_agregar));
            ResultadoRedis::Int(cantidad_ingresada as isize)
        }
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}

fn aggregar_al_set(
    comando: &mut ComandoInfo,
    set: &mut HashSet<String>,
) -> (HashSet<String>, usize) {
    let mut cantidad_ingresada = 0;

    while let Some(parametro) = comando.get_parametro() {
        if !set.contains(&parametro) {
            set.insert(parametro);
            cantidad_ingresada += 1;
        }
    }
    (set.clone(), cantidad_ingresada)
}

fn scard(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(clave) => clave,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'scard' command".to_string(),
            )
        }
    };

    match bdd.lock() {
        Ok(bdd) => match bdd.obtener_valor(&clave) {
            Some(TipoRedis::Set(set)) => ResultadoRedis::Int(set.len() as isize),
            None => ResultadoRedis::Int(0),
            _ => ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
            ),
        },
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}

fn sismember(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(clave) => clave,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'scard' command".to_string(),
            )
        }
    };

    match bdd.lock() {
        Ok(bdd) => match bdd.obtener_valor(&clave) {
            Some(TipoRedis::Set(set)) => {
                let parametro = match comando.get_parametro() {
                    Some(parametro) => parametro,
                    None => {
                        return ResultadoRedis::Error(
                            "WRONGTYPE Operation against a key holding the wrong kind of value"
                                .to_string(),
                        )
                    }
                };
                ResultadoRedis::Int(if set.contains(&parametro) { 1 } else { 0 })
            }
            None => ResultadoRedis::Int(0),
            _ => ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
            ),
        },
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}

fn smembers(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(clave) => clave,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'smembers' command".to_string(),
            )
        }
    };

    match bdd.lock() {
        Ok(bdd) => match bdd.obtener_valor(&clave) {
            Some(TipoRedis::Set(set)) => {
                let mut vector = vec![];
                for valor in set.iter() {
                    vector.push(ResultadoRedis::BulkStr(valor.clone()));
                }
                ResultadoRedis::Vector(vector)
            }
            None => ResultadoRedis::Vector(vec![]),
            _ => ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
            ),
        },
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}

fn srem(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(clave) => clave,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'srem' command".to_string(),
            )
        }
    };

    match bdd.lock() {
        Ok(mut bdd) => {
            let (a_agregar, cantidad_eliminada) = match bdd.obtener_valor(&clave) {
                Some(TipoRedis::Set(set)) => eliminar_del_set(comando, &mut set.clone()),
                None => return ResultadoRedis::Int(0),
                _ => {
                    return ResultadoRedis::Error(
                        "WRONGTYPE Operation against a key holding the wrong kind of value"
                            .to_string(),
                    )
                }
            };
            bdd.guardar_valor(clave, TipoRedis::Set(a_agregar));
            ResultadoRedis::Int(cantidad_eliminada as isize)
        }
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}

fn eliminar_del_set(
    comando: &mut ComandoInfo,
    set: &mut HashSet<String>,
) -> (HashSet<String>, usize) {
    let mut cantidad_eliminada = 0;

    while let Some(parametro) = comando.get_parametro() {
        if set.contains(&parametro) {
            set.remove(&parametro);
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
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let vector = vec![
            "SADD".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = sadd(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(1), resultado,);

        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        assert_eq!(
            h.lock()
                .unwrap()
                .obtener_valor(&"miClave".to_string())
                .unwrap(),
            &TipoRedis::Set(set),
        );
    }

    #[test]
    fn sadd_cuando_se_envia_un_valor_que_esta_repetido_esta_no_se_almacena() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let vector = vec![
            "SADD".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        sadd(&mut comando, Arc::clone(&h));
        let resultado = sadd(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(0), resultado,);
    }

    #[test]
    fn sadd_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec![
            "SADD".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = sadd(&mut comando, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            resultado,
        );
    }

    //scard
    #[test]
    fn scard_cuando_se_envia_una_clave_que_no_esta_se_devuelve_0_cardinalidad() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let vector = vec!["SCARD".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = scard(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(0), resultado,);
    }

    #[test]
    fn scard_cuando_se_envia_una_clave_que_posee_dos_elementos_se_devuelve_2_de_cardinalidad() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        set.insert("otroValor".to_string());

        bdd.guardar_valor("miClave".to_string(), TipoRedis::Set(set));
        let vector = vec!["SCARD".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = scard(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(2), resultado);
    }

    #[test]
    fn scard_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec!["SCARD".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = scard(&mut comando, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            resultado,
        );
    }

    //sismember
    #[test]
    fn sismember_cuando_se_envia_una_clave_que_no_esta_devuelve_0_ya_que_no_pertenece() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let vector = vec![
            "SISMEMBER".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = sismember(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(0), resultado,);
    }

    #[test]
    fn sismember_cuando_se_envia_una_clave_que_posee_dos_elementos_y_se_pregunta_si_uno_de_ellos_pertenece_se_devuelve_1_de_true(
    ) {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        set.insert("otroValor".to_string());

        bdd.guardar_valor("miClave".to_string(), TipoRedis::Set(set));
        let vector = vec![
            "SISMEMBER".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = sismember(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(1), resultado);
    }

    #[test]
    fn sismember_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec![
            "SISMEMBER".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = sismember(&mut comando, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            resultado,
        );
    }
    //smembers
    #[test]
    fn smembers_cuando_se_envia_una_clave_que_no_esta_devuelve_un_vector_vacio() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let vector = vec!["SMEMBERS".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = smembers(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Vector(vec![]), resultado);
    }

    #[test]
    fn smembers_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec!["SMEMBERS".to_string(), "miClave".to_string()];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = smembers(&mut comando, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            resultado,
        );
    }

    //srem
    #[test]
    fn srem_cuando_se_envia_una_clave_que_no_esta_no_se_elimina_ningun_valor() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let vector = vec![
            "SREM".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = srem(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(0), resultado,);
    }

    #[test]
    fn srem_cuando_se_envia_una_clave_que_posee_dos_elementos_y_se_elimina_uno_este_no_se_encuentra(
    ) {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();

        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        set.insert("otroValor".to_string());

        bdd.guardar_valor("miClave".to_string(), TipoRedis::Set(set));
        let vector = vec![
            "SREM".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = srem(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(1), resultado,);

        let mut set = HashSet::new();
        set.insert("otroValor".to_string());
        assert_eq!(
            h.lock()
                .unwrap()
                .obtener_valor(&"miClave".to_string())
                .unwrap(),
            &TipoRedis::Set(set),
        );
    }

    #[test]
    fn srem_cuando_se_envia_una_clave_que_posee_dos_elementos_y_se_eliminan_solo_uno_dos_veces_no_se_encuentra_el_restante(
    ) {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();

        let mut set = HashSet::new();
        set.insert("miValor".to_string());
        set.insert("otroValor".to_string());

        bdd.guardar_valor("miClave".to_string(), TipoRedis::Set(set));
        let vector = vec![
            "SREM".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = srem(&mut comando, Arc::clone(&h));

        assert_eq!(ResultadoRedis::Int(1), resultado,);

        let mut set = HashSet::new();
        set.insert("otroValor".to_string());
        assert_eq!(
            h.lock()
                .unwrap()
                .obtener_valor(&"miClave".to_string())
                .unwrap(),
            &TipoRedis::Set(set),
        );
    }

    #[test]
    fn srem_cuando_se_envia_una_clave_invalida_se_envia_el_error_adecuado() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor(
            "miClave".to_string(),
            TipoRedis::Str("unString".to_string()),
        );
        let vector = vec![
            "SREM".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ];

        let h = Arc::new(Mutex::new(bdd));
        let mut comando = ComandoInfo::new(vector);
        let resultado = srem(&mut comando, Arc::clone(&h));

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            resultado,
        );
    }
}
