pub struct ComandoInfo {
	nombre: String,
	clave: String,
	parametros: Vec<String>,
	index: usize,
}

impl ComandoInfo {
	
	pub fn new(nombre: String, clave: String,parametros: Vec<String>) -> Self{
		ComandoInfo{
			nombre,
			clave,
			parametros,
			index: 0
		}
	}


	pub fn get_clave(&self) -> &str{
		&self.clave
	}

	pub fn get_parametro(&self) -> Option<&str>{
		if self.index < self.parametros.len() {
			return Some(&self.parametros[self.index]);
		};
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn comando_info_devuelve_correctamente_todos_los_parametros(){
		let nombre = "GET".to_string();
		let clave = "clave".to_string();
		let parametros = vec!["arg1".to_string(),"arg2".to_string(),"arg3".to_string(),"arg4".to_string()];

		let comando_info = ComandoInfo::new(nombre,clave,parametros);

		assert_eq!("GET",comando_info.get_clave());

		assert_eq!(Some("arg1"),comando_info.get_parametro());
		assert_eq!(Some("arg2"),comando_info.get_parametro());
		assert_eq!(Some("arg3"),comando_info.get_parametro());
		assert_eq!(Some("arg4"),comando_info.get_parametro());
		assert_eq!(None,comando_info.get_parametro());
	}
}


