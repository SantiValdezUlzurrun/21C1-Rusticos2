use std::collections::LinkedList;
use std::collections::HashMap;

static SEPARADOR: &str = "\r\n";
static FORMATO_GET: &str = "*3\r\n$3\r\nSET\r\n";
static FORMATO_LPUSH: &str = "$4\r\nLPUSH\r\n";
static ID_ARG: &str = "*";
static ID_TAM_STR: &str = "$";

pub enum TipoRedis {
    Str(String),
    List(LinkedList<String>)
}

fn guardar_elemento(elemento: &str) -> String{

    let len_elemento = elemento.len();
    ID_TAM_STR.to_string()+&len_elemento.to_string()+SEPARADOR+elemento+SEPARADOR
}

fn guardar_clave_valor(clave: String,  valor:&TipoRedis) -> String{

    match valor {

        TipoRedis::Str(valor) => FORMATO_GET.to_string()+ &guardar_elemento(&clave) + &guardar_elemento(&valor),
    
        
        TipoRedis::List(lista) => {
            
            let cant_arg = lista.len() + 2;
            let string_cant_arg = ID_ARG.to_string()+&cant_arg.to_string()+SEPARADOR;

            let mut string_comando = string_cant_arg+FORMATO_LPUSH+ &guardar_elemento(&clave);
            
            for valor in lista.iter() {
                string_comando += &guardar_elemento(valor);
            }
            string_comando
        }
    }
    
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserto_varios_strings_en_hash_map_y_guardar_clave_valor_devuelve_el_mensaje_para_volver_a_cargarlos(){
        let mut map = HashMap::new();
        map.insert("UnaClave1", TipoRedis::Str("UnValor".to_string()));
        map.insert("UnaClave2", TipoRedis::Str("UnValor".to_string()));
        map.insert("UnaClave3", TipoRedis::Str("UnValor".to_string()));

        let mut vector : Vec<String>= vec!();
        for (key, val) in map.iter() {
        vector.push(guardar_clave_valor(key.to_string(),val));
        }

        assert!(vector.contains(&String::from("*3\r\n$3\r\nSET\r\n$9\r\nUnaClave1\r\n$7\r\nUnValor\r\n")));
        assert!(vector.contains(&String::from("*3\r\n$3\r\nSET\r\n$9\r\nUnaClave2\r\n$7\r\nUnValor\r\n")));
        assert!(vector.contains(&String::from("*3\r\n$3\r\nSET\r\n$9\r\nUnaClave3\r\n$7\r\nUnValor\r\n")));
    }


    #[test]
    fn inserto_varios_tipo_redis_en_hash_map_y_guardar_clave_valor_devuelve_el_mensaje_para_volver_a_cargarlos(){
        let mut map = HashMap::new();
        map.insert("UnaClave1", TipoRedis::Str("UnValor".to_string()));
        map.insert("UnaClave2", TipoRedis::Str("UnValor".to_string()));

        let mut lista = TipoRedis::List(LinkedList::new());

        match lista {

            TipoRedis::List(ref mut lista) => { 
                lista.push_back("PRIMER_VALOR".to_string());
                lista.push_back("SEGUNDO_VALOR".to_string());
                lista.push_back("TERCER_VALOR".to_string());
            }

            TipoRedis::Str(_) =>{}
        }

        map.insert("milista", lista);

        let mut vector : Vec<String>= vec!();
        for (key, val) in map.iter() {
        vector.push(guardar_clave_valor(key.to_string(),val));
        }

        assert!(vector.contains(&String::from("*3\r\n$3\r\nSET\r\n$9\r\nUnaClave1\r\n$7\r\nUnValor\r\n")));
        assert!(vector.contains(&String::from("*3\r\n$3\r\nSET\r\n$9\r\nUnaClave2\r\n$7\r\nUnValor\r\n")));
        assert!(vector.contains(&String::from("*5\r\n$4\r\nLPUSH\r\n$7\r\nmilista\r\n$12\r\nPRIMER_VALOR\r\n$13\r\nSEGUNDO_VALOR\r\n$12\r\nTERCER_VALOR\r\n")));
    }
}