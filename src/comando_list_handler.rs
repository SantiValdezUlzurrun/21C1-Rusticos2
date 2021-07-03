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
            "LINDEX" => lindex,
            "LPOP" => lpop,
            "RPOP" => rpop,
            "LPUSH" => lpush,
            "LPUSHX" => lpushx,
            "RPUSH" => rpush,
            "RPUSHX" => rpushx,
            "LRANGE" => lrange,
            "LREM" => lrem,
            "LSET" => lset,
            _ => llen,
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

    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };
    let lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        _ => return ResultadoRedis::Error("WRONGTYPE".to_string()),
    };
    let indice: i32 = match parametro.parse() {
        Ok(v) => v,
        Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
    };
    let tamanio = lista.len() as i32;

    if 0 <= indice && indice < tamanio {
        ResultadoRedis::BulkStr(lista[indice as usize].clone())
    } else if 0 > indice && tamanio + indice >= 0 {
        ResultadoRedis::BulkStr(lista[(tamanio + indice) as usize].clone())
    } else {
        ResultadoRedis::BulkStr("nil".to_string())
    }
}

pub fn llen(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    let lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => Vec::new(),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    ResultadoRedis::Int(lista.len())
}

pub fn lpop(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    pop(comando, base_de_datos, false)
}

pub fn rpop(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    pop(comando, base_de_datos, true)
}

fn pop(
    comando: &mut ComandoInfo,
    base_de_datos: Arc<Mutex<BaseDeDatos>>,
    reversed: bool,
) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let indice = match comando.get_parametro() {
        Some(p) => match p.parse() {
            Ok(i) => i,
            Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
        },
        None => 1,
    };

    let mut lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => return ResultadoRedis::Nil,
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    if reversed {
        lista.reverse();
    }

    let mut a_devolver: Vec<_> = lista.drain(0..indice).collect();

    if reversed {
        lista.reverse();
        a_devolver.reverse();
    }

    if !lista.is_empty() {
        base_de_datos
            .lock()
            .unwrap()
            .guardar_valor(clave, TipoRedis::Lista(lista));
    } else {
        base_de_datos.lock().unwrap().eliminar_clave(&clave);
    }

    if a_devolver.len() == 1 {
        return ResultadoRedis::BulkStr(a_devolver[0].clone());
    }

    ResultadoRedis::Vector(
        a_devolver
            .iter()
            .map(|el| ResultadoRedis::BulkStr(el.to_string()))
            .collect(),
    )
}

fn push(
    mut lista: Vec<String>,
    clave: String,
    comando: &mut ComandoInfo,
    base_de_datos: Arc<Mutex<BaseDeDatos>>,
    reversed: bool,
) -> ResultadoRedis {
    while let Some(parametro) = comando.get_parametro() {
        if !reversed {
            lista.insert(0, parametro);
        } else {
            lista.push(parametro);
        }
    }
    let long = lista.len();
    base_de_datos
        .lock()
        .unwrap()
        .guardar_valor(clave, TipoRedis::Lista(lista));

    ResultadoRedis::Int(long)
}

pub fn lpush(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => Vec::new(),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    push(lista, clave, comando, base_de_datos, false)
}

pub fn lpushx(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => return ResultadoRedis::Int(0),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    push(lista, clave, comando, base_de_datos, false)
}

pub fn rpush(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => Vec::new(),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    push(lista, clave, comando, base_de_datos, true)
}

pub fn rpushx(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => return ResultadoRedis::Int(0),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    push(lista, clave, comando, base_de_datos, true)
}

pub fn lrange(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let inicio: i32 = match comando.get_parametro() {
        Some(p) => match p.parse() {
            Ok(i) => i,
            Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
        },
        None => return ResultadoRedis::Error("ERR numero equivocado de parametros".to_string()),
    };
    let fin: i32 = match comando.get_parametro() {
        Some(p) => match p.parse() {
            Ok(i) => i,
            Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
        },
        None => return ResultadoRedis::Error("ERR numero equivocado de parametros".to_string()),
    };
    let mut lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => return ResultadoRedis::Vector(vec![]),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    let (a, b) = match obtener_intervalo(inicio, fin, lista.len() as i32) {
        Some((a, b)) => (a, b),
        None => return ResultadoRedis::Vector(vec![]),
    };
    let a_devolver: Vec<_> = lista.drain(a..(b + 1)).collect();

    ResultadoRedis::Vector(
        a_devolver
            .iter()
            .map(|el| ResultadoRedis::BulkStr(el.to_string()))
            .collect(),
    )
}

