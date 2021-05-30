use crate::comando::crear_comando;
use crate::comando::ResultadoRedis;
use crate::parser::Parser;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

extern crate redis;

pub enum RedisError {
    ServerError,
    ConeccionError,
    InicializacionError,
}

pub struct Redis {
    direccion: String,
    tabla: HashMap<String, String>,
    _verbose: bool,
    _timeout: u32,
}

impl Redis {
    pub fn new(host: &str, port: &str, _verbose: bool, _timeout: u32) -> Self {
        Redis {
            direccion: host.to_string() + ":" + port,
            tabla: HashMap::new(),
            _verbose,
            _timeout,
        }
    }

    pub fn iniciar(&mut self) -> Result<(), RedisError> {
        let listener = match TcpListener::bind(&self.direccion) {
            Ok(l) => l,
            Err(_) => return Err(RedisError::InicializacionError),
        };

        for stream in listener.incoming().flatten() {
            match self.manejar_cliente(&stream) {
                Ok(()) => (),
                Err(e) => self.manejar_error(e),
            };
        }
        Ok(())
    }

    fn manejar_cliente(&mut self, socket: &TcpStream) -> Result<(), RedisError> {
        let socket_clon = match socket.try_clone() {
            Ok(sock) => sock,
            _ => return Err(RedisError::ServerError),
        };
        loop {
            if self.cliente_envio_informacion(socket) {
                let parser = Parser::new(&socket_clon);

                let comando = match parser.parsear_stream() {
                    Ok(orden) => orden,
                    Err(_) => return Err(RedisError::ServerError),
                };

                match self.manejar_comando(comando, socket) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                };
            } else if !self.cliente_esta_conectado(socket) {
                break;
            }
        }
        Ok(())
    }

    fn manejar_comando(
        &mut self,
        entrada: Vec<String>,
        mut stream: &TcpStream,
    ) -> Result<(), RedisError> {
        let comando = crear_comando(&entrada);

        let resultado = match comando.ejecutar(&mut self.tabla) {
            Ok(ResultadoRedis::Str(r)) => r,
            Err(_) => return Err(RedisError::ServerError),
        };

        match stream.write(format!("+{}\r\n", resultado).as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RedisError::ConeccionError),
        }
    }

    fn cliente_envio_informacion(&self, socket: &TcpStream) -> bool {
        match socket.peek(&mut [0; 128]) {
            Ok(len) => len > 0,
            Err(_) => false,
        }
    }

    fn cliente_esta_conectado(&self, socket: &TcpStream) -> bool {
        match socket.peek(&mut [0; 128]) {
            Ok(len) => len != 0,
            Err(_) => false,
        }
    }

    fn manejar_error(&self, _error: RedisError) {}
}