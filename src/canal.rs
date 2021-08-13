use crate::cliente::Cliente;

#[derive(Debug, PartialEq, Clone)]
pub struct Canal {
    nombre: String,
    suscriptores: Vec<Cliente>,
}

impl Canal {
    pub fn new(nombre: String) -> Self {
        Canal {
            nombre,
            suscriptores: Vec::new(),
        }
    }

    pub fn suscribirse(&mut self,mut suscriptor: Cliente) {
        suscriptor.notificar_subscripcion(self.nombre.clone());
        self.suscriptores.push(suscriptor);
    }

    pub fn publicar(&mut self, mensaje: String) -> usize {
        let mut publicados: usize = 0;
        for suscriptor in &mut self.suscriptores {
            match suscriptor.publicar(self.nombre.clone(), mensaje.clone()) {
                Ok(_) => publicados += 1,
                Err(_) => continue,
            }
        }
        publicados
    }

    pub fn desuscribirse(&mut self, mut suscriptor: Cliente) {
        let index = match self
            .suscriptores
            .iter()
            .position(|x| *x == suscriptor.clone())
        {
            Some(i) => i,
            None => return,
        };

        self.suscriptores.remove(index);
        suscriptor.notificar_desubscripcion(self.nombre.clone());
    }

    pub fn es_activo(&self) -> bool {
        self.suscriptores.len() > 1
    }

    pub fn len(&self) -> usize {
        self.suscriptores.len()
    }
}
