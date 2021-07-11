use crate::persistencia::{MensajePersistencia, Persistidor, PersistidorHandler};
use crate::valor::Valor;

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, PartialEq)]
pub enum ResultadoRedis {
    StrSimple(String),
    BulkStr(String),
    Int(isize),
    Vector(Vec<ResultadoRedis>),
    Nil,
    Error(String),
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum TipoRedis {
    Str(String),
    Lista(Vec<String>),
    Set(HashSet<String>),
}

pub struct BaseDeDatos {
    hashmap: HashMap<String, Valor>,
    persistidor: Persistidor,
    hilo: Option<JoinHandle<()>>,
    tx: Sender<MensajePersistencia>,
}

impl BaseDeDatos {
    pub fn new(archivo_persistencia: String) -> Self {
        let (tx, rx) = channel();
        let mut handler = PersistidorHandler::new(archivo_persistencia, 1, rx);

        let hilo_persistencia = thread::spawn(move || {
            handler.persistir();
        });

        BaseDeDatos {
            hashmap: HashMap::new(),
            persistidor: Persistidor::new(tx.clone()),
            hilo: Option::Some(hilo_persistencia),
            tx,
        }
    }

    pub fn obtener_valor(&self, clave: &str) -> Option<&TipoRedis> {
        match self.hashmap.get(clave) {
            Some(v) => v.get(),
            None => None,
        }
    }

    pub fn obtener_expiracion(&self, clave: &str) -> isize {
        match self.hashmap.get(clave) {
            Some(v) => v.obtener_expiracion(),
            None => -2,
        }
    }

    #[allow(dead_code)]
    pub fn guardar_valor_con_expiracion(
        &mut self,
        clave: String,
        expiracion: u64,
        valor: TipoRedis,
    ) {
        self.hashmap
            .insert(clave, Valor::expirable(valor, expiracion));

        self.persistirse();
    }

    pub fn actualizar_valor_con_expiracion(&mut self, clave: String, expiracion: u64) -> usize {
        match self.hashmap.get_mut(&clave) {
            Some(v) => {
                v.actualizar_expiracion(expiracion);
                1
            }
            None => 0,
        }
    }

    pub fn actualizar_valor_sin_expiracion(&mut self, clave: String) -> usize {
        match self.hashmap.get_mut(&clave) {
            Some(v) => {
                v.hacer_persistente();
                1
            }
            None => 0,
        }
    }

    pub fn guardar_valor(&mut self, clave: String, valor: TipoRedis) {
        self.hashmap.insert(clave, Valor::no_expirable(valor));

        self.persistirse();
    }

    pub fn guardar_valores(&mut self, parametros: Vec<String>) {
        let mut index = 0;
        while index != parametros.len() - 1 {
            let clave = &parametros[index];
            let valor = &parametros[index + 1];

            self.hashmap.insert(
                clave.to_string(),
                Valor::no_expirable(TipoRedis::Str(valor.to_string())),
            );

            index += 1;
        }
    }

    pub fn existe_clave(&mut self, clave: &str) -> bool {
        match self.hashmap.get(clave) {
            Some(v) => !v.expiro(),
            None => false,
        }
    }

    pub fn eliminar_clave(&mut self, clave: &str) -> usize {
        let valor = match self.hashmap.remove(clave) {
            Some(_) => 1,
            None => 0,
        };
        self.persistirse();
        valor
    }

    pub fn copiar_valor(&mut self, clave_actual: &str, clave_nueva: &str) -> Option<()> {
        let valor = match self.obtener_valor(clave_actual) {
            None => return None,
            Some(valor) => valor.clone(),
        };

        self.guardar_valor(clave_nueva.to_string(), valor);
        Some(())
    }

    pub fn actualizar_ultimo_acceso(&mut self, clave: String) -> isize {
        match self.hashmap.get_mut(&clave) {
            Some(v) => {
                v.actualizar_ultimo_acceso();
                1
            }
            None => 0,
        }
    }

    pub fn claves(&self, re: &str) -> Vec<String> {
        let regex = match Regex::new(re) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        self.hashmap
            .keys()
            .cloned()
            .into_iter()
            .filter(|c| regex.is_match(c))
            .collect()
    }

    fn persistirse(&self) {
        self.persistidor.persistir(self.hashmap.clone());
    }
}

impl Drop for BaseDeDatos {
    fn drop(&mut self) {
        self.tx.send(MensajePersistencia::Cerrar).unwrap();

        if let Some(hilo) = self.hilo.take() {
            if hilo.join().is_ok() {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn base_de_datos_devuelve_una_copia_de_un_elemento_almacenado() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));

        let valor = data_base.obtener_valor("clave");
        assert_eq!(&TipoRedis::Str("valor".to_string()), valor.unwrap());
    }

    #[test]
    fn base_de_datos_elimina_valor_almacenado() {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));

        assert!(data_base.existe_clave("clave"));

        data_base.eliminar_clave("clave");

        assert!(!data_base.existe_clave("clave"));
    }

    #[test]
    fn si_se_guarda_una_clave_que_expira_en_1_segundo_cuando_se_la_quiere_recuperar_no_se_encuentra(
    ) {
        let mut data_base = BaseDeDatos::new("eliminame.txt".to_string());
        data_base.guardar_valor_con_expiracion(
            "clave".to_string(),
            1,
            TipoRedis::Str("valor".to_string()),
        );

        thread::sleep(Duration::from_secs(2));

        assert_eq!(None, data_base.obtener_valor("clave"));
    }
}
