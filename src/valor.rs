use crate::base_de_datos::TipoRedis;

use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Valor {
    valor: TipoRedis,
    momento_de_creacion: Instant,
    vida_util: Option<Duration>,
}

impl Valor {

    pub fn new(valor: TipoRedis) -> Self {
        Valor::no_expirable(valor)
    }

    pub fn expirable(valor: TipoRedis, vida_util: u64) -> Self {
        Valor {
            valor,
            momento_de_creacion: Instant::now(),
            vida_util: Some(Duration::from_secs(vida_util)),
        }
    }

    pub fn no_expirable(valor: TipoRedis) -> Self {
        Valor {
            valor,
            momento_de_creacion: Instant::now(),
            vida_util: None,
        }
    }

    pub fn expiro(&self) -> bool {
        match self.vida_util {
            Some(vida) => self.momento_de_creacion.elapsed() >= vida,
            None => false,
        }
    }

    pub fn get(&self) -> Option<&TipoRedis> {
        if !self.expiro() {
            Some(&self.valor)
        } else {
            None
        }
    }

    pub fn actualizar_expiracion(&mut self, nueva_expiracion: u64) {
        self.momento_de_creacion = Instant::now();
        self.vida_util = Some(Duration::from_secs(nueva_expiracion));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn cuando_se_crea_una_valor_no_expirable_este_no_expira_nunca(){
        let valor = Valor::no_expirable(TipoRedis::Str("miClave".to_string()));

        assert!(!valor.expiro());
    }

    #[test]
    fn cuando_se_espera_mas_tiempo_del_que_se_dijo_que_una_clave_expiraba_la_clave_efectivamente_esta_espirada() {
        let valor = Valor::expirable(TipoRedis::Str("miClave".to_string()), 1);

        thread::sleep(Duration::from_secs(2));

        assert!(valor.expiro());
    }

}
