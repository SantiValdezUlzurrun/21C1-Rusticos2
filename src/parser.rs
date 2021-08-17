use crate::base_de_datos::ResultadoRedis;
use crate::comando_info::ComandoInfo;
use std::io::{BufRead, BufReader, Read};

/// Errores que pueden ocurrir en la ejecucion del Parser
#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    /// Error en como esta formateado el Comando de Redis
    RedisSyntaxError,
    /// Se esperaba una cadena pero estaba vacia
    MensajeVacioError,
}

/// Entidad encargada de parsear stream que cumplen con la sintaxis de Redis
pub struct Parser<R> {
    lector: BufReader<R>,
}

impl<R: Read> Parser<R> {
    /// Instancia a un parser con cualquier entidad que implemente el trait Read
    /// a partir de el se encargara de parsear
    pub fn new(stream: R) -> Self {
        Parser {
            lector: BufReader::new(stream),
        }
    }

    /// Parsea el stream obteniendo un Comando o un Error
    pub fn parsear_stream(self) -> Result<ComandoInfo, ParserError> {
        let mut lineas = self.lector.lines();

        let primer_valor = match lineas.next() {
            Some(Ok(valor)) => valor,
            _ => return Err(ParserError::MensajeVacioError),
        };

        let capacidad = match parsear_int(primer_valor) {
            Some(valor) => valor,
            None => return Err(ParserError::RedisSyntaxError),
        };

        let mut comando = Vec::with_capacity(capacidad as usize);

        while let Some(Ok(longitud_str)) = lineas.next() {
            if let Some(Ok(argumento)) = lineas.next() {
                let longitud = match parsear_int(longitud_str) {
                    Some(valor) => valor as usize,
                    None => return Err(ParserError::RedisSyntaxError),
                };

                if longitud != argumento.len() {
                    return Err(ParserError::RedisSyntaxError);
                }
                comando.push(argumento);
                if comando.len() == capacidad as usize {
                    return Ok(ComandoInfo::new(comando));
                }
            }
        }
        println!("{:?}", comando);
        Ok(ComandoInfo::new(comando))
    }
}

/// Parsea la respuesta para que cumpla con el protocolo Redis
pub fn parsear_respuesta(res: &ResultadoRedis) -> String {
    match res {
        ResultadoRedis::StrSimple(cad) => format!("+{}\r\n", cad),
        ResultadoRedis::BulkStr(cad) => format!("${}\r\n{}\r\n", cad.len(), cad),
        ResultadoRedis::Int(ent) => format!(":{}\r\n", ent),
        ResultadoRedis::Vector(vec) => format!(
            "*{}\r\n{}",
            vec.len(),
            vec.iter()
                .map(|r| parsear_respuesta(r))
                .collect::<Vec<String>>()
                .join("")
        ),
        ResultadoRedis::Nil => "$-1\r\n".to_string(),
        ResultadoRedis::Error(e) => format!("-{}\r\n", e),
        ResultadoRedis::Vacio => String::new(),
    }
}

