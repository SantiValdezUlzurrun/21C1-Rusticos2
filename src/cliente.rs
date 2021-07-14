use std::cmp::Ordering;
use crate::redis_error::RedisError;

use std::io::Write;
use std::net::TcpStream;


pub type Token = i64;

pub struct Cliente {
    id: Token,
    socket: TcpStream,
}

impl Cliente {

    pub fn new(id: Token, socket: TcpStream) -> Self {
        Cliente{
            id,
            socket,
        }
    }

    pub fn obtener_addr(&self) -> String {
        match &self.socket.local_addr() {
            Ok(a) => format!("Token: {} IP: ", self.id) + &a.to_string(),
            Err(_) => format!("Token: {}", self.id)
        }
    }

    pub fn obtener_socket(&self) -> TcpStream {
        self.socket.try_clone().unwrap()
    }

    pub fn envio_informacion(&self) -> bool {
        match self.socket.peek(&mut [0; 128]) {
            Ok(len) => len > 0,
            Err(_) => false,
        }
    }

    pub fn esta_conectado(&self) -> bool {
        match self.socket.peek(&mut [0; 128]) {
            Ok(len) => len != 0,
            Err(_) => false,
        }
    }

    pub fn enviar(&mut self, mensaje: String) -> Result<(), RedisError>{
        match self.socket.write(mensaje.as_bytes()) {
                Ok(_) => Ok(()),
                Err(_) => Err(RedisError::ConeccionError),
        }
    }
}

impl PartialEq for Cliente {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Cliente { }
