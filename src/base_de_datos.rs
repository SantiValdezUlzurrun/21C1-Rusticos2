use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::iter::FromIterator;

use crate::canal::Canal;
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

#[derive(Debug, PartialEq, Clone)]
pub enum TipoRedis {
    Str(String),
    Lista(Vec<String>),
    Set(HashSet<String>),
    Canal(Canal),
}

pub struct BaseDeDatos {
    hashmap: HashMap<String, Valor>,
    persistidor: Persistidor,
    hilo: Option<JoinHandle<()>>,
    tx: Sender<MensajePersistencia>,
}

impl BaseDeDatos {
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

    pub fn intercambiar_valor(
        &mut self,
        clave: String,
        valor_nuevo: TipoRedis,
    ) -> Option<TipoRedis> {
        let valor = match self.obtener_valor(&clave) {
            Some(TipoRedis::Lista(_)) => return Some(TipoRedis::Lista(vec![])),
            Some(TipoRedis::Set(_)) => return Some(TipoRedis::Set(HashSet::new())),
            Some(TipoRedis::Str(valor)) => Some(TipoRedis::Str(valor.to_string())),
            Some(TipoRedis::Canal(_)) => None,
            None => None,
        };

        self.hashmap.insert(clave, Valor::no_expirable(valor_nuevo));
        valor
    }

    pub fn canales_activos(&self, re: &str) -> Vec<String> {
        let mut canales: Vec<String> = Vec::new();
        let claves = self.claves(re);
        for clave in &claves {
            let canal = match self.obtener_valor(clave) {
                Some(TipoRedis::Canal(c)) => c,
                _ => continue,
            };
            if canal.es_activo() {
                canales.push(clave.to_string());
            }
        }
        canales
    }

    fn persistirse(&self) {
        self.persistidor.persistir(self.hashmap.clone());
    }
    pub fn new(archivo_persistencia: String) -> Self {
        let (tx, rx) = channel();
        let mut handler = PersistidorHandler::new(archivo_persistencia, 1, rx);

        let hilo_persistencia = thread::spawn(move || {
            handler.persistir();
        });

        BaseDeDatos {
            hashmap: HashMap::<String, Valor>::new(),
            persistidor: Persistidor::new(tx.clone()),
            hilo: Option::Some(hilo_persistencia),
            tx,
        }
    }
    #[allow(dead_code)]
    pub fn new_con_persistencia(archivo_persistencia: String) -> Self {
        let (tx, rx) = channel();
        let mut handler = PersistidorHandler::new(archivo_persistencia.clone(), 1, rx);

        let hilo_persistencia = thread::spawn(move || {
            handler.persistir();
        });

        let mut hashmap = HashMap::<String, Valor>::new();

        let archivo = match File::open(archivo_persistencia) {
            Ok(archivo) => archivo,
            Err(_) => {
                return BaseDeDatos {
                    hashmap: HashMap::new(),
                    persistidor: Persistidor::new(tx.clone()),
                    hilo: Option::Some(hilo_persistencia),
                    tx,
                }
            }
        };

        let reader = BufReader::new(archivo);
        let mut lineas = reader.lines();
        while let Some(Ok(line)) = lineas.next() {
            let mut elemento: Vec<&str> = line.split(':').collect();

            if elemento.contains(&"STRING") {
                let mut valor = Valor::no_expirable(TipoRedis::Str(elemento[2].to_string()));

                if es_expirable(elemento.clone()) {
                    let tiempo = obtener_tiempo_expiracion(elemento.clone(), "EX").unwrap_or(0);
                    valor = Valor::expirable(TipoRedis::Str(elemento[2].to_string()), tiempo);
                }
                hashmap.insert(elemento[1].to_string(), valor);
            } else if elemento.remove(0) == "LIST" {
                let clave = elemento.remove(0).to_string();
                let mut valor = Valor::no_expirable(TipoRedis::Lista(
                    elemento.iter().map(|x| x.to_string()).collect(),
                ));

                if es_expirable(elemento.clone()) {
                    let tiempo = obtener_tiempo_expiracion(elemento.clone(), "EX").unwrap_or(0);

                    valor = Valor::expirable(
                        TipoRedis::Lista(elemento.iter().map(|x| x.to_string()).collect()),
                        tiempo,
                    );
                }
                hashmap.insert(clave, valor);
            } else {
                elemento.remove(0);
                let clave = elemento.remove(0).to_string();
                let mut valor = Valor::no_expirable(TipoRedis::Set(HashSet::from_iter(
                    elemento
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                )));
                if es_expirable(elemento.clone()) {
                    let tiempo = obtener_tiempo_expiracion(elemento.clone(), "EX").unwrap_or(0);

                    valor = Valor::expirable(
                        TipoRedis::Set(HashSet::from_iter(
                            elemento
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>(),
                        )),
                        tiempo,
                    );
                }
                hashmap.insert(clave, valor);
            }
        }
        BaseDeDatos {
            hashmap,
            persistidor: Persistidor::new(tx.clone()),
            hilo: Option::Some(hilo_persistencia),
            tx,
        }
    }
}

fn es_expirable(parametros: Vec<&str>) -> bool {
    parametros.contains(&"EX")
}
fn obtener_tiempo_expiracion(parametros: Vec<&str>, support: &str) -> Option<u64> {
    match parametros.rsplit(|p| p == &support.to_string()).next() {
        Some(c) => match c[0].parse::<u64>() {
            Ok(num) => Some(num),
            Err(_) => None,
        },
        None => None,
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
