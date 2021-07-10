use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use crate::base_de_datos::TipoRedis;
use crate::valor::Valor;

const SEPARADOR: &str = "\\r\\n";
const FORMATO_GET: &str = "*3\\r\\n$3\\r\\nSET\\r\\n";
const FORMATO_LPUSH: &str = "$4\\r\\nLPUSH\\r\\n";
const ID_ARG: &str = "*";
const ID_TAM_STR: &str = "$";

pub enum MensajePersistencia {
    Info(HashMap<String, Valor>),
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
                            vector.push(guardar_clave_valor(key.to_string(), val.get()));
                        }
                        guardar_en_archivo(&self.archivo, vector);
                        self.instante = Instant::now();
                    }
                }

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
        self.persistidor
            .send(MensajePersistencia::Info(base_de_datos))
            .unwrap();
    }
}

fn guardar_elemento(elemento: &str) -> String {
    let len_elemento = elemento.len();
    ID_TAM_STR.to_string() + &len_elemento.to_string() + SEPARADOR + elemento + SEPARADOR
}

fn guardar_cant_arg(lista: &[String]) -> String {
    let cant_arg = lista.len() + 2;
    ID_ARG.to_string() + &cant_arg.to_string() + SEPARADOR
}

fn guardar_clave_valor(clave: String, valor: Option<&TipoRedis>) -> String {
    match valor {
        Some(TipoRedis::Str(valor)) => {
            FORMATO_GET.to_string() + &guardar_elemento(&clave) + &guardar_elemento(&valor)
        }

        Some(TipoRedis::Lista(lista)) => {
            let mut string_comando =
                guardar_cant_arg(&lista) + FORMATO_LPUSH + &guardar_elemento(&clave);
            for valor in lista.iter() {
                string_comando += &guardar_elemento(valor);
            }
            string_comando
        }
        _ => String::new(),
    }
}

fn guardar_en_archivo(archivo: &str, instrucciones: Vec<String>) {
    let mut archivo = OpenOptions::new()
        .write(true)
        .create(true)
        .open(archivo)
        .unwrap();

    for instruccion in instrucciones.iter() {
        if let Err(e) = writeln!(archivo, "{}", instruccion) {
            println!("{:?}", e);
        }
    }
}

#[allow(dead_code)]
fn cargar_en_vector(archivo: &str) -> Vec<String> {
    let mut vector: Vec<String> = vec![];
    let file = File::open(archivo).unwrap();
    let reader = BufReader::new(file);

    for linea in reader.lines().flatten() {
        vector.push(linea);
    }
    vector
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
            vector.push(guardar_clave_valor(key.to_string(), val.get()));
        }

        assert!(vector.contains(&String::from(
            "*3\\r\\n$3\\r\\nSET\\r\\n$9\\r\\nUnaClave1\\r\\n$7\\r\\nUnValor\\r\\n"
        )));
        assert!(vector.contains(&String::from(
            "*3\\r\\n$3\\r\\nSET\\r\\n$9\\r\\nUnaClave2\\r\\n$7\\r\\nUnValor\\r\\n"
        )));
        assert!(vector.contains(&String::from(
            "*3\\r\\n$3\\r\\nSET\\r\\n$9\\r\\nUnaClave3\\r\\n$7\\r\\nUnValor\\r\\n"
        )));
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

            TipoRedis::Str(_) => {}
            _ => {}
        }

        map.insert("milista", Valor::no_expirable(lista));

        let mut vector: Vec<String> = vec![];
        for (key, val) in map.iter() {
            vector.push(guardar_clave_valor(key.to_string(), val.get()));
        }

        assert!(vector.contains(&String::from(
            "*3\\r\\n$3\\r\\nSET\\r\\n$9\\r\\nUnaClave1\\r\\n$7\\r\\nUnValor\\r\\n"
        )));
        assert!(vector.contains(&String::from(
            "*3\\r\\n$3\\r\\nSET\\r\\n$9\\r\\nUnaClave2\\r\\n$7\\r\\nUnValor\\r\\n"
        )));
        assert!(vector.contains(&String::from("*5\\r\\n$4\\r\\nLPUSH\\r\\n$7\\r\\nmilista\\r\\n$12\\r\\nPRIMER_VALOR\\r\\n$13\\r\\nSEGUNDO_VALOR\\r\\n$12\\r\\nTERCER_VALOR\\r\\n")));
    }
}
