use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{Comando, ComandoHandler};
use crate::comando_info::ComandoInfo;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
pub struct ComandoKeyHandler {
    comando: ComandoInfo,
    a_ejecutar: Comando,
}

impl ComandoKeyHandler {
    pub fn new(comando: ComandoInfo) -> Self {
        let a_ejecutar = match comando.get_nombre().as_str() {
            "COPY" => copy,
            "DEL" => del,
            "EXISTS" => exists,
            "RENAME" => rename,
            "EXPIRE" => expire,
            "EXPIREAT" => expireat,
            "PERSIST" => persist,
            "TTL" => ttl,
            "TOUCH" => touch,
            "KEYS" => keys,
            "SORT" => sort,
            _ => tipo,
        };
        ComandoKeyHandler {
            comando,
            a_ejecutar: Box::new(a_ejecutar),
        }
    }
}

impl ComandoHandler for ComandoKeyHandler {
    fn ejecutar(mut self: Box<Self>, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&mut self.comando, bdd)
    }
}

pub fn es_comando_key(comando: &str) -> bool {
    let comandos = vec!["COPY", "DEL", "EXISTS", "RENAME", "TYPE"];
    comandos.iter().any(|&c| c == comando)
}

fn copy(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
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
    match bdd.lock().unwrap().copiar_valor(&clave, &parametro) {
        Some(_) => ResultadoRedis::Int(1),
        None => ResultadoRedis::Int(0),
    }
}

fn rename(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    let clon = Arc::clone(&bdd);
    match copy(comando, bdd) {
        ResultadoRedis::Error(_) => {
            ResultadoRedis::Error("ErrorRename clave no encontrada".to_string())
        }
        _ => {
            let vector = vec!["rename".to_string(), clave];
            let mut comando = ComandoInfo::new(vector);
            del(&mut comando, clon);
            ResultadoRedis::StrSimple("Ok".to_string())
        }
    }
}

fn tipo(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };
    match bdd.lock().unwrap().obtener_valor(&clave) {
        Some(TipoRedis::Str(_)) => ResultadoRedis::BulkStr("string".to_string()),
        Some(TipoRedis::Lista(_)) => ResultadoRedis::BulkStr("lista".to_string()),
        Some(TipoRedis::Set(_)) => ResultadoRedis::BulkStr("set".to_string()),
        _ => ResultadoRedis::BulkStr("none".to_string()),
    }
}

fn recorrer_y_ejecutar(
    comando: &mut ComandoInfo,
    base_de_datos: Arc<Mutex<BaseDeDatos>>,
    funcion: Box<dyn Fn(&str)>,
) -> ResultadoRedis {
    let mut claves_eliminadas = 0;

    while let Some(clave) = comando.get_parametro() {
        if base_de_datos.lock().unwrap().existe_clave(&clave) {
            (funcion)(&clave);
            claves_eliminadas += 1;
        }
    }
    ResultadoRedis::Int(claves_eliminadas)
}

fn del(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clon = Arc::clone(&bdd);
    recorrer_y_ejecutar(
        comando,
        bdd,
        Box::new(move |clave| {
            clon.lock().unwrap().eliminar_clave(clave);
        }),
    )
}

fn exists(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    recorrer_y_ejecutar(comando, bdd, Box::new(|_| {}))
}

fn expire(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    let parametro: u64 = match comando.get_parametro() {
        Some(p) => match p.parse() {
            Ok(t) => t,
            Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
        },
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };

    let resultado = bdd
        .lock()
        .unwrap()
        .actualizar_valor_con_expiracion(clave, parametro);
    ResultadoRedis::Int(resultado as isize)
}

fn expireat(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    let parametro: u64 = match comando.get_parametro() {
        Some(p) => match p.parse() {
            Ok(t) => t,
            Err(_) => return ResultadoRedis::Error("WrongType parametro no numerico".to_string()),
        },
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };

    let tiempo_desde_epoch = UNIX_EPOCH + Duration::from_secs(parametro);
    let tiempo_a_esperar = match tiempo_desde_epoch.duration_since(SystemTime::now()) {
        Ok(d) => d,
        Err(_) => {
            return ResultadoRedis::Error("TimeError el tiempo desde epoch ya sucedio".to_string())
        }
    };
    let resultado = bdd
        .lock()
        .unwrap()
        .actualizar_valor_con_expiracion(clave, tiempo_a_esperar.as_secs());
    ResultadoRedis::Int(resultado as isize)
}

