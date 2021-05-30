use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    RedisSyntaxError,
    MensajeVacioError,
}

pub struct Parser<R> {
    lector: BufReader<R>,
}

impl<R: Read> Parser<R> {
    pub fn new(stream: R) -> Self {
        Parser {
            lector: BufReader::new(stream),
        }
    }

    pub fn parsear_stream(self) -> Result<Vec<String>, ParserError> {
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
                    return Ok(comando);
                }
            }
        }
        Ok(comando)
    }
}

pub fn parsear_int(cadena: String) -> Option<u32> {
    cadena
        .chars()
        .find(|a| a.is_digit(10))
        .and_then(|a| a.to_digit(10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuando_se_recibe_un_mensaje_de_ping_este_se_parsea_y_se_devuelve_el_comando_correcto() {
        let stream = "*1\r\n$4\r\nPING\r\n".as_bytes();
        let parser = Parser::new(stream);
        let resultado: Vec<String> = parser.parsear_stream().unwrap();
        assert_eq!(resultado, vec!("PING".to_string()));
    }

    #[test]
    fn cuando_se_recibe_un_mensaje_de_llen_este_se_parsea_y_se_devuelve_el_comando_correcto() {
        let stream = "*2\r\n$4\r\nLLEN\r\n$6\r\nmylist\r\n".as_bytes();
        let parser = Parser::new(stream);
        let resultado: Vec<String> = parser.parsear_stream().unwrap();
        assert_eq!(resultado, vec!("LLEN".to_string(), "mylist".to_string()));
    }

    #[test]
    fn cuando_se_recibe_un_mensaje_de_sort_este_se_parsea_y_se_devuelve_el_comando_correcto() {
        let stream = "*7\r\n$4\r\nSORT\r\n$6\r\nmylist\r\n$5\r\nLIMIT\r\n$1\r\n0\r\n$1\r\n5\r\n$5\r\nALPHA\r\n$4\r\nDESC\r\n".as_bytes();
        let parser = Parser::new(stream);
        let resultado: Vec<String> = parser.parsear_stream().unwrap();
        let e: Vec<&str> = vec!["SORT", "mylist", "LIMIT", "0", "5", "ALPHA", "DESC"];
        let esperado: Vec<String> = e.iter().map(|s| s.to_string()).collect();
        assert_eq!(resultado, esperado);
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
}