pub fn parsear_int(cadena: String) -> Option<u32> {
    let mut chars = cadena.chars();
    chars.next();
    match chars.as_str().parse::<u32>() {
        Ok(int) => Some(int),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuando_se_recibe_un_mensaje_de_ping_este_se_parsea_y_se_devuelve_el_comando_correcto() {
        let stream = "*1\r\n$4\r\nPING\r\n".as_bytes();
        let parser = Parser::new(stream);
        let mut resultado = parser.parsear_stream().unwrap();
        assert_eq!(resultado.get_nombre(), "PING".to_string());
        assert_eq!(resultado.get_clave(), None);
        assert_eq!(resultado.get_parametro(), None);
    }

    #[test]
    fn cuando_se_recibe_un_mensaje_de_llen_este_se_parsea_y_se_devuelve_el_comando_correcto() {
        let stream = "*2\r\n$4\r\nLLEN\r\n$6\r\nmylist\r\n".as_bytes();
        let parser = Parser::new(stream);
        let mut resultado = parser.parsear_stream().unwrap();
        assert_eq!(resultado.get_nombre(), "LLEN".to_string());
        assert_eq!(resultado.get_clave(), Some("mylist".to_string()));
        assert_eq!(resultado.get_parametro(), None);
    }

    #[test]
    fn cuando_se_recibe_un_mensaje_de_sort_este_se_parsea_y_se_devuelve_el_comando_correcto() {
        let stream = "*7\r\n$4\r\nSORT\r\n$6\r\nmylist\r\n$5\r\nLIMIT\r\n$1\r\n0\r\n$1\r\n5\r\n$5\r\nALPHA\r\n$4\r\nDESC\r\n".as_bytes();
        let parser = Parser::new(stream);
        let mut resultado = parser.parsear_stream().unwrap();

        assert_eq!(resultado.get_nombre(), "SORT".to_string());
        assert_eq!(resultado.get_clave(), Some("mylist".to_string()));
        assert_eq!(resultado.get_parametro(), Some("LIMIT".to_string()));
        assert_eq!(resultado.get_parametro(), Some("0".to_string()));
        assert_eq!(resultado.get_parametro(), Some("5".to_string()));
        assert_eq!(resultado.get_parametro(), Some("ALPHA".to_string()));
        assert_eq!(resultado.get_parametro(), Some("DESC".to_string()));
    }

    #[test]
    fn cuando_se_manda_un_mensaje_vacio_se_lanza_un_parser_error_de_tipo_mensaje_vacio() {
        let stream = "".as_bytes();
        let parser = Parser::new(stream);
        let error = parser.parsear_stream().unwrap_err();
        assert_eq!(error, ParserError::MensajeVacioError);
    }

    #[test]
    fn cuando_se_manda_un_mensaje_con_un_error_de_sintaxis_se_lanza_un_redis_syntax_error() {
        let stream = "++\r\n$4\r\n".as_bytes();
        let parser = Parser::new(stream);
        let error = parser.parsear_stream().unwrap_err();
        assert_eq!(error, ParserError::RedisSyntaxError);
    }

    #[test]
    fn cuando_se_envia_un_resultado_redis_simple_string_envia_un_string_correcto() {
        let resultado = ResultadoRedis::StrSimple("Ok".to_string());
        assert_eq!(parsear_respuesta(&resultado), "+Ok\r\n");
    }

    #[test]
    fn cuando_se_envia_un_resultado_redis_bulk_strings_se_parsea_correctamente() {
        let resultado = ResultadoRedis::BulkStr("foo".to_string());
        assert_eq!(parsear_respuesta(&resultado), "$3\r\nfoo\r\n");
    }

    #[test]
    fn cuando_se_envia_un_resultado_redis_int_se_parsea_correctamente() {
        let resultado = ResultadoRedis::Int(55);
        assert_eq!(parsear_respuesta(&resultado), ":55\r\n");
    }

    #[test]
    fn cuando_se_envia_un_resultado_redis_vector_de_ints_se_parsea_correctamente() {
        let resultado =
            ResultadoRedis::Vector(vec![ResultadoRedis::Int(1), ResultadoRedis::Int(2)]);
        assert_eq!(parsear_respuesta(&resultado), "*2\r\n:1\r\n:2\r\n");
    }

    #[test]
    fn cuando_se_envia_un_resultado_redis_vector_de_resultados_se_parsea_correctamente() {
        let resultado = ResultadoRedis::Vector(vec![
            ResultadoRedis::Int(1),
            ResultadoRedis::Int(2),
            ResultadoRedis::Int(3),
            ResultadoRedis::Int(4),
            ResultadoRedis::BulkStr("foobar".to_string()),
        ]);
        assert_eq!(
            parsear_respuesta(&resultado),
            "*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$6\r\nfoobar\r\n"
        );
    }

    #[test]
    fn a() {
        let stream = "*3\r\n$3\r\nSET\r\n$7\r\ncatedra\r\n$18\r\nTallerProgramacion\r\n".as_bytes();
        let parser = Parser::new(stream);
        let mut resultado = parser.parsear_stream().unwrap();
        assert_eq!(resultado.get_nombre(), "SET".to_string());
        assert_eq!(resultado.get_clave(), Some("catedra".to_string()));
        assert_eq!(
            resultado.get_parametro(),
            Some("TallerProgramacion".to_string())
        );
    }
}
