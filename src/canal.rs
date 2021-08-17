use crate::base_de_datos::ResultadoRedis;
use crate::cliente::Cliente;
use crate::redis_error::RedisError;

/// Representa un canal donde se pueden suscribir clientes y publicar mensajes
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

    pub fn suscribirse(&mut self, suscriptor: Cliente) {
        if self.notificar_suscripcion(suscriptor.clone()).is_ok() {
            self.suscriptores.push(suscriptor)
        }
    }

    pub fn publicar(&mut self, mensaje: String) -> usize {
        let mut publicados: usize = 0;
        let resultado = ResultadoRedis::Vector(vec![
            ResultadoRedis::BulkStr("message".to_string()),
            ResultadoRedis::BulkStr(self.nombre.clone()),
            ResultadoRedis::BulkStr(mensaje),
        ]);
        for suscriptor in &mut self.suscriptores {
            match suscriptor.enviar_resultado(&resultado) {
                Ok(_) => publicados += 1,
                Err(_) => continue,
            }
        }
        publicados
    }

    pub fn desuscribirse(&mut self, suscriptor: Cliente) {
        let index = match self
            .suscriptores
            .iter()
            .position(|x| *x == suscriptor.clone())
        {
            Some(i) => i,
            None => return,
        };

        self.suscriptores.remove(index);
        if self.notificar_desubscripcion(suscriptor).is_ok() {}
    }

    pub fn es_activo(&self) -> bool {
        self.suscriptores.len() > 1
    }

    pub fn len(&self) -> usize {
        self.suscriptores.len()
    }

    fn notificar_suscripcion(&self, mut suscriptor: Cliente) -> Result<(), RedisError> {
        let cant = self.suscriptores.len() + 1;
        let resultado = ResultadoRedis::Vector(vec![
            ResultadoRedis::BulkStr("subscribe".to_string()),
            ResultadoRedis::BulkStr(self.nombre.clone()),
            ResultadoRedis::Int(cant as isize),
        ]);
        suscriptor.enviar_resultado(&resultado)
    }
    fn notificar_desubscripcion(&self, mut suscriptor: Cliente) -> Result<(), RedisError> {
        let resultado = ResultadoRedis::Vector(vec![
            ResultadoRedis::BulkStr("unsubscribe".to_string()),
            ResultadoRedis::BulkStr(self.nombre.clone()),
            ResultadoRedis::Int(self.suscriptores.len() as isize),
        ]);
        suscriptor.enviar_resultado(&resultado)
    }
}
