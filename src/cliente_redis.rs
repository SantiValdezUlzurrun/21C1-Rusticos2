use crate::parser::parsear_respuesta;
use crate::comando_info::ComandoInfo;
use std::net::TcpStream;

use crate::parser::Parser;
use crate::redis_error::RedisError;
use crate::base_de_datos::ResultadoRedis;
use crate::cliente::TipoCliente;
#[derive(Clone)]
pub struct ClienteRedis;

impl ClienteRedis {
    pub fn new() -> Self {
        ClienteRedis
    }
}

impl TipoCliente for ClienteRedis {
    fn obtener_comando(&mut self, stream: &mut TcpStream) -> Result<Option<ComandoInfo>, RedisError> {
        let parser = Parser::new(stream);

        match parser.parsear_stream() {
            Ok(orden) => Ok(Some(orden)),
            Err(_) => Err(RedisError::ServerError),
        }
    }

    fn parsear_resultado(&self, resultado: &ResultadoRedis) -> String {
        parsear_respuesta(&resultado)
    }
}
