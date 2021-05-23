use std::net::TcpStream;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;

use crate::parser::Parser;

extern crate redis;

pub enum RedisError {
    ServerError,
    ConeccionError,
    InicializacionError,
}

pub struct Redis {
    direccion : String,
    //tabla: BaseDatosRedis
}

impl Redis {
    
    pub fn new(host: &str, port: &str) -> Self {
        Redis {
            direccion: host.to_string() + ":" + port,
        }
    }
    
    pub fn iniciar(&self) -> Result<(), RedisError>{
        let listener = match TcpListener::bind(&self.direccion) {
            Ok(l) => l,
            Err(e) => return Err(RedisError::InicializacionError),
        };

        for stream in listener.incoming() {
            if let Ok(mut socket) = stream {
                
                match self.manejar_cliente(socket) {
                    Ok(()) => (),
                    Err(e) => self.manejar_error(e),
                };
            }
        }
        Ok(())
    }

   fn manejar_cliente(&self, mut socket: TcpStream) -> Result<(), RedisError> {
        
        let socket_clon = match socket.try_clone() {
            Ok(sock) => sock,
            _ => return Err(RedisError::ServerError),
        };
        let parser = Parser::new(socket_clon);
        let comando = match parser.parsear_stream() {
            Ok(orden) => orden,
            Err(e) => return Err(RedisError::ServerError),
        };
        self.manejar_comando(comando, socket);

        Ok(())
    }

    fn manejar_comando(&self, entrada: Vec<String>, mut stream: TcpStream) -> std::io::Result<()>{
        println!("{:?}", entrada);
        if "ping" == entrada[0] {
            stream.write("PONG".as_bytes())?;
            stream.write("\n".as_bytes())?;
        }
        Ok(())
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
