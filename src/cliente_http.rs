use crate::cliente::TipoCliente;
use std::fs::read_to_string;
use crate::http_parser::HTTPParser;
use crate::comando_http::ComandoHTTP;
use crate::base_de_datos::ResultadoRedis;
use crate::comando_info::ComandoInfo;
use crate::parser::parsear_respuesta;
use crate::redis_error::RedisError;

use std::io::Write;
use std::net::TcpStream;

pub type Token = i64;

#[derive(Debug)]
pub struct ClienteHTTP {
    id: Token,
    socket: Option<TcpStream>,
    pag_index: String,
}

impl ClienteHTTP {
    pub fn new(id: Token, socket: TcpStream) -> Self {
        let pag_index = match read_to_string("resources/index.html") {
            Ok(s) => s,
            Err(_) => "<html></html>".to_string(),
        };


        ClienteHTTP {
            id,
            pag_index,
            socket: Some(socket),
        }
    }

    fn manejar_get(&mut self, _comando: ComandoHTTP) -> Option<ComandoInfo> {
        let respuesta = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}", self.pag_index);
        self.enviar_mensaje(respuesta);
        None
    }

    fn manejar_error(&mut self, _comando: ComandoHTTP) -> Option<ComandoInfo> {
        println!("Error");
        None
    }

    pub fn enviar_mensaje(&mut self, mensaje: String) -> Result<(), RedisError> {

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

    fn obtener_comando(&mut self, stream: &mut TcpStream) -> Result<Option<ComandoInfo>, RedisError> {
        let parser = HTTPParser::new(stream);
        let comando_http = match parser.parsear_stream() {
            Ok(orden) => orden,
            Err(_) => return Err(RedisError::ServerError),
        };

        match comando_http.get_metodo().as_str() {
            "GET" => {
                self.manejar_get(comando_http);
                Ok(None)
            },
            "POST" => Ok(Some(comando_http.get_comando().unwrap())),
            _ => {
                self.manejar_error(comando_http);
                Ok(None)
            }
        }
    }
    fn parsear_resultado(&self, resultado: &ResultadoRedis) -> String {
        format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<html><body>{}</body></html>", parsear_respuesta(&resultado))
    }



}

impl Clone for ClienteHTTP {
    fn clone(&self) -> Self {
        ClienteHTTP {
            id: self.id,
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
