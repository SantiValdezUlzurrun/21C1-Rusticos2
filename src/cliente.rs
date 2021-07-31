use crate::base_de_datos::ResultadoRedis;
use crate::comando_info::ComandoInfo;
use crate::parser::parsear_respuesta;
use crate::parser::Parser;
use crate::redis_error::RedisError;
use std::time::Duration;
use std::time::Instant;

use std::io::Write;
use std::net::TcpStream;

pub type Token = i64;

#[derive(Debug)]
pub struct Cliente {
    id: Token,
    timeout: Option<Duration>,
    ultimo_mensaje: Instant,
    socket: Option<TcpStream>,
}

impl Cliente {
    pub fn new(id: Token, timeout: u64, stream: TcpStream) -> Self {
        let duracion = match timeout {
            0 => None,
            t => Some(Duration::from_secs(t)),
        };

        Cliente {
            id,
            timeout: duracion,
            ultimo_mensaje: Instant::now(),
            socket: Some(stream),
        }
    }

    pub fn obtener_comando(&self) -> Result<Option<ComandoInfo>, RedisError> {
        let stream = match self.obtener_socket() {
            Some(s) => s,
            None => return Err(RedisError::ConeccionError),
        };

        let parser = Parser::new(stream);

        match parser.parsear_stream() {
            Ok(orden) => Ok(Some(orden)),
            Err(_) => Err(RedisError::ServerError),
        }
    }

    pub fn obtener_addr(&self) -> String {
        let socket = match &self.socket {
            None => return format!("Token: {}", self.id),
            Some(t) => t,
        };

        match socket.local_addr() {
            Ok(a) => format!("Token: {} IP: ", self.id) + &a.to_string(),
            Err(_) => format!("Token: {}", self.id),
        }
    }

    pub fn envio_informacion(&self) -> bool {
        let socket = match &self.socket {
            None => return false,
            Some(t) => t,
        };

        match socket.peek(&mut [0; 128]) {
            Ok(len) => len > 0,
            Err(_) => false,
        }
    }

    pub fn esta_conectado(&self) -> bool {
        let socket = match &self.socket {
            None => return false,
            Some(t) => t,
        };

        let esta_conectado = match socket.peek(&mut [0; 128]) {
            Ok(len) => len != 0,
            Err(_) => false,
        };

        let paso_el_timeout = match self.timeout {
            Some(d) => self.ultimo_mensaje.elapsed() > d,
            None => false,
        };

        esta_conectado && !paso_el_timeout
    }

    pub fn enviar_resultado(&mut self, resultado: &ResultadoRedis) -> Result<(), RedisError> {
        let mensaje = parsear_respuesta(&resultado);
        self.enviar_mensaje(mensaje)
    }

    pub fn enviar_mensaje(&mut self, mensaje: String) -> Result<(), RedisError> {
        self.ultimo_mensaje = Instant::now();

        let socket = match &mut self.socket {
            None => return Err(RedisError::ConeccionError),
            Some(t) => t,
        };

        match socket.write(mensaje.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RedisError::ConeccionError),
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

impl Clone for Cliente {
    fn clone(&self) -> Self {
        Cliente {
            id: self.id,
            timeout: self.timeout,
            ultimo_mensaje: self.ultimo_mensaje,
            socket: self.obtener_socket(),
        }
    }
}

impl PartialEq for Cliente {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Cliente {}
