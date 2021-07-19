use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{Comando, ComandoHandler};
use crate::comando_info::ComandoInfo;
use std::sync::{Arc, Mutex};

/*
Comando Lista faltantes:
+ getset
*/
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

pub fn es_comando_string(comando: &str) -> bool {
    let comandos = vec!["GET", "SET", "APPEND"];
    comandos.iter().any(|&c| c == comando)
}

fn get(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    match bdd.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Str(valor)) => ResultadoRedis::BulkStr(valor.to_string()),
        _ => ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
    }
}

fn set(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };

    bdd.lock()
        .unwrap()
        .guardar_valor(clave, TipoRedis::Str(parametro));
    ResultadoRedis::StrSimple("OK".to_string())
}

fn getset(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };

    match bdd
        .lock()
        .unwrap()
        .intercambiar_valor(clave, TipoRedis::Str(parametro))
    {
        Some(TipoRedis::Lista(_)) => return ResultadoRedis::Error("WRONGTYPE".to_string()),
        Some(TipoRedis::Set(_)) => return ResultadoRedis::Error("WRONGTYPE".to_string()),
        Some(TipoRedis::Str(valor_enterior)) => return ResultadoRedis::StrSimple(valor_enterior),
        None => return ResultadoRedis::Nil,
    }
}

fn append(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };
    if bdd.lock().unwrap().existe_clave(&clave) {
        let valor = match get(comando, bdd.clone()) {
            ResultadoRedis::BulkStr(valor) => valor + &parametro,
            _ => return ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
        };

        bdd.lock()
            .unwrap()
            .guardar_valor(clave, TipoRedis::Str(valor.clone()));
        return ResultadoRedis::Int(valor.len() as isize);
    };
    bdd.lock()
        .unwrap()
        .guardar_valor(clave, TipoRedis::Str(parametro.to_string()));
    ResultadoRedis::Int(parametro.len() as isize)
}

fn getdel(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let bdd_clon = Arc::clone(&bdd);
    let resultado = get(comando, bdd_clon);
    bdd.lock().unwrap().eliminar_clave(&clave);
    resultado
}

fn strlen(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    match bdd.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Str(valor)) => ResultadoRedis::Int(valor.len() as isize),
        _ => ResultadoRedis::Error("StrLen error al obtener la clave".to_string()),
    }
}

fn operar_sobre_int(
    comando: &mut ComandoInfo,
    bdd: Arc<Mutex<BaseDeDatos>>,
    f: fn(i32, i32) -> i32,
) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    let valor = match bdd.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Str(valor)) => valor.clone(),
        None => "0".to_string(),
        _ => return ResultadoRedis::Error("WRONGTYPE".to_string()),
    };

    let mut num = match valor.parse::<i32>() {
        Ok(n) => n,
        Err(_) => return ResultadoRedis::Error("Valor no es un int".to_string()),
    };

    let param = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se encontro un parametro".to_string())
        }
    };

    let param = match param.parse::<i32>() {
        Ok(p) => p,
        Err(_) => return ResultadoRedis::Error("Parametro no es un int".to_string()),
    };

    num = f(num, param);
    bdd.lock()
        .unwrap()
        .guardar_valor(clave, TipoRedis::Str(num.to_string()));

    ResultadoRedis::BulkStr(num.to_string())
}

fn decrby(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    operar_sobre_int(comando, bdd, |a, b| a - b)
}

fn incrby(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    operar_sobre_int(comando, bdd, |a, b| a + b)
}

fn mget(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let mut valores = vec![];
    let mut quedan_valores = true;

    while quedan_valores {
        let param = comando.get_parametro();
        match param {
            Some(p) => match bdd.lock().unwrap().obtener_valor(&p) {
                Some(TipoRedis::Str(valor)) => {
                    valores.push(ResultadoRedis::BulkStr(valor.to_string()));
                }
                _ => {
                    valores.push(ResultadoRedis::Nil);
                }
            },
            None => {
                quedan_valores = false;
            }
        }
    }

    if valores.is_empty() {
        return ResultadoRedis::Error("Comando sin claves".to_string());
    }
    ResultadoRedis::Vector(valores)
}

