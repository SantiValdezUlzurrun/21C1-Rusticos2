use std::net::TcpStream;
use std::net::TcpListener;
use std::collections::HashMap;
use std::io::Write;
use crate::parser::Parser;
use crate::comando::crear_comando;
use crate::comando::ResultadoRedis;

extern crate redis;

pub enum RedisError {
    ServerError,
    ConeccionError,
    InicializacionError,
}

pub struct Redis {
    direccion : String,
    tabla: HashMap<String, String>
}

impl Redis {
    
    pub fn new(host: &str, port: &str) -> Self {
        Redis {
            direccion: host.to_string() + ":" + port,
            tabla: HashMap::new(),
        }
    }
    
    pub fn iniciar(&mut self) -> Result<(), RedisError>{
        let listener = match TcpListener::bind(&self.direccion) {
            Ok(l) => l,
            Err(_) => return Err(RedisError::InicializacionError),
        };

        for stream in listener.incoming() {
            if let Ok(socket) = stream {
               
                match self.manejar_cliente(&socket) {
                    Ok(()) => (),
                    Err(e) => self.manejar_error(e),
                };
                
            }
        }
        Ok(())
    }

   fn manejar_cliente(&mut self, socket: &TcpStream) -> Result<(), RedisError> {
        
        let socket_clon = match socket.try_clone() {
            Ok(sock) => sock,
            _ => return Err(RedisError::ServerError),
        }; 
        loop {
            let parser = Parser::new(&socket_clon);
        
            let comando = match parser.parsear_stream() {
                Ok(orden) => orden,
                Err(_) => return Err(RedisError::ServerError),
            };
           
            self.manejar_comando(comando, socket);
        }
        Ok(())
    }

    fn manejar_comando(&mut self, entrada: Vec<String>, mut stream: &TcpStream) -> Result<(), RedisError>{
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
    
    fn manejar_error(&self, error: RedisError) {

    }
    
}

#[cfg(test)]
mod tests {

    #[test]
    fn recibir_comando_de_redis() {
        
        assert_eq!(2 + 2, 4);
    }
}
