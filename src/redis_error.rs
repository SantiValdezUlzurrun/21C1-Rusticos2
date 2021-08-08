use std::fmt;

pub enum RedisError {
    Server,
    Coneccion,
    Inicializacion,
}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
           RedisError::Server => write!(f, "ServerError error del servidor"),
           RedisError::Coneccion => write!(f, "ConeccionError no se ha podido establecer conexion"),
           RedisError::Inicializacion => write!(f, "InicializacionError no se ha podido inicializar el servidor en el puerto especificado"),
       }
    }
}
