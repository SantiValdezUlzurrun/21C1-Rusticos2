use std::option::Option;

#[derive(Debug, Clone)]
pub struct ComandoInfo {
    nombre: String,
    parametros: Vec<String>,
    index: usize,
}

impl ComandoInfo {
    pub fn new(mut comando_parseado: Vec<String>) -> Self {
        if comando_parseado.len() == 1 {
            return ComandoInfo {
                nombre: comando_parseado[0].clone(),
                parametros: vec![],
                index: 0,
            };
        }

        let nombre = comando_parseado[0].clone();
        comando_parseado.remove(0);

        ComandoInfo {
            nombre,
            parametros: comando_parseado,
            index: 0,
        }
    }

    pub fn get_nombre(&self) -> String {
        self.nombre.clone()
    }
    pub fn get_clave(&mut self) -> Option<String> {
        self.index = 1;
        self.parametros
            .get(0)
            .as_ref()
            .map(|clave| clave.to_string())
    }

    pub fn get_parametros(&self) -> Option<Vec<String>> {
        if !self.parametros.is_empty() {
            return Some(self.parametros.clone());
        }
        None
    }

    pub fn get_parametro(&mut self) -> Option<String> {
        if self.index < self.parametros.len() {
            let a_devolver = &self.parametros[self.index];
            self.index += 1;
            return Some(a_devolver.to_string());
        };
        None
    }

    pub fn descripcion(&self) -> String {
        let mut descripcion = self.nombre.clone();

        for param in self.parametros.iter() {
            descripcion += " ";
            descripcion += &param;
        }
        descripcion
    }
}

#[cfg(test)]
mod tests {

    use crate::comando_info::ComandoInfo;

    #[test]
    fn comando_info_devuelve_correctamente_todos_los_parametros() {
        let parametros = vec![
            "GET".to_string(),
            "clave".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
            "arg3".to_string(),
            "arg4".to_string(),
        ];

        let mut comando_info = ComandoInfo::new(parametros);

        assert_eq!(Some("clave".to_string()), comando_info.get_clave());

        assert_eq!(Some("arg1".to_string()), comando_info.get_parametro());
        assert_eq!(Some("arg2".to_string()), comando_info.get_parametro());
        assert_eq!(Some("arg3".to_string()), comando_info.get_parametro());
        assert_eq!(Some("arg4".to_string()), comando_info.get_parametro());
        assert_eq!(None, comando_info.get_parametro());
    }

    #[test]
    fn comando_info_devuelve_una_descripcion_en_forma_de_string_sobre_el_comando() {
        let parametros = vec![
            "GET".to_string(),
            "clave".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
            "arg3".to_string(),
            "arg4".to_string(),
        ];
        let comando_info = ComandoInfo::new(parametros);

        assert_eq!(
            "GET clave arg1 arg2 arg3 arg4".to_string(),
            comando_info.descripcion()
        );
    }

    #[test]
    fn comando_info_devuelve_una_lista_con_todos_los_parametros() {
        let parametros = vec![
            "MGET".to_string(),
            "clave1".to_string(),
            "clave2".to_string(),
            "clave3".to_string(),
            "clave4".to_string(),
            "clave5".to_string(),
        ];
        let comando_info = ComandoInfo::new(parametros);

        assert_eq!(
            Some(vec![
                "clave1".to_string(),
                "clave2".to_string(),
                "clave3".to_string(),
                "clave4".to_string(),
                "clave5".to_string(),
            ]),
            comando_info.get_parametros()
        );
    }
}
