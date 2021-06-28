use std::option::Option;

#[derive(Debug)]
pub struct ComandoInfo {
	nombre: String,
	clave: Option<String>,
	parametros: Vec<String>,
	index: usize,
}

impl ComandoInfo {
	
	pub fn new(comando_parseado: Vec<String>) -> Self{
		if comando_parseado.len() == 0 {
			return ComandoInfo{
				nombre: comando_parseado[0],
				clave: None,
				parametros: vec![],
				index: 0
			};
		}
		if comando_parseado.len() == 1 {
			return ComandoInfo{
				nombre: comando_parseado[0],
				clave: Some(comando_parseado[1]),
				parametros: vec![],
				index: 0
			};
		}

		let nombre = comando_parseado[0];
		let clave = comando_parseado[1];
		comando_parseado.remove(0);
		comando_parseado.remove(0);

		ComandoInfo{
			nombre: comando_parseado[0],
			clave: Some(comando_parseado[1]),
			parametros : comando_parseado,
			index: 0
		}
	}


	pub fn get_nombre(&self) -> &str{
		&self.nombre

	}
	pub fn get_clave(&self) -> Option<&str>{
		match self.clave{
			Some(clave) => Some(&clave),
			None => None,
		}
	}

	pub fn get_parametro(&self) -> Option<&str>{
		if self.index < self.parametros.len() {
			return Some(&self.parametros[self.index]);
		};
		None
	}

	pub fn descripcion(&self) -> String{
		let descripcion = self.nombre + " ";

		match self.clave{
			Some(clave) => {
				descripcion += &clave;

				if 0 < self.parametros.len() {
					let i = 0;
					while i < self.parametros.len(){
						descripcion += " ";
						descripcion += &self.parametros[i];
						i+=1;
					}
				}
				descripcion
			},
			None => descripcion,
		}
	}
}

#[cfg(test)]
mod tests {
	

	use crate::comando_info::ComandoInfo;

	fn comando_info_devuelve_correctamente_todos_los_parametros(){
		let parametros = vec!["GET".to_string(),"clave".to_string(),"arg1".to_string(),"arg2".to_string(),"arg3".to_string(),"arg4".to_string()];

		let comando_info = ComandoInfo::new(parametros);

		assert_eq!(Some("clave"),comando_info.get_clave());

		assert_eq!(Some("arg1"),comando_info.get_parametro());
		assert_eq!(Some("arg2"),comando_info.get_parametro());
		assert_eq!(Some("arg3"),comando_info.get_parametro());
		assert_eq!(Some("arg4"),comando_info.get_parametro());
		assert_eq!(None,comando_info.get_parametro());
	}

	fn comando_info_devuelve_una_descripcion_en_forma_de_string_sobre_el_comando(){
		let parametros = vec!["GET".to_string(),"clave".to_string(),"arg1".to_string(),"arg2".to_string(),"arg3".to_string(),"arg4".to_string()];
		let comando_info = ComandoInfo::new(parametros);

		assert_eq!("GET clave arg1 arg2 arg3 ag4",comando_info.descripcion());

	}
}


