use crate::cliente::Cliente;

#[derive(Debug, PartialEq, Clone)]
pub struct Canal {
    suscriptores: Vec<Cliente>
}

impl Canal {

    pub fn new() -> Self {
        Canal {
            suscriptores: Vec::new(),
        }
    }

    pub fn suscribirse(&mut self, suscriptor: Cliente) {
        self.suscriptores.push(suscriptor);
    }

    pub fn publicar(&mut self, mensaje: String) {
        for suscriptor in &mut self.suscriptores {
            suscriptor.enviar(mensaje.clone());
        }
    }

    pub fn desuscribirse(&mut self, suscriptor: Cliente) {
        let index = self.suscriptores.iter().position(|x| *x == suscriptor).unwrap();
        self.suscriptores.remove(index);
    }
}
