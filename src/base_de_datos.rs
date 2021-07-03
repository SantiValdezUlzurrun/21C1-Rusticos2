use crate::persistencia::{MensajePersistencia, Persistidor, PersistidorHandler};

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, PartialEq)]
pub enum ResultadoRedis {
    StrSimple(String),
    BulkStr(String),
    Int(usize),
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
    hashmap: HashMap<String, TipoRedis>,
    persistidor: Persistidor,
    hilo: Option<JoinHandle<()>>,
    tx: Sender<MensajePersistencia>,
}

impl BaseDeDatos {
    pub fn new(archivo_persistencia: String) -> Self {
        let (tx, rx) = channel();
        let handler = PersistidorHandler::new(archivo_persistencia, 1, rx);

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
        self.hashmap.get(clave)
    }

    pub fn guardar_valor(&mut self, clave: String, valor: TipoRedis) {
        self.hashmap.insert(clave, valor);

        self.persistirse();
    }

    pub fn guardar_valores(&mut self, parametros: Vec<String>) {
        let mut index = 0;
        while index != parametros.len() - 1{
            let clave = &parametros[index];
            let valor = &parametros[index + 1];

            self.hashmap.insert(clave.to_string(), TipoRedis::Str(valor.to_string()));

            index+=1;
        }
    }

    pub fn existe_clave(&mut self, clave: &str) -> bool {
        self.hashmap.contains_key(clave)
    }

    #[allow(dead_code)]
    pub fn cant_claves(&mut self) -> usize {
        self.hashmap.len()
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
}