fn mset(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let parametros = match comando.get_parametros() {
        Some(p) => p,
        None => return ResultadoRedis::Error("comando sin parametro".to_string()),
    };

    if parametros.len() % 2 != 0 {
        return ResultadoRedis::Error("wrong number of arguments for MSET".to_string());
    }

    bdd.lock().unwrap().guardar_valores(parametros);
    ResultadoRedis::StrSimple("OK".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn get_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::BulkStr("miValor".to_string()),
            get(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn get_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
            get(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn set_almacena_un_valor_en_el_hash() {
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec![
            "APPEND".to_owned(),
            "miClave".to_string(),
            " palabra".to_owned(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
            append(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn getdel_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));

        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash_clone = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::BulkStr("miValor".to_string()),
            getdel(&mut comando, ptr_hash_clone)
        );
        assert_eq!(
            ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
            getdel(&mut comando, ptr_hash)
        );
    }

    #[test]
    fn getdel_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::Error("GetError error al obtener la clave".to_string()),
            get(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn strlen_devuelve_el_valor_almacenado_en_el_hash() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("miValor".to_string()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::Int(7),
            strlen(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn strlen_devuelve_error_al_ser_llamado_con_una_clave_que_correspondia_a_una_lista() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec!["get".to_string(), "miClave".to_string()]);

        assert_eq!(
            ResultadoRedis::Error("StrLen error al obtener la clave".to_string()),
            strlen(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn decrby_resta_correcatemente_un_valor_entero_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec![
            "decrby".to_string(),
            "miClave".to_string(),
            "-1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE".to_string()),
            decrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn decrby_devuelve_error_un_valor_erroneo_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("1".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "decrby".to_string(),
            "miClave".to_string(),
            "a".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("Parametro no es un int".to_string()),
            decrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn incrby_resta_correcatemente_un_valor_entero_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Lista(Vec::new()));
        let mut comando = ComandoInfo::new(vec![
            "incrby".to_string(),
            "miClave".to_string(),
            "5".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE".to_string()),
            incrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn incrby_devuelve_error_un_valor_erroneo_a_una_clave_parseable() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("miClave".to_string(), TipoRedis::Str("1".to_string()));
        let mut comando = ComandoInfo::new(vec![
            "incrby".to_string(),
            "miClave".to_string(),
            "a".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("Parametro no es un int".to_string()),
            incrby(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn mget_devuelve_una_lista_con_todos_los_valores_de_las_claves() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
        bdd.guardar_valor("clave1".to_string(), TipoRedis::Str("1".to_string()));
        bdd.guardar_valor("clave2".to_string(), TipoRedis::Lista(Vec::new()));
        bdd.guardar_valor("clave3".to_string(), TipoRedis::Set(HashSet::new()));
        bdd.guardar_valor("clave4".to_string(), TipoRedis::Str("4".to_string()));

        let mut comando = ComandoInfo::new(vec!["mget".to_string()]);

        assert_eq!(
            ResultadoRedis::Error("Comando sin claves".to_string()),
            mget(&mut comando, Arc::new(Mutex::new(bdd)))
        );
    }

    #[test]
    fn mset_guarda_todos_los_valores() {
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());
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
            ResultadoRedis::Error("wrong number of arguments for MSET".to_string()),
            mset(&mut comando, ptr_hash1)
        );

        assert_eq!(None, ptr_hash.lock().unwrap().obtener_valor("clave1"));
        assert_eq!(None, ptr_hash.lock().unwrap().obtener_valor("clave2"));
        assert_eq!(None, ptr_hash.lock().unwrap().obtener_valor("clave3"));
    }

    #[test]
    fn getset_devuelve_el_antiguo_valor_almacenado() {
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());

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
        let bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());

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
        let mut bdd: BaseDeDatos = BaseDeDatos::new("eliminame.txt".to_string());

        bdd.guardar_valor("clave".to_string(), TipoRedis::Lista(vec![]));
        let ptr_hash = Arc::new(Mutex::new(bdd));
        let ptr_hash1 = Arc::clone(&ptr_hash);

        let mut comando = ComandoInfo::new(vec![
            "GETSET".to_string(),
            "clave".to_string(),
            "nueva_clave".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE".to_string()),
            getset(&mut comando, ptr_hash1)
        );

        assert_eq!(
            Some(&TipoRedis::Lista(vec![])),
            ptr_hash.lock().unwrap().obtener_valor("clave")
        );
    }
}
