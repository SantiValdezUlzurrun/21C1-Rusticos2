use crate::base_de_datos::TipoRedis;

use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Valor {
    valor: TipoRedis,
    momento_de_creacion: Instant,
    vida_util: Duration,
}

impl Valor {

    pub fn new(valor: TipoRedis, vida_util: u64) -> Self {
        Valor {
            valor,
            momento_de_creacion: Instant::now(),
            vida_util: Duration::from_secs(vida_util),
        }
    }

    pub fn expiro(&self) -> bool {
        if self.vida_util == Duration::from_secs(0) {
            return false;
        } else {
            return self.momento_de_creacion.elapsed() >= self.vida_util;
        }
    }

    pub fn get(&self) -> Option<&TipoRedis> {
        if !self.expiro() {
            Some(&self.valor)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn cuando_se_crea_una_valor_con_duracion_cero_esta_no_expira_nunca(){
        let valor = Valor::new(TipoRedis::Str("miClave".to_string()), 0);

        assert!(!valor.expiro());
    }

    #[test]
    fn cuando_se_espera_mas_tiempo_del_que_se_dijo_que_una_clave_expiraba_la_clave_efectivamente_esta_espirada() {
        let valor = Valor::new(TipoRedis::Str("miClave".to_string()), 1);

        thread::sleep(Duration::from_secs(2));

        assert!(valor.expiro());
    }

}