fn obtener_intervalo(inicio: i32, fin: i32, limite: i32) -> Option<(usize, usize)> {
    if inicio < fin {
        let a = match obtener_indice_inferior(inicio, limite) {
            Some(a) => a,
            None => return None,
        };
        let b = match obtener_indice_superior(fin, limite) {
            Some(b) => b,
            None => return None,
        };

        return Some((a as usize, b as usize));
    }
    None
}

fn obtener_indice_inferior(inicio: i32, limite: i32) -> Option<i32> {
    if esta_en_rango_lista(inicio, limite) {
        Some(inicio)
    } else if inicio < 0 {
        if esta_en_rango_lista(inicio + limite, limite) {
            Some(inicio + limite)
        } else {
            Some(0)
        }
    } else {
        None
    }
}

fn obtener_indice_superior(fin: i32, limite: i32) -> Option<i32> {
    if esta_en_rango_lista(fin, limite) {
        Some(fin)
    } else if fin < 0 {
        if esta_en_rango_lista(fin + limite, limite) {
            Some(fin + limite)
        } else {
            None
        }
    } else {
        Some(limite - 1)
    }
}

fn esta_en_rango_lista(valor: i32, limite: i32) -> bool {
    0 <= valor && valor < limite
}

pub fn lrem(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let indice: i32 = match comando.get_parametro() {
        Some(p) => match p.parse() {
            Ok(i) => i,
            Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
        },
        None => return ResultadoRedis::Error("ERR numero equivocado de parametros".to_string()),
    };
    let a_eliminar = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };

    let mut lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => return ResultadoRedis::Int(0),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    let mut i = indice;
    if indice < 0 {
        lista.reverse();
        i = indice.abs();
    }
    let mut cant_eliminada = 0;
    let mut lista_filtrada: Vec<String> = Vec::new();
    let iter = lista.iter();
    for valor in iter {
        if !(valor.eq(&a_eliminar) && cant_eliminada < i) {
            lista_filtrada.push(valor.clone());
        } else {
            cant_eliminada += 1;
        }
    }
    if indice < 0 {
        lista_filtrada.reverse();
    }

    if !lista_filtrada.is_empty() {
        base_de_datos
            .lock()
            .unwrap()
            .guardar_valor(clave, TipoRedis::Lista(lista_filtrada));
    } else {
        base_de_datos.lock().unwrap().eliminar_clave(&clave);
    }
    ResultadoRedis::Int(cant_eliminada as usize)
}

