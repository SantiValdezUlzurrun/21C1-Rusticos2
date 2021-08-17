use crate::observer::{Observable, Observer};

use crate::canal::Canal;
use crate::valor::Valor;

use regex::Regex;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]

/// Los posibles resultados que puede devolver un comando
pub enum ResultadoRedis {
    StrSimple(String),
    BulkStr(String),
    Int(isize),
    Vector(Vec<ResultadoRedis>),
    Nil,
    Error(String),
    Vacio,
}

#[derive(Debug, PartialEq, Clone)]
/// Los posibles tipos de datos que maneja el servidor redis
pub enum TipoRedis {
    Str(String),
    Lista(Vec<String>),
    Set(HashSet<String>),
    Canal(Canal),
}
/// Base de datos donde se almacenan todos los elementos almacenados
pub struct BaseDeDatos {
    hashmap: HashMap<String, Valor>,
    observadores: Vec<Box<dyn Observer + Send>>,
}

impl BaseDeDatos {
    /// Devuelve el valor que corresponde a la clave enviada por parametro
    pub fn obtener_valor(&self, clave: &str) -> Option<&TipoRedis> {
        match self.hashmap.get(clave) {
            Some(v) => v.get(),
            None => None,
        }
    }
    /// Devuelve el tiempo de expiracion de una clave almacenada en la base de datos
    pub fn obtener_expiracion(&self, clave: &str) -> isize {
        match self.hashmap.get(clave) {
            Some(v) => v.obtener_expiracion(),
            None => -2,
        }
    }
    /// Guarda una valor con un tiempo de expiracion enviado por parametro
    pub fn guardar_valor_con_expiracion(
        &mut self,
        clave: String,
        expiracion: u64,
        valor: TipoRedis,
    ) {
        self.hashmap
            .insert(clave, Valor::expirable(valor, expiracion));
        self.notificar_observadores(self.hashmap.clone());
    }
    /// Dada una clave con expiracion almacenada en la base de datos, actualiza su 'tiempo de vida' con el parametro 'expiracion'
    /// # Arguments
    ///
    /// * `self` - Referencia a la bases de datos
    /// * `clave` - Clave con la que se identifica un elemento almacenado en la base de datos
    /// * `expiracion` - Nuevo valor de expiracion de la clave
    ///
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

        self.notificar_observadores(self.hashmap.clone());
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
        self.notificar_observadores(self.hashmap.clone());
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
        self.notificar_observadores(self.hashmap.clone());
        valor
    }
    /// Dado un valor ya almacenado en la base de datos, lo copia en una nueva clave
    /// # Arguments
    ///
    /// * `self` - Referencia a la bases de datos
    /// * `clave_actual` - Clave almacenada en la base de datos
    /// * `clave_nueva` - Nuevo clave
    ///
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
    /// Devuelve todas las claves que matchean con un patron
    /// # Arguments
    ///
    /// * `self` - Referencia a la bases de datos
    /// * `re` - Patron de referencia
    ///
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
    /// Dado un elemento de tipo string, lo actulaliza con un nuevo valor
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
    /// Devuelve una lista con todos los canales activos de la base de datos
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

    pub fn borrar_claves(&mut self) {
        self.hashmap = HashMap::new();

        self.notificar_observadores(self.hashmap.clone());
    }

    pub fn cantidad_claves(&self) -> usize {
        self.hashmap.len()
    }

    pub fn info(&self) -> Vec<String> {
        let mut info = vec!["# Database".to_string(), "".to_string()];

        info.push(format!("cantidad de claves:{}", self.hashmap.len()));
        info.push(format!("capacidad:{}", self.hashmap.capacity()));

        info
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        BaseDeDatos {
            hashmap: HashMap::<String, Valor>::new(),
            observadores: vec![],
        }
    }

    pub fn new_con(tabla_persistida: HashMap<String, Valor>) -> Self {
        BaseDeDatos {
            hashmap: tabla_persistida,
            observadores: vec![],
        }
    }
}

impl Observable for BaseDeDatos {
    fn notificar_observadores(&self, bdd: HashMap<String, Valor>) {
        self.observadores
            .iter()
            .for_each(|o| o.actualizar(bdd.clone()))
    }

    fn agregar_observador(&mut self, o: Box<dyn Observer + Send>) {
        self.observadores.push(o);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn base_de_datos_devuelve_una_copia_de_un_elemento_almacenado() {
        let mut data_base = BaseDeDatos::new();
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));

        let valor = data_base.obtener_valor("clave");
        assert_eq!(&TipoRedis::Str("valor".to_string()), valor.unwrap());
    }

    #[test]
    fn base_de_datos_elimina_valor_almacenado() {
        let mut data_base = BaseDeDatos::new();
        data_base.guardar_valor("clave".to_string(), TipoRedis::Str("valor".to_string()));

        assert!(data_base.existe_clave("clave"));

        data_base.eliminar_clave("clave");

        assert!(!data_base.existe_clave("clave"));
    }

    #[test]
    fn si_se_guarda_una_clave_que_expira_en_1_segundo_cuando_se_la_quiere_recuperar_no_se_encuentra(
    ) {
        let mut data_base = BaseDeDatos::new();
        data_base.guardar_valor_con_expiracion(
            "clave".to_string(),
            1,
            TipoRedis::Str("valor".to_string()),
        );

        thread::sleep(Duration::from_secs(2));
        assert_eq!(None, data_base.obtener_valor("clave"));
    }
}
