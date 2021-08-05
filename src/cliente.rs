use crate::cliente_http::ClienteHTTP;
use crate::cliente_redis::ClienteRedis;
use crate::base_de_datos::ResultadoRedis;
use crate::comando_info::ComandoInfo;
use crate::redis_error::RedisError;
use std::time::Duration;
use std::time::Instant;

use std::io::Write;
use std::net::TcpStream;
use std::fmt;

pub type Token = i64;

pub trait TipoCliente: ClienteClone {
    fn obtener_comando(&mut self, stream: &mut TcpStream) -> Result<Option<ComandoInfo>, RedisError>;
    fn parsear_resultado(&self, resultado: &ResultadoRedis) -> String;
}
pub trait ClienteClone {
    fn clone_box(&self) -> Box<dyn TipoCliente + Send>;
}

impl Clone for Box<dyn TipoCliente + Send> {
    fn clone(&self) -> Box<dyn TipoCliente + Send> {
        self.clone_box()
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

fn tipo_cliente(id: Token, stream: TcpStream, mensaje: String) -> Box<dyn TipoCliente + Send> {

    if mensaje.contains("HTTP") {
        Box::new(ClienteHTTP::new(id, stream))
    } else {
        Box::new(ClienteRedis::new())
    }
}

pub struct Cliente {
    id: Token,
    timeout: Option<Duration>,
    ultimo_mensaje: Instant,
    socket: Option<TcpStream>,
    tipo: Box<dyn TipoCliente + Send>,
}

impl Cliente {
    pub fn new(id: Token, timeout: u64, stream: TcpStream) -> Self {
        let duracion = match timeout {
            0 => None,
            t => Some(Duration::from_secs(t)),
        };
        let mut buffer = [0; 1024];
        stream.peek(&mut buffer);
        let porcion = match String::from_utf8(buffer.to_vec()){
            Ok(s) => s,
            Err(_) => "".to_string(),
        };

        let tipo = tipo_cliente(id, stream.try_clone().unwrap(), porcion);
        Cliente {
            id,
            timeout: duracion,
            ultimo_mensaje: Instant::now(),
            socket: Some(stream),
            tipo
        }
    }

    pub fn obtener_comando(&mut self) -> Result<Option<ComandoInfo>, RedisError> {
        let mut stream = match self.obtener_socket() {
            Some(s) => s,
            None => return Err(RedisError::ConeccionError),
        };
        self.tipo.obtener_comando(&mut stream)
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
        let mensaje = self.tipo.parsear_resultado(&resultado);
        self.enviar_mensaje(mensaje)
    }

    pub fn enviar_mensaje(&mut self, mensaje: String) -> Result<(), RedisError> {
        self.ultimo_mensaje = Instant::now();

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

impl Clone for Cliente {
    fn clone(&self) -> Self {
        Cliente {
            id: self.id,
            timeout: self.timeout,
            ultimo_mensaje: self.ultimo_mensaje,
            socket: self.obtener_socket(),
            tipo: self.tipo.clone_box(),
        }
    }
}

impl PartialEq for Cliente {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Cliente {}

impl fmt::Debug for Cliente {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cliente")
         .field("id", &self.id)
         .field("timeout", &self.timeout)
         .field("ultimo_mensaje", &self.ultimo_mensaje)
         .field("socket", &self.socket)
         .finish()
    }
}
