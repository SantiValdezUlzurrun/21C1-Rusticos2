use std::fmt;

/// Representa un error ocurrido por la ejecucion del servidor de Redis
pub enum RedisError {
    Server,
    Coneccion,
    Inicializacion,
}

/// Mensaje mas descriptivo del porque del lanzamiento del error
impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
           RedisError::Server => write!(f, "ServerError error del servidor"),
           RedisError::Coneccion => write!(f, "ConeccionError no se ha podido establecer conexion"),
           RedisError::Inicializacion => write!(f, "InicializacionError no se ha podido inicializar el servidor en el puerto especificado"),
       }
    }
}
