use crate::base_de_datos::ResultadoRedis;
use crate::comando_http::ComandoHTTP;
use crate::parser::ParserError;
use std::io::{BufReader, Read};

pub struct HTTPParser<R> {
    lector: BufReader<R>,
}

impl<R: Read + std::fmt::Debug> HTTPParser<R> {
    pub fn new(stream: R) -> Self {
        HTTPParser {
            lector: BufReader::new(stream),
        }
    }

    pub fn parsear_stream(mut self) -> Result<ComandoHTTP, ParserError> {
        let mut buffer = vec![0; 5000];
        match self.lector.read(&mut buffer) {
            Ok(_) => (),
            _ => return Err(ParserError::MensajeVacioError),
        };
        let mut mensaje = match String::from_utf8(buffer) {
            Ok(s) => s,
            Err(_) => return Err(ParserError::MensajeVacioError),
        };
        mensaje.push_str("\r\n");
        let mut lineas: Vec<&str> = mensaje.split("\r\n").collect();
        let metodo = match obtener_metodo(lineas.remove(0)) {
            Ok(m) => m,
            Err(e) => return Err(e),
        };
        let headers = match obtener_headers(&mut lineas) {
            Ok(m) => m,
            Err(e) => return Err(e),
        };
        let comando = match obtener_comando_redis(&mut lineas, &metodo[0]) {
            Ok(m) => m,
            Err(e) => return Err(e),
        };
        Ok(ComandoHTTP::new(metodo, headers, comando))
    }
}

pub fn parsear_respuesta(res: &ResultadoRedis) -> String {
    match res {
        ResultadoRedis::StrSimple(cad) => format!("(string) {}", cad),
        ResultadoRedis::BulkStr(cad) => format!("(bulk-string) {}", cad),
        ResultadoRedis::Int(ent) => format!("(int) {}", ent),
        ResultadoRedis::Vector(vec) => format!(
            "(vector) {}",
            vec.iter()
                .map(|r| parsear_respuesta(r))
                .collect::<Vec<String>>()
                .join("")
        ),
        ResultadoRedis::Nil => "(nil)".to_string(),
        ResultadoRedis::Error(e) => format!("(error) {}", e),
    }
}

fn obtener_metodo(linea: &str) -> Result<Vec<String>, ParserError> {
    let metodo: Vec<&str> = linea.split(' ').collect();
    if metodo.len() != 3 {
        Err(ParserError::MensajeVacioError)
    } else {
        Ok(metodo.iter().map(|s| s.to_string()).collect())
    }
}

fn obtener_headers(lineas: &mut Vec<&str>) -> Result<Vec<String>, ParserError> {
    let mut headers = vec![];
    let mut index = 0;
    for linea in lineas.iter() {
        if (*linea).is_empty() {
            break;
        }
        let header: Vec<&str> = linea.split(": ").collect();
        if header.len() != 2 {
            return Err(ParserError::MensajeVacioError);
        }
        headers.append(&mut header.iter().map(|s| s.to_string()).collect());
        index += 1;
    }
    lineas.drain(0..index + 1);
    Ok(headers)
}

fn obtener_comando_redis(lineas: &mut Vec<&str>, metodo: &str) -> Result<Vec<String>, ParserError> {
    if metodo != "POST" {
        return Ok(Vec::new());
    }
    let linea = lineas[0].replace("\0", "");
    let comando: Vec<&str> = linea.split('=').collect();
    if comando[0] != "comando" {
        return Err(ParserError::RedisSyntaxError);
    }
    let comando_redis: Vec<&str> = comando[1].split('+').collect();
    Ok(comando_redis.iter().map(|s| s.to_string()).collect())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn comando_http_parsea_bien_el_metodo_get() {
        let stream = "GET /hello.htm HTTP/1.1\r\nUser-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\nHost: www.tutorialspoint.com\r\nAccept-Language: en-us\r\nAccept-Encoding: gzip, deflate\r\nConnection: Keep-Alive\r\n\r\n".as_bytes();
        let parser = HTTPParser::new(stream);

        let comando = parser.parsear_stream().unwrap();

        assert_eq!("GET".to_string(), comando.get_metodo());
        assert!(comando.get_comando().is_none());
    }

    #[test]
    fn comando_http_parsea_bien_el_metodo_post_obteniendo_el_comando_set_key_1() {
        let stream = "POST / HTTP/1.1\r\nHost: foo.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 13\r\n\r\ncomando=set+key+1".as_bytes();
        let parser = HTTPParser::new(stream);

        let comando = parser.parsear_stream().unwrap();

        assert_eq!("POST".to_string(), comando.get_metodo());
        assert_eq!(
            "set".to_string(),
            comando.get_comando().unwrap().get_nombre()
        );
    }

    #[test]
    fn comando_http_parsea_bien_el_metodo_get_con_mas_headers() {
        let stream = "GET / HTTP/1.1\r\nHost: 127.0.0.1:8080\r\nConnection: keep-alive\r\nCache-Control: max-age=0\r\nsec-ch-ua: ' Not A;Brand';v='99', 'Chromium';v='92'\r\nsec-ch-ua-mobile: ?0\r\nUpgrade-Insecure-Requests: 1\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.107 Safari/537.36\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9\r\nSec-Fetch-Site: none\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-User: ?1\r\nSec-Fetch-Dest: document\r\nAccept-Encoding: gzip, deflate, br\r\nAccept-Language: es-419,es;q=0.9,en;q=0.8\r\nCookie: _xsrf=2|e2fa887a|143bb4d760e3f93ddf62f00d31f52215|1626138365; username-127-0-0-1-8888='2|1:0|10:1627770495|23:username-127-0-0-1-8888|44:ZmViN2NkZDcxM2YyNGQ3NzkzOTVjZjkxZDA0ZjBmNjM=|155cba90c7e7bc1b8f94b9505143b6d57153add6b773957f373328a688fd5cef\r\n\r\n\r\n".as_bytes();
        let parser = HTTPParser::new(stream);

        let comando = parser.parsear_stream().unwrap();

        assert_eq!("GET".to_string(), comando.get_metodo());
        assert!(comando.get_comando().is_none());
    }
}
