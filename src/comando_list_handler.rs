/*
use crate::comando_info::ComandoInfo;
use crate::base_de_datos::{BaseDeDatos,ResultadoRedis,TipoRedis};
use std::sync::{Arc, Mutex};


fn lindex(comando_info: ComandoInfo, base_de_datos: Arc<Mutex<BaseDeDatos>>) -> ResultadoRedis {
	let lista = match base_de_datos.lock().unwrap().obtener_valor(&comando_info.get_clave()){
		Ok(TipoRedis::Lista(lista)) => lista,
		_ => return ResultadoRedis::Error("WRONGTYPE".to_string())
	};

	let indice : i32 = comando_info.parametros.parse().unwrap();
	let tamanio = lista.len() as i32;
	
	if 0 < indice && indice < tamanio {
		ResultadoRedis::BulkStr(lista[indice as usize].clone())

	} else if 0 > indice && tamanio + indice >= 0 {
		ResultadoRedis::BulkStr(lista[(tamanio + indice) as usize].clone())

	} else{
		ResultadoRedis::BulkStr("nil".to_string())
	}
}

#[cfg(test)]
mod tests {
    use crate::comando_info::ComandoInfo;
	use super::*;

    #[test]
    fn lindex_busca_un_indice_positivo_en_una_clave_valor_de_la_base_de_datos(){

    	let mut data_base = BaseDeDatos::new();
    	data_base.guardar_valor("clave".to_string(),TipoRedis::Lista(vec!["1".to_string(),"2".to_string(),"3".to_string()]));
    	let ptr = Arc::new(Mutex::new(data_base));

    	let comando = vec!["lindex".to_string(),"clave".to_string(),"1".to_string()];


    	let comando_info = ComandoInfo::new("lindex".to_string(),"clave".to_string(),vec!["1".to_string()]);

    	assert_eq!(ResultadoRedis::BulkStr("2".to_string()),lindex(comando_info,ptr));
    }


    #[test]
    fn lindex_busca_un_indice_negativo_en_una_clave_valor_de_la_base_de_datos(){

    	let mut data_base = BaseDeDatos::new();
    	data_base.guardar_valor("clave".to_string(),TipoRedis::Lista(vec!["1".to_string(),"2".to_string(),"3".to_string()]));
    	let ptr = Arc::new(Mutex::new(data_base));

    	let comando = vec!["lindex".to_string(),"clave".to_string(),"-2".to_string()];

    	assert_eq!(ResultadoRedis::BulkStr("2".to_string()),lindex(&comando,ptr));
    }


    #[test]
    fn lindex_busca_un_indice_fuera_de_rango_en_una_clave_valor_de_la_base_de_datos(){

    	let mut data_base = BaseDeDatos::new();
    	data_base.guardar_valor("clave".to_string(),TipoRedis::Lista(vec!["1".to_string(),"2".to_string(),"3".to_string()]));
    	let ptr = Arc::new(Mutex::new(data_base));

    	let comando = vec!["lindex".to_string(),"clave".to_string(),"65".to_string()];

    	assert_eq!(ResultadoRedis::BulkStr("nil".to_string()),lindex(&comando,ptr));
    }
}
*/