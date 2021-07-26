use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use crate::base_de_datos::TipoRedis;
use crate::valor::Valor;

const STRING: &str = "STRING";
const LIST: &str = "LIST";
const SET: &str = "SET";
const EX: &str = "EX";
const SEPARADOR: &str = ":";

pub enum MensajePersistencia {
    Info(HashMap<String, Valor>),
    ArchivoAPersistir(String),
    Cerrar,
}

pub struct PersistidorHandler {
    archivo: String,
    intervalo: Duration,
    instante: Instant,
    receptor: Receiver<MensajePersistencia>,
}

impl PersistidorHandler {
    pub fn new(archivo: String, intervalo: u64, receptor: Receiver<MensajePersistencia>) -> Self {
        PersistidorHandler {
            archivo,
            receptor,
            instante: Instant::now(),
            intervalo: Duration::from_secs(intervalo),
        }
    }

    pub fn persistir(&mut self) {
        while let Ok(mensaje) = self.receptor.recv() {
            match mensaje {
                MensajePersistencia::Info(a_persistir) => {
                    if self.instante.elapsed() >= self.intervalo {
                        //persisto
                        let mut vector: Vec<String> = vec![];
                        for (key, val) in a_persistir.iter() {
                            vector.push(guardar_clave_valor(
                                key.to_string(),
                                val.get(),
                                val.get_tiempo(),
                            ));
                        }
                        match guardar_en_archivo(&self.archivo, vector) {
                            Ok(_) => (),
                            Err(_) => break,
                        };
                        self.instante = Instant::now();
                    }
                }

                MensajePersistencia::ArchivoAPersistir(a) => self.archivo = a,

                MensajePersistencia::Cerrar => break,
            };
        }
    }
}

pub struct Persistidor {
    persistidor: Sender<MensajePersistencia>,
}

impl Persistidor {
    pub fn new(persistidor: Sender<MensajePersistencia>) -> Self {
        Persistidor { persistidor }
    }

    pub fn persistir(&self, base_de_datos: HashMap<String, Valor>) {
        if self
            .persistidor
            .send(MensajePersistencia::Info(base_de_datos))
            .is_ok()
        {}
    }

    pub fn cambiar_archivo(&self, ruta_nueva: String) {
        if self
            .persistidor
            .send(MensajePersistencia::ArchivoAPersistir(ruta_nueva))
            .is_ok()
        {}
    }
}

fn guardar_clave_valor(clave: String, valor: Option<&TipoRedis>, time: Option<Duration>) -> String {
    match (valor, time) {
        (Some(TipoRedis::Str(valor)), Some(duration)) => {
            STRING.to_string()
                + SEPARADOR
                + &clave
                + SEPARADOR
                + valor
                + SEPARADOR
                + EX
                + SEPARADOR
                + &(duration.as_secs().to_string())
        }

        (Some(TipoRedis::Str(valor)), None) => {
            STRING.to_string() + SEPARADOR + &clave + SEPARADOR + valor
        }

        (Some(TipoRedis::Lista(lista)), Some(duration)) => {
            let mut persistencia_lista = LIST.to_string() + SEPARADOR + &clave;
            for valor in lista.iter() {
                persistencia_lista += &(SEPARADOR.to_string() + valor);
            }
            persistencia_lista +=
                &(SEPARADOR.to_string() + EX + SEPARADOR + &(duration.as_secs().to_string()));
            persistencia_lista
        }

        (Some(TipoRedis::Lista(lista)), None) => {
            let mut persistencia_lista = LIST.to_string() + SEPARADOR + &clave;
            for valor in lista.iter() {
                persistencia_lista += &(SEPARADOR.to_string() + valor);
            }
            persistencia_lista
        }

        (Some(TipoRedis::Set(set)), Some(duration)) => {
            let mut persistencia_set = SET.to_string() + SEPARADOR + &clave;
            for valor in set.iter() {
                persistencia_set += &(SEPARADOR.to_string() + valor);
            }
            persistencia_set +=
                &(SEPARADOR.to_string() + EX + SEPARADOR + &(duration.as_secs().to_string()));
            persistencia_set
        }
        (Some(TipoRedis::Set(set)), None) => {
            let mut persistencia_set = SET.to_string() + SEPARADOR + &clave;
            for valor in set.iter() {
                persistencia_set += &(SEPARADOR.to_string() + valor);
            }
            persistencia_set
        }
        _ => String::new(),
    }
}