fn persist(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    let resultado = bdd.lock().unwrap().actualizar_valor_sin_expiracion(clave);
    ResultadoRedis::Int(resultado as isize)
}

fn ttl(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clave = match comando.get_clave() {
        Some(c) => c,
        None => return ResultadoRedis::Error("ClaveError no se encontro una clave".to_string()),
    };

    let resultado = bdd.lock().unwrap().obtener_expiracion(&clave);
    ResultadoRedis::Int(resultado)
}

fn touch(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let mut acum = 0;
    while let Some(clave) = comando.get_parametro() {
        acum += bdd.lock().unwrap().actualizar_ultimo_acceso(clave);
    }
    ResultadoRedis::Int(acum)
}

fn keys(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let re = match comando.get_parametro() {
        Some(p) => p,
        None => {
            return ResultadoRedis::Error("ParametroError no se envio el parametro".to_string())
        }
    };

    let vector: Vec<String> = bdd.lock().unwrap().claves(&re);

    ResultadoRedis::Vector(
        vector
            .iter()
            .map(|v| ResultadoRedis::BulkStr(v.to_string()))
            .collect(),
    )
}

fn es_parseable(num: &str) -> bool {
    num.parse::<i32>().is_ok()
}

fn tiene_solo_valores_numericos(valores: Vec<String>) -> bool {
    for valor in valores.iter() {
        if !es_parseable(valor) {
            return false;
        }
    }
    true
}

fn selecionar_rango(parametros: Vec<String>, valores: Vec<String>) -> Option<Vec<String>> {
    let rango = parametros
        .into_iter()
        .filter(|i| es_parseable(i))
        .collect::<Vec<_>>();

    let offset = match rango[0].parse::<i32>() {
        Ok(num) => num,
        Err(_) => return None,
    };
    let count = match rango[1].parse::<i32>() {
        Ok(num) => num,
        Err(_) => return None,
    };
    let mut index_min = 0;
    let mut index_max = valores.len() - 1;

    if count == 0 {
        return Some(vec![]);
    }

    if 0 <= offset && offset < valores.len() as i32 {
        index_min = offset as usize;
    }
    if 0 <= count && count < valores.len() as i32 - offset {
        index_max = count as usize + offset as usize - 1;
    }

    Some(valores[index_min..=index_max].to_vec())
}

fn sort_elemento_con_pesos_interno(
    mut valores: Vec<String>,
    parametros: Vec<String>,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    if !parametros.contains(&"alpha".to_string()) {
        match tiene_solo_valores_numericos(valores.clone()) {
            true => {}
            false => {
                return ResultadoRedis::Error(
                    "ERR One or more scores can't be converted into double".to_string(),
                )
            }
        }
    }
    valores.sort();
    sort_configuracion_lista_ordenada(parametros, valores, bdd)
}

