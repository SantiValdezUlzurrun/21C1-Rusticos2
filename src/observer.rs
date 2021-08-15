use crate::valor::Valor;
use std::collections::HashMap;

pub trait Observable {
    fn notificar_observadores(&self, bdd: HashMap<String, Valor>);
    fn agregar_observador(&mut self, o: Box<dyn Observer + Send>);
}

pub trait Observer {
    fn actualizar(&self, bdd: HashMap<String, Valor>);
}
