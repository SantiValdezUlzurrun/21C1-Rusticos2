use std::fmt;

pub enum RedisError {
    ServerError,
    ConeccionError,
    InicializacionError,
}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
           RedisError::ServerError => write!(f, "ServerError error del servidor"),
           RedisError::ConeccionError => write!(f, "ConeccionError no se ha podido establecer conexion"),
           RedisError::InicializacionError => write!(f, "InicializacionError no se ha podido inicializar el servidor en el puerto especificado"),
       }
    }
}
