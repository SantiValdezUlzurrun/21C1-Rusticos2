use crate::redis_error::RedisError;

use std::io::Write;
use std::net::TcpStream;


pub type Token = i64;

#[derive(Debug)]
pub struct Cliente {
    id: Token,
    socket: Option<TcpStream>,
}

impl Cliente {

    pub fn new(id: Token, socket: TcpStream) -> Self {
        Cliente{
            id,
            socket: Some(socket),
        }
    }

    pub fn obtener_addr(&self) -> String {
        let socket = match &self.socket {
            None => return format!("Token: {}", self.id),
            Some(t) => t,
        };

        match socket.local_addr() {
            Ok(a) => format!("Token: {} IP: ", self.id) + &a.to_string(),
            Err(_) => format!("Token: {}", self.id)
        }
    }

    pub fn obtener_socket(&self) -> Option<TcpStream> {
        let socket = match &self.socket {
            None => return None,
            Some(t) => t,
        };

        match socket.try_clone() {
            Ok(t) => Some(t),
            Err(_) => None,
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

        match socket.peek(&mut [0; 128]) {
            Ok(len) => len != 0,
            Err(_) => false,
        }
    }

    pub fn enviar(&mut self, mensaje: String) -> Result<(), RedisError>{
        let socket = match &mut self.socket {
            None => return Err(RedisError::ConeccionError),
            Some(t) => t,
        };

        match socket.write(mensaje.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RedisError::ConeccionError),
        }
    }
}

impl Clone for Cliente {
    fn clone(&self) -> Self {
        Cliente {
            id: self.id,
            socket: self.obtener_socket(),
        }
    }
}

impl PartialEq for Cliente {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Cliente { }
