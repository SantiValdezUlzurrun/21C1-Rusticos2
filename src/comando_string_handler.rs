use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{Comando, ComandoHandler};
use crate::comando_info::ComandoInfo;
use std::sync::{Arc, Mutex};

pub struct ComandoStringHandler {
    comando: ComandoInfo,
    a_ejecutar: Comando,
}

impl ComandoStringHandler {
    pub fn new(comando: ComandoInfo) -> Self {
        let a_ejecutar = match comando.get_nombre().as_str() {
            "GET" => get,
            "APPEND" => append,
            "GETDEL" => getdel,
            "STRLEN" => strlen,
            "DECRBY" => decrby,
            "INCRBY" => incrby,
            "MGET" => mget,
            "MSET" => mset,
            "GETSET" => getset,
            _ => set,
        };
        ComandoStringHandler {
            comando,
            a_ejecutar: Box::new(a_ejecutar),
        }
    }
}

impl ComandoHandler for ComandoStringHandler {
    fn ejecutar(mut self: Box<Self>, hash_map: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&mut self.comando, hash_map)
    }
}
/// Se encarga de detectar si el comando corresponde a los implementados del tipo string
pub fn es_comando_string(comando: &str) -> bool {
    let comandos = vec![
        "GET", "SET", "APPEND", "STRLEN", "INCRBY", "DECRBY", "MGET", "MSET", "GETSET", "GETDEL",
    ];
    comandos.iter().any(|&c| c == comando)
}
/// Devuelve el valor de una clave, si la clave no existe, se retorna el valor especial nil. Se retorna un error si el valor almacenado en esa clave no es un string, porque GET maneja solamente strings
fn get(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'get' command".to_string(),
            )
        }
    };

    match bdd.lock() {
        Ok(bdd) => match bdd.obtener_valor(&clave) {
            Some(TipoRedis::Str(valor)) => ResultadoRedis::BulkStr(valor.to_string()),
            None => ResultadoRedis::Nil,
            _ => ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
            ),
        },
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}

