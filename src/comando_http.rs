use crate::comando_info::ComandoInfo;

/// Representa una request del protocolo HTTP
pub struct ComandoHTTP {
    metodo: String,
    args: Vec<String>,
    _headers: Vec<String>,
    comando_redis: Option<ComandoInfo>,
}

impl ComandoHTTP {
    /// Instancia una request HTTP en condiciones de obtener sus valores
    ///
    /// # Argumentos
    ///
    /// * `metodo` - Vector de cadenas que posee el nombre del metodo, la URI y la version del protocolo
    /// * `headers` - headers de la request
    /// * `comando` - comando redis obtenido de la request
    pub fn new(mut metodo: Vec<String>, _headers: Vec<String>, comando: Vec<String>) -> Self {
        ComandoHTTP {
            metodo: metodo.remove(0),
            args: metodo,
            _headers,
            comando_redis: if comando.is_empty() {
                None
            } else {
                Some(ComandoInfo::new(comando))
            },
        }
    }

    pub fn get_metodo(&self) -> String {
        self.metodo.clone()
    }

    pub fn get_comando(&self) -> Option<ComandoInfo> {
        self.comando_redis.clone()
    }

    pub fn get_argumento(&self) -> Option<String> {
        if self.args.is_empty() {
            None
        } else {
            Some(self.args[0].to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comando_http_devuelve_correctamente_todos_los_parametros() {
        let metodo = vec!["GET".to_string(), "/".to_string(), "HTTP/1.1".to_string()];

        let comando_redis = vec![
            "GET".to_string(),
            "clave".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
            "arg3".to_string(),
            "arg4".to_string(),
        ];

        let comando_http = ComandoHTTP::new(metodo, vec![], comando_redis);

        assert_eq!("GET".to_string(), comando_http.get_metodo());
        assert_eq!(
            Some("clave".to_string()),
            comando_http.get_comando().unwrap().get_clave()
        );
    }

    #[test]
    fn comando_http_devuelve_el_argumento_correctamente() {
        let metodo = vec![
            "GET".to_string(),
            "/favicon.ico".to_string(),
            "HTTP/1.1".to_string(),
        ];

        let comando_redis = vec![
            "GET".to_string(),
            "clave".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
            "arg3".to_string(),
            "arg4".to_string(),
        ];

        let comando_http = ComandoHTTP::new(metodo, vec![], comando_redis);

        assert_eq!(
            Some("/favicon.ico".to_string()),
            comando_http.get_argumento()
        );
    }
}
