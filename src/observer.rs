use crate::valor::Valor;
use std::collections::HashMap;

/// Representa a una entidad observable que se encargara de notificar a sus observadores
pub trait Observable {
    fn notificar_observadores(&self, bdd: HashMap<String, Valor>);
    fn agregar_observador(&mut self, o: Box<dyn Observer + Send>);
}

/// Representa a una entidad observadora que se actualizara al ser notificada
pub trait Observer {
    fn actualizar(&self, bdd: HashMap<String, Valor>);
}
