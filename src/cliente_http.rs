use crate::base_de_datos::ResultadoRedis;
use crate::cliente::{TipoCliente, Token};
use crate::comando_http::ComandoHTTP;
use crate::comando_info::ComandoInfo;
use crate::http_parser::HTTPParser;
use crate::parser::parsear_respuesta; //TODO: Cambiar por http_parsear_respuesta
use crate::redis_error::RedisError;
use std::fs::read_to_string;

use std::fmt;
use std::io::Write;
use std::net::TcpStream;

pub struct ClienteHTTP {
    id: Token,
    socket: Option<TcpStream>,
    mando: bool,
    pag_index: String,
}

impl ClienteHTTP {
    pub fn new(id: Token, socket: TcpStream) -> Self {
        let pag_index = match read_to_string("resources/test.html") {
            Ok(s) => s,
            Err(_) => "<html><body>Error al levantar la pagina</body></html>".to_string(),
        };

        ClienteHTTP {
            id,
            pag_index,
            socket: Some(socket),
            mando: false,
        }
    }

    fn manejar_get(&mut self, _comando: ComandoHTTP) -> Result<Option<ComandoInfo>, RedisError> {
        let respuesta = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}",
            self.pag_index
        );
        match self.enviar_mensaje(respuesta) {
            Ok(_) => Ok(None),
            Err(_) => Err(RedisError::ServerError),
        }
    }
    fn obtener_comando_de_post(
        &mut self,
        comando: ComandoHTTP,
    ) -> Result<Option<ComandoInfo>, RedisError> {
        match comando.get_comando() {
            Some(c) => Ok(Some(c)),
            None => Err(RedisError::ServerError),
        }
    }

    fn manejar_error(&mut self, _comando: ComandoHTTP) -> Result<Option<ComandoInfo>, RedisError> {
        let respuesta = "HTTP/1.1 400 BAD REQUEST\r\n\r\n".to_string();
        match self.enviar_mensaje(respuesta) {
            Ok(_) => Ok(None),
            Err(_) => Err(RedisError::ServerError),
        }
    }

    fn obtener_socket(&self) -> Option<TcpStream> {
        let socket = match &self.socket {
            None => return None,
            Some(t) => t,
        };

        match socket.try_clone() {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
}

impl TipoCliente for ClienteHTTP {
    fn obtener_comando(&mut self) -> Result<Option<ComandoInfo>, RedisError> {
        let stream = match self.obtener_socket() {
            Some(s) => s,
            None => return Err(RedisError::ConeccionError),
        };
        self.mando = true;
        let parser = HTTPParser::new(stream);
        let comando_http = match parser.parsear_stream() {
            Ok(orden) => orden,
            Err(_) => return Err(RedisError::ServerError),
        };

        match comando_http.get_metodo().as_str() {
            "GET" => self.manejar_get(comando_http),
            "POST" => self.obtener_comando_de_post(comando_http),
            _ => self.manejar_error(comando_http),
        }
    }

    fn obtener_addr(&self) -> String {
        let socket = match &self.socket {
            None => return format!("Token: {}", self.id),
            Some(t) => t,
        };

        match socket.local_addr() {
            Ok(a) => format!("Token: {} IP: ", self.id) + &a.to_string(),
            Err(_) => format!("Token: {}", self.id),
        }
    }

    fn envio_informacion(&self) -> bool {
        !self.mando
    }

    fn esta_conectado(&self) -> bool {
        !self.mando
    }

    fn enviar_resultado(&mut self, resultado: &ResultadoRedis) -> Result<(), RedisError> {
        self.mando = true;
        let mensaje = parsear_resultado(&resultado);
        self.enviar_mensaje(mensaje)
    }

    fn enviar_mensaje(&mut self, mensaje: String) -> Result<(), RedisError> {
        let socket = match &mut self.socket {
            None => return Err(RedisError::ConeccionError),
            Some(t) => t,
        };

        match socket.write(mensaje.as_bytes()) {
            Ok(_) => (),
            Err(_) => return Err(RedisError::ConeccionError),
        };

        match socket.flush() {
            Ok(_) => Ok(()),
            Err(_) => Err(RedisError::ConeccionError),
        }
    }

    fn obtener_token(&self) -> Token {
        self.id
    }
}

fn parsear_resultado(resultado: &ResultadoRedis) -> String {
    format!("HTTP/1.1 200 OK\r\n\r\n{}", parsear_respuesta(&resultado))
}

impl Clone for ClienteHTTP {
    fn clone(&self) -> Self {
        ClienteHTTP {
            id: self.id,
            mando: self.mando,
            socket: self.obtener_socket(),
            pag_index: self.pag_index.clone(),
        }
    }
}

impl PartialEq for ClienteHTTP {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ClienteHTTP {}

impl fmt::Debug for ClienteHTTP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClienteHTTP")
            .field("id", &self.id)
            .field("socket", &self.socket)
            .finish()
    }
}
