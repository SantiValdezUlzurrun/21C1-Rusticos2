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

    if 0 < indice && indice < tamanio {
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
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let indice = match comando.get_parametro() {
        Some(p) => {
            match p.parse() {
                Ok(i) => i,
                Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
            }
        },
        None => 1,
    };

    let mut lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => return ResultadoRedis::Nil,
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    let a_devolver: Vec<_> = lista.drain(0..indice).collect();

    if lista.len() > 0 {
        base_de_datos.lock().unwrap().guardar_valor(clave, TipoRedis::Lista(lista));
    } else {
        base_de_datos.lock().unwrap().eliminar_clave(&clave);
    }

    if a_devolver.len() == 1 {
        return ResultadoRedis::BulkStr(a_devolver[0].clone());
    }

    ResultadoRedis::Vector(
        a_devolver.iter()
             .map(|el| ResultadoRedis::BulkStr(el.to_string()))
             .collect()
    )
}

pub fn lpush(comando: &mut ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let mut lista = match base_de_datos.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        None => Vec::new(),
        _ => return ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
    };

    while let Some(parametro) = comando.get_parametro() {

        lista.insert(0, parametro);
    }
    let long = lista.len();
    base_de_datos.lock().unwrap().guardar_valor(clave, TipoRedis::Lista(lista));

    ResultadoRedis::Int(long)
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

        let mut comando = ComandoInfo::new(vec![
            "llen".to_string(),
            "milista".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Int(0),
            llen(&mut comando, ptr)
        );
    }

    #[test]
    fn llen_si_se_llama_llen_a_un_string_se_devuelve_un_error_de_tipo() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("milista".to_string(), TipoRedis::Str("hola".to_string()));

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "llen".to_string(),
            "milista".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            llen(&mut comando, ptr)
        );
    }

    #[test]
    fn llen_si_se_llama_llen_a_una_lista_devuelve_la_longitud_correctamente() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("milista".to_string(), TipoRedis::Lista(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]));

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "llen".to_string(),
            "milista".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Int(4),
            llen(&mut comando, ptr)
        );
    }

    #[test]
    fn lpop_si_no_existe_la_lista_devuelve_nil(){
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpop".to_string(),
            "milista".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Nil,
            lpop(&mut comando, ptr)
        );
    }

    #[test]
    fn lpop_si_existia_una_lista_y_luego_se_la_elimina_y_se_vuelve_a_hacer_pop_devuelve_nil(){
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("milista".to_string(), TipoRedis::Lista(vec!["unvalor".to_string()]));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpop".to_string(),
            "milista".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("unvalor".to_string()),
            lpop(&mut comando, Arc::clone(&ptr))
        );
        assert_eq!(
            ResultadoRedis::Nil,
            lpop(&mut comando, ptr)
        );
    }

    #[test]
    fn lpop_si_se_llama_sobre_un_tipo_distinto_a_una_lista_devuelve_wrong_type(){
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpop".to_string(),
            "clave".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Error("WRONGTYPE La clave no es una lista".to_string()),
            lpop(&mut comando, ptr)
        );
    }

    #[test]
    fn lpop_si_se_llama_a_lpop_sin_el_parametro_count_se_devuelve_el_resultado_correcto(){
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("milista".to_string(), TipoRedis::Lista(vec!["1".to_string(), "2".to_string()]));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpop".to_string(),
            "milista".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::BulkStr("1".to_string()),
            lpop(&mut comando, ptr)
        );
    }

    #[test]
    fn lpop_si_se_llama_a_lpop_con_el_parametro_count_se_devuelve_el_resultado_correcto(){
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());

        data_base.guardar_valor("milista".to_string(), TipoRedis::Lista(vec!["1".to_string(), "2".to_string()]));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpop".to_string(),
            "milista".to_string(),
            "2".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Vector(vec![ResultadoRedis::BulkStr("1".to_string()), ResultadoRedis::BulkStr("2".to_string())]),
            lpop(&mut comando, ptr)
        );
    }

    #[test]
    fn lpush_si_se_pushea_a_alguna_clave_existente_que_no_es_una_lista_devuelve_wrong_type(){
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
    fn lpush_si_no_existe_la_lista_se_crea_con_los_parametros_devolviendo_la_longitud_adecuada(){
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());

        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpush".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Int(3),
            lpush(&mut comando, ptr)
        );
    }
    #[test]
    fn lpush_si_no_existe_la_lista_se_crea_con_los_parametros_en_el_orden_adecuado(){
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

        let lista = ptr.lock().unwrap().obtener_valor("milista").unwrap().clone();

        assert_eq!(
            TipoRedis::Lista(vec!["c".to_string(), "b".to_string(), "a".to_string()]),
            lista,
        );
    }

    #[test]
    fn lpush_cuando_se_pushea_a_una_lista_se_ordena_adecuadamente(){
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("milista".to_string(), TipoRedis::Lista(vec!["d".to_string()]));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "lpush".to_string(),
            "milista".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        lpush(&mut comando, Arc::clone(&ptr));

        let lista = ptr.lock().unwrap().obtener_valor("milista").unwrap().clone();

        assert_eq!(
            TipoRedis::Lista(vec!["c".to_string(), "b".to_string(), "a".to_string(), "d".to_string()]),
            lista,
        );
    }

}
