use crate::comando::TipoRedis;
use std::collections::HashMap;

pub struct BaseDeDatos {
	hashmap : HashMap<String, TipoRedis>
}

impl BaseDeDatos{

	pub fn new()-> Self{
		BaseDeDatos{
			hashmap: HashMap::new(),
		}
	}

	pub fn obtener_valor(&mut self,clave:&str) -> Result<TipoRedis,&'static str>{
		match self.hashmap.get(clave){
			Some(TipoRedis::Str(string)) => Ok(TipoRedis::Str(string.clone())), 
			Some(TipoRedis::Lista(lista)) => Ok(TipoRedis::Lista(lista.clone())),
			Some(TipoRedis::Set(set)) => Ok(TipoRedis::Set(set.clone())),
			_ =>  Err("Error clave no encontrada"),
		}
	}

	pub fn guardar_valor(&mut self, clave:String, valor:TipoRedis){
		self.hashmap.insert(clave,valor);
	}

	pub fn existe_clave(&mut self, clave:&str) -> bool {
		self.hashmap.contains_key(clave)
	}

	pub fn cant_claves(&mut self) -> usize{
		self.hashmap.len()
	}

	pub fn eliminar_clave(&mut self, clave:&str) -> usize {
		match self.hashmap.remove(clave){
			Some(valor) => 1,
			None => 0,
		}
	}

	pub fn copiar_valor(&mut self, clave_actual: &str, clave_nueva:&str) -> Result<(), &'static str>{
		match self.obtener_valor(clave_actual){
			Err(_) => Err("Error clave no encontrada"),
			Ok(valor) => {
				Ok(self.guardar_valor(clave_nueva.to_string(),valor))
			}
		}
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_de_datos_devuelve_una_copia_de_un_elemento_almacenado(){
    	let mut data_base = BaseDeDatos::new();
    	data_base.guardar_valor("clave".to_string(),TipoRedis::Str("valor".to_string()));

    	let valor = data_base.obtener_valor("clave");
    	assert_eq!(TipoRedis::Str("valor".to_string()),valor.unwrap());
    }

    #[test]
    fn base_de_datos_elimina_valor_almacenado(){
    	let mut data_base = BaseDeDatos::new();
    	data_base.guardar_valor("clave".to_string(),TipoRedis::Str("valor".to_string()));

    	assert!(data_base.existe_clave("clave"));

    	data_base.eliminar_clave("clave");

    	assert!(!data_base.existe_clave("clave"));
    }

}