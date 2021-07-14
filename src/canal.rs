use std::net::TcpStream;
use std::io::Write;

pub struct Canal {
    suscriptores: Vec<TcpStream>
}

impl Canal {

    pub fn new() -> Self {
        Canal {
            suscriptores: Vec::new(),
        }
    }

    pub fn suscribirse(&mut self, suscriptor: TcpStream) {
        self.suscriptores.push(suscriptor);
    }

    pub fn publicar(&mut self, cadena: &str) {
        for suscriptor in &mut self.suscriptores {
            suscriptor.write(cadena.as_bytes());
        }
    }

    pub fn desuscribirse(&mut self, suscriptor: TcpStream) {

    }
}
