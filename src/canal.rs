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

    pub fn publicar(&mut self, mensaje: String) -> usize {
        for suscriptor in &mut self.suscriptores {
            suscriptor.enviar(mensaje.clone());
        }
        self.suscriptores.len()
    }

    pub fn desuscribirse(&mut self, suscriptor: Cliente) {
        let index = self.suscriptores.iter().position(|x| *x == suscriptor).unwrap();
        self.suscriptores.remove(index);
    }

    pub fn es_activo(&self) -> bool {
        self.suscriptores.len() > 1
    }

    pub fn len(&self) -> usize {
        self.suscriptores.len()
    }
}