fn guardar_en_archivo(archivo: &str, instrucciones: Vec<String>) -> Result<()> {
    let mut archivo = match OpenOptions::new().write(true).create(true).open(archivo) {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    for instruccion in instrucciones.iter() {
        if let Err(e) = writeln!(archivo, "{}", instruccion) {
            println!("{:?}", e);
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn cargar_en_vector(archivo: &str) -> Result<Vec<String>> {
    let mut vector: Vec<String> = vec![];
    let file = match File::open(archivo) {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    let reader = BufReader::new(file);

    for linea in reader.lines().flatten() {
        vector.push(linea);
    }
    Ok(vector)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn inserto_varios_strings_en_hash_map_y_guardar_clave_valor_devuelve_el_mensaje_para_volver_a_cargarlos(
    ) {
        let mut map = HashMap::new();
        map.insert(
            "UnaClave1",
            Valor::no_expirable(TipoRedis::Str("UnValor".to_string())),
        );
        map.insert(
            "UnaClave2",
            Valor::no_expirable(TipoRedis::Str("UnValor".to_string())),
        );
        map.insert(
            "UnaClave3",
            Valor::no_expirable(TipoRedis::Str("UnValor".to_string())),
        );

        let mut vector: Vec<String> = vec![];
        for (key, val) in map.iter() {
            vector.push(guardar_clave_valor(
                key.to_string(),
                val.get(),
                val.get_tiempo(),
            ));
        }
        assert!(vector.contains(&"STRING:UnaClave1:UnValor".to_string()));
        assert!(vector.contains(&"STRING:UnaClave2:UnValor".to_string()));
        assert!(vector.contains(&"STRING:UnaClave3:UnValor".to_string()));
    }

    #[test]
    fn inserto_varios_tipo_redis_en_hash_map_y_guardar_clave_valor_devuelve_el_mensaje_para_volver_a_cargarlos(
    ) {
        let mut map = HashMap::new();
        map.insert(
            "UnaClave1",
            Valor::no_expirable(TipoRedis::Str("UnValor".to_string())),
        );
        map.insert(
            "UnaClave2",
            Valor::no_expirable(TipoRedis::Str("UnValor".to_string())),
        );

        let mut lista = TipoRedis::Lista(Vec::new());

        match lista {
            TipoRedis::Lista(ref mut lista) => {
                lista.push("PRIMER_VALOR".to_string());
                lista.push("SEGUNDO_VALOR".to_string());
                lista.push("TERCER_VALOR".to_string());
            }
            _ => {}
        }

        map.insert("milista", Valor::no_expirable(lista));

        let mut vector: Vec<String> = vec![];
        for (key, val) in map.iter() {
            vector.push(guardar_clave_valor(
                key.to_string(),
                val.get(),
                val.get_tiempo(),
            ));
        }
        assert!(vector.contains(&"STRING:UnaClave1:UnValor".to_string()));
        assert!(vector.contains(&"STRING:UnaClave2:UnValor".to_string()));
        assert!(
            vector.contains(&"LIST:milista:PRIMER_VALOR:SEGUNDO_VALOR:TERCER_VALOR".to_string())
        );
    }

    #[test]
    fn inserto_varios_strings_con_persistencia_en_hash_map_y_guardar_clave_valor_devuelve_el_mensaje_para_volver_a_cargarlos(
    ) {
        let mut map = HashMap::new();
        map.insert(
            "UnaClave1",
            Valor::expirable(TipoRedis::Str("UnValor".to_string()), 3000),
        );
        map.insert(
            "UnaClave2",
            Valor::expirable(TipoRedis::Str("UnValor".to_string()), 3000),
        );
        map.insert(
            "UnaClave3",
            Valor::expirable(TipoRedis::Str("UnValor".to_string()), 3000),
        );

        let mut lista = TipoRedis::Lista(Vec::new());

        match lista {
            TipoRedis::Lista(ref mut lista) => {
                lista.push("PRIMER_VALOR".to_string());
                lista.push("SEGUNDO_VALOR".to_string());
                lista.push("TERCER_VALOR".to_string());
            }
            _ => {}
        }

        map.insert("milista", Valor::expirable(lista, 4500));

        let mut vector: Vec<String> = vec![];
        for (key, val) in map.iter() {
            vector.push(guardar_clave_valor(
                key.to_string(),
                val.get(),
                val.get_tiempo(),
            ));
        }
        assert!(vector.contains(&"STRING:UnaClave1:UnValor:EX:3000".to_string()));
        assert!(vector.contains(&"STRING:UnaClave2:UnValor:EX:3000".to_string()));
        assert!(vector.contains(&"STRING:UnaClave3:UnValor:EX:3000".to_string()));
        assert!(vector
            .contains(&"LIST:milista:PRIMER_VALOR:SEGUNDO_VALOR:TERCER_VALOR:EX:4500".to_string()));
    }
}
