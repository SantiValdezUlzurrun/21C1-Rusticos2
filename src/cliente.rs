use crate::base_de_datos::ResultadoRedis;
use crate::cliente_http::ClienteHTTP;
use crate::cliente_redis::ClienteRedis;
use crate::comando_info::ComandoInfo;
use crate::redis_error::RedisError;
use std::fmt;
use std::fmt::Debug;

use std::net::TcpStream;

pub type Token = i64;
pub type Cliente = Box<dyn TipoCliente + Send>;

pub trait TipoCliente: ClienteClone + ClienteDebug {
    fn obtener_comando(&mut self) -> Result<Option<ComandoInfo>, RedisError>;
    fn obtener_addr(&self) -> String;
    fn envio_informacion(&self) -> bool;
    fn esta_conectado(&self) -> bool;
    fn enviar_resultado(&mut self, resultado: &ResultadoRedis) -> Result<(), RedisError>;
    fn enviar_mensaje(&mut self, mensaje: String) -> Result<(), RedisError>;
    fn obtener_token(&self) -> Token;
    fn soporta_comando(&self, comando: &str) -> bool;
}

pub trait ClienteClone {
    fn clone_box(&self) -> Box<dyn TipoCliente + Send>;
}

impl Clone for Box<dyn TipoCliente + Send> {
    fn clone(&self) -> Box<dyn TipoCliente + Send> {
        self.clone_box()
    }
}

pub trait ClienteDebug {
    fn fmt_box(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl Debug for Cliente {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_box(f)
    }
}

impl PartialEq for Cliente {
    fn eq(&self, other: &Self) -> bool {
        (**self).obtener_token() == (**other).obtener_token()
    }
}

impl Eq for Cliente {}

impl<T> ClienteDebug for T
where
    T: 'static + TipoCliente + Debug + Send,
{
    fn fmt_box(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

impl<T> ClienteClone for T
where
    T: 'static + TipoCliente + Clone + Send,
{
    fn clone_box(&self) -> Box<dyn TipoCliente + Send> {
        Box::new(self.clone())
    }
}

pub fn crear_cliente(id: Token, timeout: u64, stream: TcpStream) -> Box<dyn TipoCliente + Send> {
    let mut buffer = [0; 1024];
    match stream.peek(&mut buffer) {
        Ok(_) => (),
        Err(_) => return Box::new(ClienteRedis::new(id, timeout, stream)),
    };

    let mensaje = match String::from_utf8(buffer.to_vec()) {
        Ok(s) => s,
        Err(_) => "".to_string(),
    };
    if mensaje.contains("HTTP") {
        Box::new(ClienteHTTP::new(id, stream))
    } else {
        Box::new(ClienteRedis::new(id, timeout, stream))
    }
}
