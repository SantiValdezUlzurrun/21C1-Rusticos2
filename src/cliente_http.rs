use crate::base_de_datos::ResultadoRedis;
use crate::cliente::{TipoCliente, Token};
use crate::comando_http::ComandoHttp;
use crate::comando_info::ComandoInfo;
use crate::http_parser::{parsear_respuesta, HttpParser};
use crate::redis_error::RedisError;
use std::fs::{read_to_string, File};

use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;

/// Representa un Cliente que se comunica utilizando el protocolo HTTP
pub struct ClienteHttp {
    id: Token,
    socket: Option<TcpStream>,
    mando: bool,
    pag_index: String,
    icono: Vec<u8>,
}

impl ClienteHttp {
    /// Se instancia un ClienteHTTP en condiciones de procesar mensajes
    ///
    /// # Argumentos
    ///
    /// * `token` - id unica
    /// * `socket` - stream especifico del cliente
    pub fn new(id: Token, socket: TcpStream) -> Self {
        let pag_index = match read_to_string("resources/index.html") {
            Ok(s) => s,
            Err(_) => "<html><body>Error al levantar la pagina</body></html>".to_string(),
        };

        let mut buffer = Vec::new();
        match File::open("resources/favicon.png") {
            Ok(mut file) => match file.read_to_end(&mut buffer) {
                Ok(_) => (),
                Err(_) => buffer = Vec::new(),
            },
            Err(_) => buffer = Vec::new(),
        };

        ClienteHttp {
            id,
            pag_index,
            icono: buffer,
            socket: Some(socket),
            mando: false,
        }
    }

    /// Procesa requests Get devolviendo que no recibio ningun comando
    fn manejar_get(&mut self, comando: ComandoHttp) -> Result<Option<ComandoInfo>, RedisError> {
        if comando.get_argumento() == Some("/favicon.ico".to_string()) {
            let respuesta = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/gif\r\nContent-Length: {}\r\n\r\n",
                self.icono.len(),
            );

            match self.enviar_mensaje(respuesta) {
                Ok(_) => (),
                Err(_) => return Err(RedisError::Server),
            };

            match self.enviar_bytes(&self.icono.clone()) {
                Ok(_) => Ok(None),
                Err(_) => Err(RedisError::Server),
            }
        } else {
            let respuesta = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}",
                self.pag_index
            );

            match self.enviar_mensaje(respuesta) {
                Ok(_) => Ok(None),
                Err(_) => Err(RedisError::Server),
            }
        }
    }

    /// Obtiene el comando redis encapsulado en el ComandoHTTP
    fn obtener_comando_de_post(
        &mut self,
        comando: ComandoHttp,
    ) -> Result<Option<ComandoInfo>, RedisError> {
        match comando.get_comando() {
            Some(c) => Ok(Some(c)),
            None => Err(RedisError::Server),
        }
    }

    /// Maneja cualquier tipo de request no manejada respondiendo que no se recibio ningun comando
    fn manejar_error(&mut self, _comando: ComandoHttp) -> Result<Option<ComandoInfo>, RedisError> {
        let respuesta = "HTTP/1.1 400 BAD REQUEST\r\n\r\n".to_string();
        match self.enviar_mensaje(respuesta) {
            Ok(_) => Ok(None),
            Err(_) => Err(RedisError::Server),
        }
    }

    /// Intenta clonar el TcpStream asociado al Cliente
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

    fn enviar_bytes(&mut self, bytes: &[u8]) -> Result<(), RedisError> {
        let socket = match &mut self.socket {
            None => return Err(RedisError::Coneccion),
            Some(t) => t,
        };

        match socket.write(bytes) {
            Ok(_) => (),
            Err(_) => return Err(RedisError::Coneccion),
        };

        match socket.flush() {
            Ok(_) => Ok(()),
            Err(_) => Err(RedisError::Coneccion),
        }
    }
}

impl TipoCliente for ClienteHttp {
    /// Encapsula el obtener el comando en particular
    ///
    /// # Resultados
    ///
    /// * `Ok(Some(c))` - Se obtiene el comando enviado correctamente
    /// * `Ok(None)` - El usuario no envio un comando pero su request fue procesada correctamente
    /// * `Err(e)` - Se produjo un error al la hora de obtener el comando
    fn obtener_comando(&mut self) -> Result<Option<ComandoInfo>, RedisError> {
        let stream = match self.obtener_socket() {
            Some(s) => s,
            None => return Err(RedisError::Coneccion),
        };
        self.mando = true;
        let parser = HttpParser::new(stream);
        let comando_http = match parser.parsear_stream() {
            Ok(orden) => orden,
            Err(_) => return Err(RedisError::Server),
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

        let mensaje = format!("HTTP/1.1 200 OK\r\n\r\n{}", parsear_respuesta(resultado));
        self.enviar_mensaje(mensaje)
    }

    fn enviar_mensaje(&mut self, mensaje: String) -> Result<(), RedisError> {
        self.enviar_bytes(mensaje.as_bytes())
    }

    fn obtener_token(&self) -> Token {
        self.id
    }

    fn soporta_comando(&self, comando: &str) -> bool {
        let comandos = vec![
            "COPY",
            "DEL",
            "EXISTS",
            "RENAME",
            "EXPIRE",
            "EXPIREAT",
            "PERSIST",
            "TTL",
            "TOUCH",
            "KEYS",
            "SORT",
            "TYPE",
            "LINDEX",
            "LPOP",
            "RPOP",
            "LPUSH",
            "LPUSHX",
            "RPUSH",
            "RPUSHX",
            "LRANGE",
            "LREM",
            "LSET",
            "LLEN",
            "SADD",
            "SCARD",
            "SISMEMBER",
            "SMEMBERS",
            "SREM",
            "GET",
            "SET",
            "APPEND",
            "STRLEN",
            "INCRBY",
            "DECRBY",
            "MGET",
            "MSET",
            "GETSET",
            "GETDEL",
            "FLUSHDB",
            "DBSIZE",
            "CONFIG",
            "INFO",
            "PING",
        ];
        comandos.iter().any(|&c| c == comando)
    }
}

impl Clone for ClienteHttp {
    fn clone(&self) -> Self {
        ClienteHttp {
            id: self.id,
            mando: self.mando,
            socket: self.obtener_socket(),
            pag_index: self.pag_index.clone(),
            icono: self.icono.clone(),
        }
    }
}

impl PartialEq for ClienteHttp {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ClienteHttp {}

impl fmt::Debug for ClienteHttp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClienteHTTP")
            .field("id", &self.id)
            .field("socket", &self.socket)
            .finish()
    }
}
