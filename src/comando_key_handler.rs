use crate::comando_info::ComandoInfo;
use crate::base_de_datos::{BaseDeDatos, ResultadoRedis, TipoRedis};
use crate::comando::{Comando, ComandoHandler};
use std::sync::{Arc, Mutex};

pub struct ComandoKeyHandler {
    comando: ComandoInfo,
    a_ejecutar: Comando,
}

impl ComandoKeyHandler {
    pub fn new(comando: ComandoInfo) -> Self {
        let a_ejecutar = match comando.get_nombre() {
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
    fn ejecutar(self: Box<Self>, bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
        (self.a_ejecutar)(&self.comando, bdd)
    }
}

#[allow(dead_code)]
pub fn es_comando_key(comando: &str) -> bool {
    let comandos = vec!["COPY", "DEL", "EXISTS", "RENAME", "TYPE"];
    comandos.iter().any(|&c| c == comando)
}

fn copy(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    match bdd.lock().unwrap().copiar_valor(&comando[1], &comando[2]) {
        Some(_) => ResultadoRedis::Int(1),
        None => ResultadoRedis::Int(0),
    }
}

fn rename(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clon = Arc::clone(&bdd);
    match copy(comando, bdd) {
        ResultadoRedis::Error(_) => {
            ResultadoRedis::Error("ErrorRename clave no encontrada".to_string())
        }
        _ => {
            let vector = vec!["rename".to_string(), comando[1].clone()];
            del(&vector, clon);
            ResultadoRedis::StrSimple("Ok".to_string())
        }
    }
}

fn tipo(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    match bdd.lock().unwrap().obtener_valor(&comando[1]) {
        Some(TipoRedis::Str(_)) => ResultadoRedis::BulkStr("string".to_string()),
        Some(TipoRedis::Lista(_)) => ResultadoRedis::BulkStr("lista".to_string()),
        Some(TipoRedis::Set(_)) => ResultadoRedis::BulkStr("set".to_string()),
        _ => ResultadoRedis::BulkStr("none".to_string()),
    }
}

fn recorrer_y_ejecutar(
    comando: &[String],
    base_de_datos: Arc<Mutex<BaseDeDatos>>,
    funcion: Box<dyn Fn(&str)>,
) -> ResultadoRedis {
    let mut claves_eliminadas = 0;
    let mut iterador = comando.iter();
    iterador.next();

    for clave in iterador {
        if base_de_datos.lock().unwrap().existe_clave(clave) {
            (funcion)(clave);
            claves_eliminadas += 1;
        }
    }
    ResultadoRedis::Int(claves_eliminadas)
}

fn del(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    let clon = Arc::clone(&bdd);
    recorrer_y_ejecutar(
        comando,
        bdd,
        Box::new(move |clave| {
            clon.lock().unwrap().eliminar_clave(clave);
        }),
    )
}

fn exists(comando: &[String], bdd: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
    recorrer_y_ejecutar(comando, bdd, Box::new(|_| {}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_de_datos::TipoRedis;
    use std::collections::HashSet;
    use std::collections::LinkedList;


    #[test]
    fn copy_copia_el_valor_de_una_clave_en_otra() {
        let mut data_base = BaseDeDatos::new("eliminame".to_string());
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));

        let ptr_arc = Arc::new(Mutex::new(data_base));
        let arc_clone = Arc::clone(&ptr_arc);

        let comando = vec![
            "copy".to_string(),
            "clave".to_string(),
            "otra_clave".to_string(),
        ];
        copy(&comando, ptr_arc);

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
        let data_base = BaseDeDatos::new("eliminame".to_string());
        let ptr_arc = Arc::new(Mutex::new(data_base));

        let comando = vec![
            "copy".to_string(),
            "clave".to_string(),
            "otra_clave".to_string(),
        ];
        assert_eq!(copy(&comando, ptr_arc), ResultadoRedis::Int(0));
    }

    #[test]
    fn del_elimina_las_claves_guardadas_en_la_base_de_datos(){
        let mut data_base = BaseDeDatos::new();    
        data_base.guardar_valor("1".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(),TipoRedis::Lista(vec![]));    
        data_base.guardar_valor("4".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(),TipoRedis::Str("valor".to_string()));
    
        let comando = vec!["del".to_string(),"1".to_string(),"2".to_string(),"3".to_string(),"8".to_string()];
        assert_eq!(ResultadoRedis::Int(3),del(&comando, Arc::new(Mutex::new(data_base))));
    }

    #[test]
    fn del_trata_de_elimina_las_claves_que_no_estan_guardadas_en_la_base_de_datos(){
        let mut data_base = BaseDeDatos::new();    
        data_base.guardar_valor("1".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(),TipoRedis::Lista(vec![]));    
        data_base.guardar_valor("4".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(),TipoRedis::Str("valor".to_string()));
    
        let comando = vec!["del".to_string(),"6".to_string(),"7".to_string(),"8".to_string(),"8".to_string()];
        assert_eq!(ResultadoRedis::Int(0),del(&comando, Arc::new(Mutex::new(data_base))));
    }

    #[test]
    fn del_elimina_las_claves_repetidas_que_de_la_base_de_datos(){
        let mut data_base = BaseDeDatos::new();    
        data_base.guardar_valor("1".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(),TipoRedis::Lista(vec![]));    
        data_base.guardar_valor("4".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(),TipoRedis::Str("valor".to_string()));
    
        let comando = vec!["del".to_string(),"1".to_string(),"1".to_string(),"3".to_string(),"4".to_string()];
        assert_eq!(ResultadoRedis::Int(3),del(&comando, Arc::new(Mutex::new(data_base))));
    }

    #[test]
    fn existis_chequea_las_claves_guardadas_en_la_base_de_datos(){
        let mut data_base = BaseDeDatos::new();    
        data_base.guardar_valor("1".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(),TipoRedis::Lista(vec![]));    
        data_base.guardar_valor("4".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(),TipoRedis::Str("valor".to_string()));
    
        let comando = vec!["del".to_string(),"1".to_string(),"2".to_string(),"3".to_string(),"8".to_string()];
        assert_eq!(ResultadoRedis::Int(3),exists(&comando, Arc::new(Mutex::new(data_base))));
    } 

    #[test]
    fn existis_chequea_las_claves_repetidas_que_de_la_base_de_datos(){
        let mut data_base = BaseDeDatos::new();    
        data_base.guardar_valor("1".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("2".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("3".to_string(),TipoRedis::Lista(vec![]));    
        data_base.guardar_valor("4".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("5".to_string(),TipoRedis::Str("valor".to_string()));
    
        let comando = vec!["del".to_string(),"1".to_string(),"1".to_string(),"3".to_string(),"4".to_string()];
        assert_eq!(ResultadoRedis::Int(3),del(&comando, Arc::new(Mutex::new(data_base))));
    }   

    #[test]
    fn rename_cambia_modifica_la_clave_pedida() {
        let mut data_base = BaseDeDatos::new("eliminame".to_string());
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));

        let ptr_arc = Arc::new(Mutex::new(data_base));
        let arc_clone = Arc::clone(&ptr_arc);

        let comando = vec![
            "rename".to_string(),
            "clave".to_string(),
            "otra_clave".to_string(),
        ];
        rename(&comando, ptr_arc);

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
    fn tipo_devuelve_el_tipo_del_valor_almacenado_con_esa_clave(){
        let mut data_base = BaseDeDatos::new();
        data_base.guardar_valor("string".to_string(),TipoRedis::Str("valor".to_string()));
        data_base.guardar_valor("lista".to_string(),TipoRedis::Lista(vec![]));
        data_base.guardar_valor("set".to_string(),TipoRedis::Set(HashSet::new()));

        let ptr1 = Arc::new(Mutex::new(data_base));
        let ptr2 = Arc::clone(&ptr1);
        let ptr3 = Arc::clone(&ptr1);
        let ptr4 = Arc::clone(&ptr1);

        let vec1 = vec!["type".to_string(), "string".to_string()];
        let vec2 = vec!["type".to_string(), "lista".to_string()];
        let vec3 = vec!["type".to_string(), "set".to_string()];
        let vec4 = vec!["type".to_string(), "clave".to_string()];

        assert_eq!(
            tipo(&vec1, ptr1),
            ResultadoRedis::BulkStr("string".to_string())
        );
        assert_eq!(
            tipo(&vec2, ptr2),
            ResultadoRedis::BulkStr("lista".to_string())
        );
        assert_eq!(
            tipo(&vec3, ptr3),
            ResultadoRedis::BulkStr("set".to_string())
        );
        assert_eq!(
            tipo(&vec4, ptr4),
            ResultadoRedis::BulkStr("none".to_string())
        );
    }
}
