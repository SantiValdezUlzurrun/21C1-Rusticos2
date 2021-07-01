use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{Comando, ComandoHandler};
use crate::comando_info::ComandoInfo;
use std::sync::{Arc, Mutex};

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

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_de_datos::TipoRedis;
    use std::collections::HashSet;

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
}
