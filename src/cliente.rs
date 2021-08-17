use crate::base_de_datos::ResultadoRedis;
use crate::cliente_http::ClienteHTTP;
use crate::cliente_redis::ClienteRedis;
use crate::comando_info::ComandoInfo;
use crate::redis_error::RedisError;
use std::fmt;
use std::fmt::Debug;

use std::net::TcpStream;

/// Token unico asociado a un Cliente
pub type Token = i64;

/// Interfaz de Cliente
pub type Cliente = Box<dyn TipoCliente + Send>;

/// Mensajes publicos que un Cliente debe implementar
pub trait TipoCliente: ClienteClone + ClienteDebug {
    /// Encapsula el obtener el comando en particular
    ///
    /// # Resultados
    ///
    /// * `Ok(Some(c))` - Se obtiene el comando enviado correctamente
    /// * `Ok(None)` - El usuario no envio un comando pero fue procesada correctamente
    /// * `Err(e)` - Se produjo un error al la hora de obtener el comando
    fn obtener_comando(&mut self) -> Result<Option<ComandoInfo>, RedisError>;

    /// Devuelve una descripcion del Cliente
    fn obtener_addr(&self) -> String;

    fn envio_informacion(&self) -> bool;

    fn esta_conectado(&self) -> bool;

    /// enviar el resultado procesandolo en el protocolo especifico
    fn enviar_resultado(&mut self, resultado: &ResultadoRedis) -> Result<(), RedisError>;

    /// envia un mensaje sin procesar al Cliente
    fn enviar_mensaje(&mut self, mensaje: String) -> Result<(), RedisError>;

    fn obtener_token(&self) -> Token;

    /// Predicado que indica si un Cliente puede enviar determinado comando
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

/// Crea a un cliente especifico dependiendo de como sea el protocolo que utilice
/// ya sea HTTP o Redis
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