fn obtener_tiempo_expiracion(parametros: Vec<String>, support: &str) -> Option<u64> {
    match parametros.rsplit(|p| p == &support.to_string()).next() {
        Some(c) => match c[0].clone().parse::<u64>() {
            Ok(num) => Some(num),
            Err(_) => None,
        },
        None => None,
    }
}
/// Setea que la clave especificada almacene el valor especificado de tipo string. Si la clave contiene un valor previo, la clave es sobreescrita, independientemente del tipo de dato contenido (descartando también el valor previo de TTL)
fn set(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'set' command".to_string(),
            )
        }
    };

    let parametros = match comando.get_parametros() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'set' command".to_string(),
            )
        }
    };
    if parametros.len() == 1 {
        return ResultadoRedis::Error(
            "ERR wrong number of arguments for 'set' command".to_string(),
        );
    }
    match bdd.lock() {
        Ok(mut bdd) => {
            if parametros.contains(&"EX".to_string()) {
                let expiracion = match obtener_tiempo_expiracion(parametros.clone(), "EX") {
                    Some(e) => e,
                    None => {
                        return ResultadoRedis::Error(
                            "ERR value is not an integer or out of range".to_string(),
                        )
                    }
                };
                bdd.guardar_valor_con_expiracion(
                    clave,
                    expiracion,
                    TipoRedis::Str(parametros[1].clone()),
                )
            } else if parametros.contains(&"PX".to_string()) {
                let expiracion = match obtener_tiempo_expiracion(parametros.clone(), "PX") {
                    Some(e) => e,
                    None => {
                        return ResultadoRedis::Error(
                            "ERR value is not an integer or out of range".to_string(),
                        )
                    }
                };
                bdd.guardar_valor_con_expiracion(
                    clave,
                    expiracion / 1000,
                    TipoRedis::Str(parametros[1].clone()),
                )
            } else {
                bdd.guardar_valor(clave, TipoRedis::Str(parametros[1].clone()))
            }
        }
        Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
    ResultadoRedis::StrSimple("OK".to_string())
}
/// Atómicamente setea el valor a la clave deseada, y retorna el valor anterior almacenado en la clave
fn getset(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'getset' command".to_string(),
            )
        }
    };
    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'getset' command".to_string(),
            )
        }
    };

    match bdd.lock() {
        Ok(mut bdd) => match bdd.intercambiar_valor(clave, TipoRedis::Str(parametro)) {
            Some(TipoRedis::Str(valor_enterior)) => ResultadoRedis::StrSimple(valor_enterior),
            None => ResultadoRedis::Nil,
            _ => ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
            ),
        },
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}
/// Si la clave ya existe y es un string, este comando agrega el valor al final del string. Si no existe, es creada con el string vacío y luego le agrega el valor deseado. En este caso es similar al comando SET
fn append(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'append' command".to_string(),
            )
        }
    };
    let mut valor = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'append' command".to_string(),
            )
        }
    };
    match bdd.lock() {
        Ok(mut bdd) => {
            if bdd.existe_clave(&clave) {
                valor = match bdd.obtener_valor(&clave) {
                    Some(TipoRedis::Str(v)) => v.to_string() + &valor,
                    _ => {
                        return ResultadoRedis::Error(
                            "WRONGTYPE Operation against a key holding the wrong kind of value"
                                .to_string(),
                        )
                    }
                };
            };
            bdd.guardar_valor(clave, TipoRedis::Str(valor.to_string()));
            ResultadoRedis::Int(valor.len() as isize)
        }
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}
/// Obtiene el valor y elimina la clave. Es similar a GET, pero adicionalmente elimina la clave
fn getdel(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for 'getdel' command".to_string(),
            )
        }
    };
    let bdd_clon = Arc::clone(&bdd);
    match get(comando, bdd_clon) {
        ResultadoRedis::Error(s) => ResultadoRedis::Error(s),
        ResultadoRedis::BulkStr(valor) => match bdd.lock() {
            Ok(mut bdd) => {
                bdd.eliminar_clave(&clave);
                ResultadoRedis::BulkStr(valor)
            }
            Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
        },
        _ => ResultadoRedis::Nil,
    }
}
/// Retorna el largo del valor de tipo string almacenado en una clave. Retorna error si la clave no almacena un string
fn strlen(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Int(0),
    };
    match bdd.lock() {
        Ok(bdd) => match bdd.obtener_valor(&clave) {
            Some(TipoRedis::Str(valor)) => ResultadoRedis::Int(valor.len() as isize),
            _ => ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
            ),
        },
        Err(_) => ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
}
/// Dado un elemento almacenado en la base de datos que puede ser casteable a un int y una funcion que opere sobre el, se aplica la funcion devolviendo otro int
fn operar_sobre_int(
    comando: &mut ComandoInfo,
    bdd: Arc<Mutex<BaseDeDatos>>,
    f: fn(i32, i32) -> i32,
) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ERR wrong number of arguments".to_string()),
    };

    let valor = match bdd.lock() {
        Ok(bdd) => match bdd.obtener_valor(&clave) {
            Some(TipoRedis::Str(valor)) => valor.clone(),
            None => "0".to_string(),
            _ => {
                return ResultadoRedis::Error(
                    "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
                )
            }
        },
        Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
    };

    let mut num = match valor.parse::<i32>() {
        Ok(n) => n,
        Err(_) => {
            return ResultadoRedis::Error("ERR value is not an integer or out of range".to_string())
        }
    };

    let param = match comando.get_parametro() {
        Some(p) => p,
        None => 0.to_string(),
    };

    let param = match param.parse::<i32>() {
        Ok(p) => p,
        Err(_) => {
            return ResultadoRedis::Error("ERR value is not an integer or out of range".to_string())
        }
    };

    num = f(num, param);
    match bdd.lock() {
        Ok(mut bdd) => bdd.guardar_valor(clave, TipoRedis::Str(num.to_string())),
        Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
    }
    ResultadoRedis::BulkStr(num.to_string())
}
/// Decrementa el número almacenado en una clave por el valor deseado. Si la clave no existe, se setea en 0 antes de realizar la operación
fn decrby(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    operar_sobre_int(comando, bdd, |a, b| a - b)
}
/// Incrementa el número almacenado en la clave en un incremento. Si la clave no existe, es seteado a 0 antes de realizar la operación. Devuelve error si la clave contiene un valor de tipo erróneo o un string que no puede ser representado como entero
fn incrby(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    operar_sobre_int(comando, bdd, |a, b| a + b)
}
/// Retorna el valor de todas las claves especificadas. Para las claves que no contienen valor o el valor no es un string, se retorna el tipo especial nil
fn mget(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let mut valores = vec![];
    let mut quedan_valores = true;

    while quedan_valores {
        let param = comando.get_parametro();
        match param {
            Some(p) => match bdd.lock() {
                Ok(bdd) => match bdd.obtener_valor(&p) {
                    Some(TipoRedis::Str(valor)) => {
                        valores.push(ResultadoRedis::BulkStr(valor.to_string()))
                    }
                    _ => valores.push(ResultadoRedis::Nil),
                },
                Err(_) => {
                    return ResultadoRedis::Error("ERR when accessing the database".to_string())
                }
            },
            None => {
                quedan_valores = false;
            }
        }
    }

    if valores.is_empty() {
        return ResultadoRedis::Error("ERR wrong number of arguments for mget command".to_string());
    }
    ResultadoRedis::Vector(valores)
}
/// Setea las claves data a sus respectivos valores, reemplazando los valores existentes con los nuevos valores como SET. MSET es atómica, de modo que todas las claves son actualizadas a la vez. No es posible para los clientes ver que algunas claves del conjunto fueron modificadas, mientras otras no
fn mset(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let parametros = match comando.get_parametros() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error(
                "ERR wrong number of arguments for mset command".to_string(),
            )
        }
    };

    if parametros.len() % 2 != 0 {
        return ResultadoRedis::Error("ERR wrong number of arguments for mset command".to_string());
    }

    match bdd.lock() {
        Ok(mut bdd) => bdd.guardar_valores(parametros),
        Err(_) => return ResultadoRedis::Error("ERR when accessing the database".to_string()),
    };
    ResultadoRedis::StrSimple("OK".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn get_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::BulkStr("miValor".to_string()),
            get(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn get_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            get(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn set_almacena_un_valor_en_el_hash() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let mut comando = ComandoInfo::new(vec![
            "get".to_string(),
            "miClave".to_string(),
            "miValor".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::StrSimple("OK".to_string()),
            set(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn append_agrega_el_string_enviado_al_final_del_string_guardado_con_la_misma_clave() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec![
            "APPEND".to_string(),
            "miClave".to_string(),
            "conAlgoAppendeado".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(24), append(&mut comando, ptr_hash1));
        assert_eq!(
            ResultadoRedis::BulkStr("miValorconAlgoAppendeado".to_string()),
            get(&mut comando, ptr_hash)
        );
    }

    #[test]
    fn append_agrega_un_string_al_hash_porque_no_hay_un_elemento_guarado_con_esa_clave() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec![
            "APPEND".to_string(),
            "miClave".to_string(),
            "conAlgoAppendeado".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(17), append(&mut comando, ptr_hash1));
        assert_eq!(
            ResultadoRedis::BulkStr("conAlgoAppendeado".to_string()),
            get(&mut comando, ptr_hash)
        );
    }

    #[test]
    fn append_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec![
            "APPEND".to_owned(),
            "miClave".to_string(),
            " palabra".to_owned(),
        ]);

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            append(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn getdel_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));

        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash_clone = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::BulkStr("miValor".to_string()),
            getdel(&mut comando, ptr_hash_clone)
        );
        assert_eq!(ResultadoRedis::Nil, getdel(&mut comando, ptr_hash));
    }

    #[test]
    fn getdel_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            get(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn strlen_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::Int(7),
            strlen(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn strlen_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            strlen(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn decrby_resta_correcatemente_un_valor_entero_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("1".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "decrby".to_string(),
            "miClave".to_string(),
            "1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("0".to_string()),
            decrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn decrby_resta_correcatemente_un_valor_entero_a_una_clave_negativa_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("-10".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "decrby".to_string(),
            "miClave".to_string(),
            "1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("-11".to_string()),
            decrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn decrby_resta_correcatemente_un_valor_negativo_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("10".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "decrby".to_string(),
            "miClave".to_string(),
            "-1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("11".to_string()),
            decrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn decrby_setea_correcatemente_un_valor_entero_a_una_clave_inexistente() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let mut comando = ComandoInfo::new(vec![
            "decrby".to_string(),
            "miClave".to_string(),
            "-1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("1".to_string()),
            decrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn decrby_devuelve_error_un_valor_entero_a_una_clave_inparseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec![
            "decrby".to_string(),
            "miClave".to_string(),
            "-1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            decrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn decrby_devuelve_error_un_valor_erroneo_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("1".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "decrby".to_string(),
            "miClave".to_string(),
            "a".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("ERR value is not an integer or out of range".to_string()),
            decrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn incrby_resta_correcatemente_un_valor_entero_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("1".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "incrby".to_string(),
            "miClave".to_string(),
            "1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("2".to_string()),
            incrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn incrby_resta_correcatemente_un_valor_entero_a_una_clave_negativa_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("-10".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "incrby".to_string(),
            "miClave".to_string(),
            "1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("-9".to_string()),
            incrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn incrby_resta_correcatemente_un_valor_negativo_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("10".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "incrby".to_string(),
            "miClave".to_string(),
            "-1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("9".to_string()),
            incrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn incrby_setea_correcatemente_un_valor_entero_a_una_clave_inexistente() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let mut comando = ComandoInfo::new(vec![
            "incrby".to_string(),
            "miClave".to_string(),
            "1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("1".to_string()),
            incrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn incrby_devuelve_error_un_valor_entero_a_una_clave_inparseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec![
            "incrby".to_string(),
            "miClave".to_string(),
            "5".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            incrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn incrby_devuelve_error_un_valor_erroneo_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("1".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "incrby".to_string(),
            "miClave".to_string(),
            "a".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("ERR value is not an integer or out of range".to_string()),
            incrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn mget_devuelve_una_lista_con_todos_los_valores_de_las_claves() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("clave1".to_string(), TipoRedis::Str("1".to_string()));
        bdd.guardar_valor("clave2".to_string(), TipoRedis::Str("2".to_string()));
        bdd.guardar_valor("clave3".to_string(), TipoRedis::Str("3".to_string()));
        bdd.guardar_valor("clave4".to_string(), TipoRedis::Str("4".to_string()));

        let mut comando = ComandoInfo::new(vec![
            "mget".to_string(),
            "clave1".to_string(),
            "clave2".to_string(),
            "clave3".to_string(),
            "clave4".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![
                ResultadoRedis::BulkStr("1".to_string()),
                ResultadoRedis::BulkStr("2".to_string()),
                ResultadoRedis::BulkStr("3".to_string()),
                ResultadoRedis::BulkStr("4".to_string())
            ]),
            mget(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn mget_devuelve_una_lista_con_todos_los_valores_de_las_claves_y_si_la_clave_no_existe_devuelve_nil(
    ) {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("clave1".to_string(), TipoRedis::Str("1".to_string()));
        bdd.guardar_valor("clave2".to_string(), TipoRedis::Str("2".to_string()));
        bdd.guardar_valor("clave3".to_string(), TipoRedis::Str("3".to_string()));
        bdd.guardar_valor("clave4".to_string(), TipoRedis::Str("4".to_string()));

        let mut comando = ComandoInfo::new(vec![
            "mget".to_string(),
            "clave1".to_string(),
            "clave2".to_string(),
            "clave3".to_string(),
            "clave1000".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![
                ResultadoRedis::BulkStr("1".to_string()),
                ResultadoRedis::BulkStr("2".to_string()),
                ResultadoRedis::BulkStr("3".to_string()),
                ResultadoRedis::Nil
            ]),
            mget(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn mget_devuelve_una_lista_con_todos_nil_si_la_clave_no_existen() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("clave1".to_string(), TipoRedis::Str("1".to_string()));
        bdd.guardar_valor("clave2".to_string(), TipoRedis::Str("2".to_string()));
        bdd.guardar_valor("clave3".to_string(), TipoRedis::Str("3".to_string()));
        bdd.guardar_valor("clave4".to_string(), TipoRedis::Str("4".to_string()));

        let mut comando = ComandoInfo::new(vec![
            "mget".to_string(),
            "clave1000".to_string(),
            "clave2000".to_string(),
            "clave3000".to_string(),
            "clave4000".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![
                ResultadoRedis::Nil,
                ResultadoRedis::Nil,
                ResultadoRedis::Nil,
                ResultadoRedis::Nil
            ]),
            mget(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn mget_devuelve_una_lista_con_todos_nil_si_la_clave_no_es_de_tipo_str() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("clave1".to_string(), TipoRedis::Str("1".to_string()));
        bdd.guardar_valor("clave2".to_string(), TipoRedis::Lista(Vec::new()));
        bdd.guardar_valor("clave3".to_string(), TipoRedis::Set(HashSet::new()));
        bdd.guardar_valor("clave4".to_string(), TipoRedis::Str("4".to_string()));

        let mut comando = ComandoInfo::new(vec![
            "mget".to_string(),
            "clave1".to_string(),
            "clave2".to_string(),
            "clave3".to_string(),
            "clave4".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![
                ResultadoRedis::BulkStr("1".to_string()),
                ResultadoRedis::Nil,
                ResultadoRedis::Nil,
                ResultadoRedis::BulkStr("4".to_string())
            ]),
            mget(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn mget_devuelve_una_error_con_si_no_hay_parametro() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();
        bdd.guardar_valor("clave1".to_string(), TipoRedis::Str("1".to_string()));
        bdd.guardar_valor("clave2".to_string(), TipoRedis::Lista(Vec::new()));
        bdd.guardar_valor("clave3".to_string(), TipoRedis::Set(HashSet::new()));
        bdd.guardar_valor("clave4".to_string(), TipoRedis::Str("4".to_string()));

        let mut comando = ComandoInfo::new(vec!["mget".to_string()]);

        assert_eq!(
            ResultadoRedis::Error("ERR wrong number of arguments for mget command".to_string()),
            mget(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn mset_guarda_todos_los_valores() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec![
            "MSET".to_string(),
            "clave1".to_string(),
            "1".to_string(),
            "clave2".to_string(),
            "2".to_string(),
            "clave3".to_string(),
            "3".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::StrSimple("OK".to_string()),
            mset(&mut comando, ptr_hash1)
        );

        assert_eq!(
            Some(&TipoRedis::Str("1".to_string())),
            ptr_hash.lock().unwrap().obtener_valor("clave1")
        );
        assert_eq!(
            Some(&TipoRedis::Str("2".to_string())),
            ptr_hash.lock().unwrap().obtener_valor("clave2")
        );
        assert_eq!(
            Some(&TipoRedis::Str("3".to_string())),
            ptr_hash.lock().unwrap().obtener_valor("clave3")
        );
    }

    #[test]
    fn mset_devuelve_un_error_tener_una_cantidad_de_argumentos_impares() {
        let bdd: BaseDeDatos = BaseDeDatos::new();
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec![
            "MSET".to_string(),
            "clave1".to_string(),
            "1".to_string(),
            "clave2".to_string(),
            "2".to_string(),
            "clave3".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("ERR wrong number of arguments for mset command".to_string()),
            mset(&mut comando, ptr_hash1)
        );

        assert_eq!(None, ptr_hash.lock().unwrap().obtener_valor("clave1"));
        assert_eq!(None, ptr_hash.lock().unwrap().obtener_valor("clave2"));
        assert_eq!(None, ptr_hash.lock().unwrap().obtener_valor("clave3"));
    }

    #[test]
    fn getset_devuelve_el_antiguo_valor_almacenado() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();

        bdd.guardar_valor("clave".to_string(), TipoRedis::Str("clave".to_string()));
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec![
            "GETSET".to_string(),
            "clave".to_string(),
            "nueva_clave".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::StrSimple("clave".to_string()),
            getset(&mut comando, ptr_hash1)
        );

        assert_eq!(
            Some(&TipoRedis::Str("nueva_clave".to_string())),
            ptr_hash.lock().unwrap().obtener_valor("clave")
        );
    }

    #[test]
    fn getset_devuelve_el_nil_si_no_existia_esa_clave() {
        let bdd: BaseDeDatos = BaseDeDatos::new();

        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec![
            "GETSET".to_string(),
            "clave".to_string(),
            "nueva_clave".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Nil, getset(&mut comando, ptr_hash1));

        assert_eq!(
            Some(&TipoRedis::Str("nueva_clave".to_string())),
            ptr_hash.lock().unwrap().obtener_valor("clave")
        );
    }

    #[test]
    fn getset_devuelve_error_porque_la_clave_no_corresponde_a_un_string() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new();

        bdd.guardar_valor("clave".to_string(), TipoRedis::Lista(vec![]));
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec![
            "GETSET".to_string(),
            "clave".to_string(),
            "nueva_clave".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            ),
            getset(&mut comando, ptr_hash1)
        );

        assert_eq!(
            Some(&TipoRedis::Lista(vec![])),
            ptr_hash.lock().unwrap().obtener_valor("clave")
        );
    }
}