pub fn lset(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
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

    let indice: i32 = match parametro.parse() {
        Ok(v) => v,
        Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
    };

    let parametro = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };

    let mut lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    let tamanio = lista.len() as i32;

    if 0 <= indice && indice < tamanio {
        lista[indice as usize] = parametro;
    } else if 0 > indice && tamanio + indice >= 0 {
        lista[(tamanio + indice) as usize] = parametro;
    } else {
        return ResultadoRedis::Error("ERR index out of range".to_string());
    }

    base_de_datos
        .lock()
        .unwrap()
        .guardar_valor(clave, TipoRedis::Lista(lista));
    ResultadoRedis::StrSimple("OK".to_string())
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

    #[test]
    fn llen_la_longitud_de_una_lista_inexistente_es_cero() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["llen".to_string(), "milista".to_string()]);

        assert_eq!(ResultadoRedis::Int(0), llen(&mut comando, ptr));
    }

    #[test]
    fn llen_si_se_llama_llen_a_un_string_se_devuelve_un_error_de_tipo() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("milista".to_string(), TipoRedis::Str("hola".to_string()));

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["llen".to_string(), "milista".to_string()]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            llen(&mut comando, ptr)
        );
    }

    #[test]
    fn llen_si_se_llama_llen_a_una_lista_devuelve_la_longitud_correctamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ]),
        );

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["llen".to_string(), "milista".to_string()]);

        assert_eq!(ResultadoRedis::Int(4), llen(&mut comando, ptr));
    }

    #[test]
    fn lpop_si_no_existe_la_lista_devuelve_nil() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["lpop".to_string(), "milista".to_string()]);

        assert_eq!(ResultadoRedis::Nil, lpop(&mut comando, ptr));
    }

    #[test]
    fn lpop_si_existia_una_lista_y_luego_se_la_elimina_y_se_vuelve_a_hacer_pop_devuelve_nil() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["unvalor".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["lpop".to_string(), "milista".to_string()]);

        assert_eq!(
            ResultadoRedis::BulkStr("unvalor".to_string()),
            lpop(&mut comando, Arc::clone(&ptr))
        );
        assert_eq!(ResultadoRedis::Nil, lpop(&mut comando, ptr));
    }

    #[test]
    fn lpop_si_se_llama_sobre_un_tipo_distinto_a_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["lpop".to_string(), "clave".to_string()]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            lpop(&mut comando, ptr)
        );
    }

    #[test]
    fn lpop_si_se_llama_a_lpop_sin_el_parametro_count_se_devuelve_el_resultado_correcto() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["1".to_string(), "2".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["lpop".to_string(), "milista".to_string()]);

        assert_eq!(
            ResultadoRedis::BulkStr("1".to_string()),
            lpop(&mut comando, ptr)
        );
    }

    #[test]
    fn lpop_si_se_llama_a_lpop_con_el_parametro_count_se_devuelve_el_resultado_correcto() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["1".to_string(), "2".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpop".to_string(),
            "milista".to_string(),
            "2".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![
                ResultadoRedis::BulkStr("1".to_string()),
                ResultadoRedis::BulkStr("2".to_string())
            ]),
            lpop(&mut comando, ptr)
        );
    }

    #[test]
    fn lpush_si_se_pushea_a_alguna_clave_existente_que_no_es_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpush".to_string(),
            "clave".to_string(),
            "valor".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            lpush(&mut comando, ptr)
        );
    }

    #[test]
    fn lpush_si_no_existe_la_lista_se_crea_con_los_parametros_devolviendo_la_longitud_adecuada() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpush".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(3), lpush(&mut comando, ptr));
    }
    #[test]
    fn lpush_si_no_existe_la_lista_se_crea_con_los_parametros_en_el_orden_adecuado() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpush".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        lpush(&mut comando, Arc::clone(&ptr));

        let lista = ptr
            .lock()
            .unwrap()
            .obtener_valor("milista")
            .unwrap()
            .clone();

        assert_eq!(
            TipoRedis::Lista(vec!["c".to_string(), "b".to_string(), "a".to_string()]),
            lista,
        );
    }

    #[test]
    fn lpush_cuando_se_pushea_a_una_lista_se_ordena_adecuadamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["d".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpush".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        lpush(&mut comando, Arc::clone(&ptr));

        let lista = ptr
            .lock()
            .unwrap()
            .obtener_valor("milista")
            .unwrap()
            .clone();

        assert_eq!(
            TipoRedis::Lista(vec![
                "c".to_string(),
                "b".to_string(),
                "a".to_string(),
                "d".to_string()
            ]),
            lista,
        );
    }

    #[test]
    fn lpushx_si_se_pushea_a_alguna_clave_existente_que_no_es_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpushx".to_string(),
            "clave".to_string(),
            "valor".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            lpushx(&mut comando, ptr)
        );
    }

    #[test]
    fn lpushx_si_no_existe_la_lista_no_se_crea() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpushx".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(0), lpushx(&mut comando, ptr));
    }

    #[test]
    fn lpushx_cuando_se_pushea_a_una_lista_se_ordena_adecuadamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["d".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpushx".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Int(4),
            lpushx(&mut comando, Arc::clone(&ptr))
        );

        let lista = ptr
            .lock()
            .unwrap()
            .obtener_valor("milista")
            .unwrap()
            .clone();

        assert_eq!(
            TipoRedis::Lista(vec![
                "c".to_string(),
                "b".to_string(),
                "a".to_string(),
                "d".to_string()
            ]),
            lista,
        );
    }

    #[test]
    fn lrange_si_se_lo_llama_sobre_algo_que_no_es_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrange".to_string(),
            "clave".to_string(),
            "1".to_string(),
            "50".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            lrange(&mut comando, ptr)
        );
    }

    #[test]
    fn lrange_si_se_pide_el_rango_de_una_lista_inexistente_devuelve_vector_vacio() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrange".to_string(),
            "clave".to_string(),
            "1".to_string(),
            "50".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(Vec::new()),
            lrange(&mut comando, ptr)
        );
    }

    #[test]
    fn lrange_rangos_positivos_devuelven_los_elementos_correctamente_hasta_el_indice_final_inclusive(
    ) {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "clave".to_string(),
            TipoRedis::Lista(vec!["0".to_string(), "1".to_string(), "2".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrange".to_string(),
            "clave".to_string(),
            "1".to_string(),
            "2".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![
                ResultadoRedis::BulkStr("1".to_string()),
                ResultadoRedis::BulkStr("2".to_string())
            ]),
            lrange(&mut comando, ptr)
        );
    }

    #[test]
    fn lrange_rangos_enteros_devuelven_los_elementos_correctamente_hasta_el_indice_final_inclusive()
    {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "clave".to_string(),
            TipoRedis::Lista(vec!["0".to_string(), "1".to_string(), "2".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrange".to_string(),
            "clave".to_string(),
            "-35".to_string(),
            "0".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![ResultadoRedis::BulkStr("0".to_string())]),
            lrange(&mut comando, ptr)
        );
    }

    #[test]
    fn lrange_rangos_enteros_con_interseccion_nula_devuelve_vector_vacio() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "clave".to_string(),
            TipoRedis::Lista(vec!["0".to_string(), "1".to_string(), "2".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrange".to_string(),
            "clave".to_string(),
            "-2".to_string(),
            "0".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Vector(vec![]), lrange(&mut comando, ptr));
    }

    #[test]
    fn lrem_si_se_pide_eliminar_de_una_clave_que_no_es_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrem".to_string(),
            "clave".to_string(),
            "-2".to_string(),
            "valor".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            lrem(&mut comando, ptr)
        );
    }

    #[test]
    fn lrem_si_se_pide_eliminar_un_valor_que_no_esta_en_la_lista_no_se_devuelve_un_cero() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrem".to_string(),
            "clave".to_string(),
            "-2".to_string(),
            "valor".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(0), lrem(&mut comando, ptr));
    }

    #[test]
    fn lrem_si_se_pide_eliminar_dos_claves_de_izquierda_a_derecha_se_eliminan_correctamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "clave".to_string(),
            TipoRedis::Lista(vec![
                "hola".to_string(),
                "que".to_string(),
                "hola".to_string(),
                "dame".to_string(),
                "hola".to_string(),
            ]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrem".to_string(),
            "clave".to_string(),
            "2".to_string(),
            "hola".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(2), lrem(&mut comando, Arc::clone(&ptr)));

        assert_eq!(
            TipoRedis::Lista(vec![
                "que".to_string(),
                "dame".to_string(),
                "hola".to_string()
            ]),
            ptr.lock().unwrap().obtener_valor("clave").unwrap().clone()
        );
    }

    #[test]
    fn lrem_si_se_pide_eliminar_dos_claves_de_derecha_a_izquierda_se_eliminan_correctamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "clave".to_string(),
            TipoRedis::Lista(vec![
                "hola".to_string(),
                "que".to_string(),
                "hola".to_string(),
                "dame".to_string(),
                "hola".to_string(),
                "pepe".to_string(),
            ]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lrem".to_string(),
            "clave".to_string(),
            "-2".to_string(),
            "hola".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(2), lrem(&mut comando, Arc::clone(&ptr)));

        assert_eq!(
            TipoRedis::Lista(vec![
                "hola".to_string(),
                "que".to_string(),
                "dame".to_string(),
                "pepe".to_string()
            ]),
            ptr.lock().unwrap().obtener_valor("clave").unwrap().clone()
        );
    }

    #[test]
    fn lset_si_se_lo_llama_sobre_algo_que_no_es_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lset".to_string(),
            "clave".to_string(),
            "1".to_string(),
            "valor".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            lset(&mut comando, ptr)
        );
    }

    #[test]
    fn lset_inserta_en_la_lista_adecuadamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Lista(vec!["a".to_string()]));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lset".to_string(),
            "clave".to_string(),
            "0".to_string(),
            "b".to_string(),
        ]);
        assert_eq!(
            ResultadoRedis::StrSimple("OK".to_string()),
            lset(&mut comando, Arc::clone(&ptr))
        );

        let lista = ptr.lock().unwrap().obtener_valor("clave").unwrap().clone();

        assert_eq!(TipoRedis::Lista(vec!["b".to_string()]), lista);
    }

    #[test]
    fn rpop_si_no_existe_la_lista_devuelve_nil() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["rpop".to_string(), "milista".to_string()]);

        assert_eq!(ResultadoRedis::Nil, rpop(&mut comando, ptr));
    }

    #[test]
    fn rpop_si_existia_una_lista_y_luego_se_la_elimina_y_se_vuelve_a_hacer_pop_devuelve_nil() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["unvalor".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["rpop".to_string(), "milista".to_string()]);

        assert_eq!(
            ResultadoRedis::BulkStr("unvalor".to_string()),
            rpop(&mut comando, Arc::clone(&ptr))
        );
        assert_eq!(ResultadoRedis::Nil, rpop(&mut comando, ptr));
    }

    #[test]
    fn rpop_si_se_llama_sobre_un_tipo_distinto_a_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["rpop".to_string(), "clave".to_string()]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            rpop(&mut comando, ptr)
        );
    }

    #[test]
    fn rpop_si_se_llama_a_rpop_sin_el_parametro_count_se_devuelve_el_resultado_correcto() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["1".to_string(), "2".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec!["rpop".to_string(), "milista".to_string()]);

        assert_eq!(
            ResultadoRedis::BulkStr("2".to_string()),
            rpop(&mut comando, ptr)
        );
    }

    #[test]
    fn rpop_si_se_llama_a_rpop_con_el_parametro_count_se_devuelve_el_resultado_correcto() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "rpop".to_string(),
            "milista".to_string(),
            "2".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![
                ResultadoRedis::BulkStr("2".to_string()),
                ResultadoRedis::BulkStr("3".to_string())
            ]),
            rpop(&mut comando, ptr)
        );
    }

    #[test]
    fn rpush_si_se_pushea_a_alguna_clave_existente_que_no_es_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "rpush".to_string(),
            "clave".to_string(),
            "valor".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            rpush(&mut comando, ptr)
        );
    }

    #[test]
    fn rpush_si_no_existe_la_lista_se_crea_con_los_parametros_devolviendo_la_longitud_adecuada() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "rpush".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(3), rpush(&mut comando, ptr));
    }
    #[test]
    fn rpush_si_no_existe_la_lista_se_crea_con_los_parametros_en_el_orden_adecuado() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "rpush".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        rpush(&mut comando, Arc::clone(&ptr));

        let lista = ptr
            .lock()
            .unwrap()
            .obtener_valor("milista")
            .unwrap()
            .clone();

        assert_eq!(
            TipoRedis::Lista(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            lista,
        );
    }

    #[test]
    fn rpush_cuando_se_pushea_a_una_lista_se_ordena_adecuadamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["d".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpush".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        rpush(&mut comando, Arc::clone(&ptr));

        let lista = ptr
            .lock()
            .unwrap()
            .obtener_valor("milista")
            .unwrap()
            .clone();

        assert_eq!(
            TipoRedis::Lista(vec![
                "d".to_string(),
                "a".to_string(),
                "b".to_string(),
                "c".to_string()
            ]),
            lista,
        );
    }

    #[test]
    fn rpushx_si_se_pushea_a_alguna_clave_existente_que_no_es_una_lista_devuelve_wrong_type() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "rpushx".to_string(),
            "clave".to_string(),
            "valor".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            rpushx(&mut comando, ptr)
        );
    }

    #[test]
    fn rpushx_si_no_existe_la_lista_no_se_crea() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpushx".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        assert_eq!(ResultadoRedis::Int(0), rpushx(&mut comando, ptr));
    }

    #[test]
    fn rpushx_cuando_se_pushea_a_una_lista_se_ordena_adecuadamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "milista".to_string(),
            TipoRedis::Lista(vec!["d".to_string()]),
        );
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "rpushx".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Int(4),
            rpushx(&mut comando, Arc::clone(&ptr))
        );

        let lista = ptr
            .lock()
            .unwrap()
            .obtener_valor("milista")
            .unwrap()
            .clone();

        assert_eq!(
            TipoRedis::Lista(vec![
                "d".to_string(),
                "a".to_string(),
                "b".to_string(),
                "c".to_string()
            ]),
            lista,
        );
    }
}