fn obetener_tupla_valor_peso(
    valores: Vec<String>,
    pesos: Vec<String>,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> Option<Vec<(String, String)>> {
    let mut tuplas: Vec<(String, String)> = vec![];
    for valor in &valores {
        let mut split_valor = valor.rsplit('_');
        let indice_valor = split_valor.next();

        for peso in &pesos {
            let mut indice_peso = peso.rsplit('_');
            let indice_peso = indice_peso.next();

            if indice_peso == indice_valor {
                let peso = match bdd.lock().unwrap().obtener_valor(&peso) {
                    Some(TipoRedis::Str(peso)) => peso.clone(),
                    _ => return None,
                };
                tuplas.push((valor.clone(), peso.to_string()))
            }
        }
    }
    Some(tuplas)
}

fn sort_configuracion_lista_ordenada(
    parametros: Vec<String>,
    mut valores: Vec<String>,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    if parametros.contains(&"desc".to_string()) {
        valores.reverse();
    }

    if parametros.contains(&"limit".to_string()) {
        valores = match selecionar_rango(parametros.clone(), valores) {
            Some(r) => r,
            None => return ResultadoRedis::Error("ERR in sort limit offset count".to_string()),
        };
    }

    if parametros.contains(&"store".to_string()) {
        let mut split_get = parametros.rsplit(|p| p == &"store".to_string());
        let clave = &split_get.next().unwrap()[0];

        let tamanio = valores.len();

        bdd.lock()
            .unwrap()
            .guardar_valor(clave.to_string(), TipoRedis::Lista(valores));
        return ResultadoRedis::StrSimple(tamanio.to_string());
    }

    ResultadoRedis::Vector(
        valores
            .iter()
            .map(|x| ResultadoRedis::StrSimple(x.to_string()))
            .collect::<Vec<ResultadoRedis>>(),
    )
}

fn sort_elemento_con_pesos_externos(
    valores: Vec<String>,
    parametros: Vec<String>,
    bdd: Arc<Mutex<BaseDeDatos>>,
) -> ResultadoRedis {
    let mut split_by = parametros.rsplit(|p| p == &"by".to_string());
    let pesos = bdd.lock().unwrap().claves(&split_by.next().unwrap()[0]);

    if pesos.len() != valores.len() {
        return ResultadoRedis::Vector(
            valores
                .iter()
                .map(|x| ResultadoRedis::StrSimple(x.to_string()))
                .collect::<Vec<ResultadoRedis>>(),
        );
    }
    let mut tuplas = match obetener_tupla_valor_peso(valores, pesos, bdd.clone()) {
        Some(t) => t,
        None => {
            return ResultadoRedis::Error("Error al encontrar el peso externo en sort".to_string())
        }
    };

    tuplas.sort_by(|a, b| a.1.cmp(&b.1));

    if parametros.contains(&"get".to_string()) {
        let mut split_get = parametros.rsplit(|p| p == &"get".to_string());
        let objetos = bdd.lock().unwrap().claves(&split_get.next().unwrap()[0]);
        let mut resultado: Vec<ResultadoRedis> = vec![];
        let mut pusheado = false;
        for valor in &tuplas {
            for objeto in &objetos {
                if objeto.contains(&valor.0) {
                    if let Some(TipoRedis::Str(valor)) = bdd.lock().unwrap().obtener_valor(&objeto)
                    {
                        pusheado = true;
                        resultado.push(ResultadoRedis::StrSimple(valor.to_string()))
                    }
                }
            }
            if !pusheado {
                resultado.push(ResultadoRedis::Nil);
            }
        }
        return ResultadoRedis::Vector(resultado);
    }

    let resultado = tuplas
        .iter()
        .map(|x| x.0.to_string())
        .collect::<Vec<String>>();

    sort_configuracion_lista_ordenada(parametros, resultado, bdd)
}

fn sort(comando: &mut ComandoInfo, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let parametros = match comando.get_parametros() {
        Some(p) => p,
        None => return ResultadoRedis::Error("ParametroError no hay parametros".to_string()),
    };

    let valores = match bdd.lock().unwrap().obtener_valor(&parametros[0]) {
        Some(TipoRedis::Lista(lista)) => lista.clone(),
        Some(TipoRedis::Set(set)) => Vec::from_iter(set)
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        Some(&TipoRedis::Str(_)) => return ResultadoRedis::Error("WRONGTYPE".to_string()),
        Some(TipoRedis::Canal(_)) => return ResultadoRedis::Error("WRONGTYPE".to_string()),
        None => return ResultadoRedis::Vector(vec![]),
    };

    if valores.is_empty() {
        return ResultadoRedis::Vector(vec![]);
    }

    if parametros.contains(&"by".to_string()) {
        sort_elemento_con_pesos_externos(valores, parametros, bdd)
    } else {
        sort_elemento_con_pesos_interno(valores, parametros, bdd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_de_datos::TipoRedis;
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn copy_copia_el_valor_de_una_clave_en_otra() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));

        let ptr_arc = Arc::new(Mutex::new(data_base));
        let arc_clone = Arc::clone(&ptr_arc);

        let comando = vec![
            "copy".to_string(),
            "clave".to_string(),
            "otra_clave".to_string(),
        ];
        let mut comando = ComandoInfo::new(comando);
        copy(&mut comando, ptr_arc);

        assert_eq!(
            arc_clone
                .lock()
                .unwrap()
                .obtener_valor("otra_clave")
                .unwrap(),
            &TipoRedis::Str("valor".to_string())
        );
    }

    #[test]
    fn copy_copiar_una_clave_que_no_existe_devuelve_un_error() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());
        let ptr_arc = Arc::new(Mutex::new(data_base));

        let comando = vec![
            "copy".to_string(),
            "clave".to_string(),
            "otra_clave".to_string(),
        ];
        let mut comando_info = ComandoInfo::new(comando);
        assert_eq!(copy(&mut comando_info, ptr_arc), ResultadoRedis::Int(0));
    }

    #[test]
    fn del_elimina_las_claves_guardadas_en_la_base_de_datos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("1".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(), TipoRedis::Lista(Vec::new()));
        data_base.guardar_valor("4".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(), TipoRedis::Str("valor".to_string()));

        let comando = vec![
            "del".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "8".to_string(),
        ];
        let mut comando_info = ComandoInfo::new(comando);
        assert_eq!(
            ResultadoRedis::Int(3),
            del(&mut comando_info, Arc::new(Mutex::new(data_base)))
        );
    }

    #[test]
    fn del_trata_de_elimina_las_claves_que_no_estan_guardadas_en_la_base_de_datos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("1".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(), TipoRedis::Lista(Vec::new()));
        data_base.guardar_valor("4".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(), TipoRedis::Str("valor".to_string()));

        let comando = vec![
            "del".to_string(),
            "6".to_string(),
            "7".to_string(),
            "8".to_string(),
            "8".to_string(),
        ];
        let mut comando_info = ComandoInfo::new(comando);
        assert_eq!(
            ResultadoRedis::Int(0),
            del(&mut comando_info, Arc::new(Mutex::new(data_base)))
        );
    }

    #[test]
    fn del_elimina_las_claves_repetidas_que_de_la_base_de_datos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("1".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(), TipoRedis::Lista(Vec::new()));
        data_base.guardar_valor("4".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(), TipoRedis::Str("valor".to_string()));

        let comando = vec![
            "del".to_string(),
            "1".to_string(),
            "1".to_string(),
            "3".to_string(),
            "4".to_string(),
        ];
        let mut comando_info = ComandoInfo::new(comando);
        assert_eq!(
            ResultadoRedis::Int(3),
            del(&mut comando_info, Arc::new(Mutex::new(data_base)))
        );
    }

    #[test]
    fn existis_chequea_las_claves_guardadas_en_la_base_de_datos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("1".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(), TipoRedis::Lista(Vec::new()));
        data_base.guardar_valor("4".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(), TipoRedis::Str("valor".to_string()));

        let comando = vec![
            "del".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "8".to_string(),
        ];
        let mut comando_info = ComandoInfo::new(comando);
        assert_eq!(
            ResultadoRedis::Int(3),
            exists(&mut comando_info, Arc::new(Mutex::new(data_base)))
        );
    }

    #[test]
    fn existis_chequea_las_claves_repetidas_que_de_la_base_de_datos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("1".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(), TipoRedis::Lista(Vec::new()));
        data_base.guardar_valor("4".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(), TipoRedis::Str("valor".to_string()));

        let comando = vec![
            "del".to_string(),
            "1".to_string(),
            "1".to_string(),
            "3".to_string(),
            "4".to_string(),
        ];
        let mut comando_info = ComandoInfo::new(comando);
        assert_eq!(
            ResultadoRedis::Int(3),
            del(&mut comando_info, Arc::new(Mutex::new(data_base)))
        );
    }

    #[test]
    fn rename_cambia_modifica_la_clave_pedida() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));

        let ptr_arc = Arc::new(Mutex::new(data_base));
        let arc_clone = Arc::clone(&ptr_arc);

        let comando = vec![
            "rename".to_string(),
            "clave".to_string(),
            "otra_clave".to_string(),
        ];
        let mut comando_info = ComandoInfo::new(comando);
        rename(&mut comando_info, ptr_arc);

        assert!(arc_clone.lock().unwrap().existe_clave("otra_clave"));
        assert_eq!(
            arc_clone
                .lock()
                .unwrap()
                .obtener_valor("otra_clave")
                .unwrap(),
            &TipoRedis::Str("valor".to_string())
        );
        assert!(!arc_clone.lock().unwrap().existe_clave("clave"));
    }

    #[test]
    fn tipo_devuelve_el_tipo_del_valor_almacenado_con_esa_clave() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("string".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("lista".to_string(), TipoRedis::Lista(Vec::new()));
        data_base.guardar_valor("set".to_string(), TipoRedis::Set(HashSet::new()));

        let ptr1 = Arc::new(Mutex::new(data_base));
        let ptr2 = Arc::clone(&ptr1);
        let ptr3 = Arc::clone(&ptr1);
        let ptr4 = Arc::clone(&ptr1);

        let vec1 = vec!["type".to_string(), "string".to_string()];
        let vec2 = vec!["type".to_string(), "lista".to_string()];
        let vec3 = vec!["type".to_string(), "set".to_string()];
        let vec4 = vec!["type".to_string(), "clave".to_string()];

        let mut comando_info1 = ComandoInfo::new(vec1);
        let mut comando_info2 = ComandoInfo::new(vec2);
        let mut comando_info3 = ComandoInfo::new(vec3);
        let mut comando_info4 = ComandoInfo::new(vec4);

        assert_eq!(
            tipo(&mut comando_info1, ptr1),
            ResultadoRedis::BulkStr("string".to_string())
        );
        assert_eq!(
            tipo(&mut comando_info2, ptr2),
            ResultadoRedis::BulkStr("lista".to_string())
        );
        assert_eq!(
            tipo(&mut comando_info3, ptr3),
            ResultadoRedis::BulkStr("set".to_string())
        );
        assert_eq!(
            tipo(&mut comando_info4, ptr4),
            ResultadoRedis::BulkStr("none".to_string())
        );
    }

    #[test]
    fn expire_cuando_se_crea_una_clave_no_expirable_y_se_la_pasa_a_volatil_esta_expira_correctamente(
    ) {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));
        let ptr = Arc::new(Mutex::new(data_base));

        let mut comando = ComandoInfo::new(vec![
            "expire".to_string(),
            "clave".to_string(),
            "1".to_string(),
        ]);

        assert_eq!(
            ResultadoRedis::Int(1),
            expire(&mut comando, Arc::clone(&ptr))
        );

        thread::sleep(Duration::from_secs(2));

        assert!(!ptr.lock().unwrap().existe_clave("clave"));
    }

    #[test]
    fn keys_si_se_ingresa_la_siguiente_re_el_resultado_es_el_correcto() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("hello".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("hallo".to_string(), TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("hillo".to_string(), TipoRedis::Str("valor".to_string()));

        let mut comando = ComandoInfo::new(vec!["keys".to_string(), "h[ae]llo".to_string()]);

        let valor = match keys(&mut comando, Arc::new(Mutex::new(data_base))) {
            ResultadoRedis::Vector(v) => v,
            _ => Vec::new(),
        };
        assert!(valor.contains(&ResultadoRedis::BulkStr("hallo".to_string())));
        assert!(valor.contains(&ResultadoRedis::BulkStr("hello".to_string())));
    }

    #[test]
    fn sort_ordena_los_elementos_numericos_en_una_lista() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "5".to_string(),
                "3".to_string(),
                "4".to_string(),
                "2".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec!["sort".to_string(), "mylist".to_string()]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("2".to_string()),
                ResultadoRedis::StrSimple("3".to_string()),
                ResultadoRedis::StrSimple("4".to_string()),
                ResultadoRedis::StrSimple("5".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_alphanumerico_en_una_lista() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "5".to_string(),
                "3".to_string(),
                "a".to_string(),
                "4".to_string(),
                "2".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "alpha".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("2".to_string()),
                ResultadoRedis::StrSimple("3".to_string()),
                ResultadoRedis::StrSimple("4".to_string()),
                ResultadoRedis::StrSimple("5".to_string()),
                ResultadoRedis::StrSimple("a".to_string())
            ])
        );
    }

    #[test]
    fn sort_devuelve_un_error_si_no_esta_el_parametro_alpha_en_una_lista_de_palabras() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "a".to_string(),
                "c".to_string(),
                "d".to_string(),
                "z".to_string(),
                "b".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec!["sort".to_string(), "mylist".to_string()]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Error(
                "ERR One or more scores can't be converted into double".to_string()
            )
        );
    }

    #[test]
    fn sort_ordena_los_elementos_si_esta_el_parametro_alpha_en_una_lista_de_palabras() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "a".to_string(),
                "c".to_string(),
                "d".to_string(),
                "z".to_string(),
                "b".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "alpha".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("a".to_string()),
                ResultadoRedis::StrSimple("b".to_string()),
                ResultadoRedis::StrSimple("c".to_string()),
                ResultadoRedis::StrSimple("d".to_string()),
                ResultadoRedis::StrSimple("z".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_en_orden_descendiente_en_una_lista_de_numeros_si_esta_la_clave_desc(
    ) {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "5".to_string(),
                "3".to_string(),
                "4".to_string(),
                "2".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "desc".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("5".to_string()),
                ResultadoRedis::StrSimple("4".to_string()),
                ResultadoRedis::StrSimple("3".to_string()),
                ResultadoRedis::StrSimple("2".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_en_una_lista_de_numeros_con_el_rango_pedido() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "5".to_string(),
                "3".to_string(),
                "4".to_string(),
                "2".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "limit".to_string(),
            "0".to_string(),
            "2".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("2".to_string()),
                ResultadoRedis::StrSimple("3".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_en_una_lista_de_numeros_con_otro_rango() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "5".to_string(),
                "3".to_string(),
                "4".to_string(),
                "2".to_string(),
                "6".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "limit".to_string(),
            "-2".to_string(),
            "8".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("2".to_string()),
                ResultadoRedis::StrSimple("3".to_string()),
                ResultadoRedis::StrSimple("4".to_string()),
                ResultadoRedis::StrSimple("5".to_string()),
                ResultadoRedis::StrSimple("6".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_en_una_lista_de_numeros_con_rango_uno() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "5".to_string(),
                "3".to_string(),
                "4".to_string(),
                "2".to_string(),
                "6".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "limit".to_string(),
            "2".to_string(),
            "2".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("4".to_string()),
                ResultadoRedis::StrSimple("5".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_en_una_lista_de_numeros_con_rango_menos_uno_menos_diez() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "5".to_string(),
                "3".to_string(),
                "4".to_string(),
                "2".to_string(),
                "6".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "limit".to_string(),
            "-1".to_string(),
            "-10".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("2".to_string()),
                ResultadoRedis::StrSimple("3".to_string()),
                ResultadoRedis::StrSimple("4".to_string()),
                ResultadoRedis::StrSimple("5".to_string()),
                ResultadoRedis::StrSimple("6".to_string())
            ])
        );
    }

    #[test]
    fn sort_devuelve_una_lista_vacia_si_el_valor_no_existes() {
        let data_base = BaseDeDatos::new("eliminame.txt".to_string());
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "limit".to_string(),
            "0".to_string(),
            "2".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(valor, ResultadoRedis::Vector(vec![]));
    }

    #[test]
    fn sort_ordena_los_elementos_en_una_lista_con_pesos_externos() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("peso_2".to_string(), TipoRedis::Str("2".to_string()));
        data_base.guardar_valor("peso_3".to_string(), TipoRedis::Str("3".to_string()));
        data_base.guardar_valor("peso_4".to_string(), TipoRedis::Str("4".to_string()));
        data_base.guardar_valor("peso_5".to_string(), TipoRedis::Str("5".to_string()));
        data_base.guardar_valor("peso_6".to_string(), TipoRedis::Str("6".to_string()));
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "objeto_5".to_string(),
                "objeto_3".to_string(),
                "objeto_4".to_string(),
                "objeto_2".to_string(),
                "objeto_6".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "by".to_string(),
            "peso_*".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("objeto_2".to_string()),
                ResultadoRedis::StrSimple("objeto_3".to_string()),
                ResultadoRedis::StrSimple("objeto_4".to_string()),
                ResultadoRedis::StrSimple("objeto_5".to_string()),
                ResultadoRedis::StrSimple("objeto_6".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_en_una_lista_con_pesos_externos_faltantes() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("peso_3".to_string(), TipoRedis::Str("3".to_string()));
        data_base.guardar_valor("peso_4".to_string(), TipoRedis::Str("4".to_string()));
        data_base.guardar_valor("peso_5".to_string(), TipoRedis::Str("5".to_string()));
        data_base.guardar_valor("peso_6".to_string(), TipoRedis::Str("6".to_string()));
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "objeto_5".to_string(),
                "objeto_3".to_string(),
                "objeto_4".to_string(),
                "objeto_2".to_string(),
                "objeto_6".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "by".to_string(),
            "peso_*".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("objeto_5".to_string()),
                ResultadoRedis::StrSimple("objeto_3".to_string()),
                ResultadoRedis::StrSimple("objeto_4".to_string()),
                ResultadoRedis::StrSimple("objeto_2".to_string()),
                ResultadoRedis::StrSimple("objeto_6".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_en_una_lista_con_todos_pesos_externos_faltantes() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "objeto_5".to_string(),
                "objeto_3".to_string(),
                "objeto_4".to_string(),
                "objeto_2".to_string(),
                "objeto_6".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "by".to_string(),
            "peso_*".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("objeto_5".to_string()),
                ResultadoRedis::StrSimple("objeto_3".to_string()),
                ResultadoRedis::StrSimple("objeto_4".to_string()),
                ResultadoRedis::StrSimple("objeto_2".to_string()),
                ResultadoRedis::StrSimple("objeto_6".to_string()),
            ])
        );
    }

    #[test]
    fn sort_ordena_los_elementos_y_devuelve_los_objetos_que_representan_las_ids() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("peso_1".to_string(), TipoRedis::Str("1".to_string()));
        data_base.guardar_valor("peso_2".to_string(), TipoRedis::Str("2".to_string()));
        data_base.guardar_valor("peso_3".to_string(), TipoRedis::Str("3".to_string()));
        data_base.guardar_valor("peso_4".to_string(), TipoRedis::Str("4".to_string()));
        data_base.guardar_valor("peso_5".to_string(), TipoRedis::Str("5".to_string()));

        data_base.guardar_valor(
            "objeto_1".to_string(),
            TipoRedis::Str("primero".to_string()),
        );
        data_base.guardar_valor(
            "objeto_2".to_string(),
            TipoRedis::Str("segundo".to_string()),
        );
        data_base.guardar_valor(
            "objeto_3".to_string(),
            TipoRedis::Str("tercero".to_string()),
        );
        data_base.guardar_valor("objeto_4".to_string(), TipoRedis::Str("cuarto".to_string()));
        data_base.guardar_valor("objeto_5".to_string(), TipoRedis::Str("quinto".to_string()));

        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "objeto_2".to_string(),
                "objeto_5".to_string(),
                "objeto_4".to_string(),
                "objeto_3".to_string(),
                "objeto_1".to_string(),
            ]),
        );
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "by".to_string(),
            "peso_*".to_string(),
            "get".to_string(),
            "objeto_*".to_string(),
        ]);
        let valor = sort(&mut comando, Arc::new(Mutex::new(data_base)));
        assert_eq!(
            valor,
            ResultadoRedis::Vector(vec![
                ResultadoRedis::StrSimple("primero".to_string()),
                ResultadoRedis::StrSimple("segundo".to_string()),
                ResultadoRedis::StrSimple("tercero".to_string()),
                ResultadoRedis::StrSimple("cuarto".to_string()),
                ResultadoRedis::StrSimple("quinto".to_string())
            ])
        );
    }

    #[test]
    fn sort_ordena_por_pesos_externos_los_elementos_los_guarda_con_la_clave_dada() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("peso_1".to_string(), TipoRedis::Str("1".to_string()));
        data_base.guardar_valor("peso_2".to_string(), TipoRedis::Str("2".to_string()));
        data_base.guardar_valor("peso_3".to_string(), TipoRedis::Str("3".to_string()));
        data_base.guardar_valor("peso_4".to_string(), TipoRedis::Str("4".to_string()));
        data_base.guardar_valor("peso_5".to_string(), TipoRedis::Str("5".to_string()));

        data_base.guardar_valor(
            "mylist".to_string(),
            TipoRedis::Lista(vec![
                "objeto_2".to_string(),
                "objeto_5".to_string(),
                "objeto_4".to_string(),
                "objeto_3".to_string(),
                "objeto_1".to_string(),
            ]),
        );
        let ptr = Arc::new(Mutex::new(data_base));
        let ptr_clone = Arc::clone(&ptr);
        let mut comando = ComandoInfo::new(vec![
            "sort".to_string(),
            "mylist".to_string(),
            "by".to_string(),
            "peso_*".to_string(),
            "store".to_string(),
            "ordenados".to_string(),
        ]);
        assert_eq!(
            ResultadoRedis::StrSimple("5".to_string()),
            sort(&mut comando, ptr)
        );
        assert_eq!(
            Some(&TipoRedis::Lista(vec![
                "objeto_1".to_string(),
                "objeto_2".to_string(),
                "objeto_3".to_string(),
                "objeto_4".to_string(),
                "objeto_5".to_string()
            ])),
            ptr_clone.lock().unwrap().obtener_valor("ordenados")
        );
    }
}
