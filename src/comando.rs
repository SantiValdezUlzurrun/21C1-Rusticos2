use std::collections::HashMap;

pub enum ResultadoRedis {
    Str(String),
    //Int(u32),
    //Vector(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    GetError,
}

pub trait Command {
    fn ejecutar(
        &self,
        hash_map: &mut HashMap<String, String>,
    ) -> Result<ResultadoRedis, CommandError>;
}

struct SetCommand {
    clave: String,
    valor: String,
}

impl Command for SetCommand {
    fn ejecutar(
        &self,
        hash_map: &mut HashMap<String, String>,
    ) -> Result<ResultadoRedis, CommandError> {
        hash_map.insert(self.clave.clone(), self.valor.clone());
        Ok(ResultadoRedis::Str("OK".to_string()))
    }
}

impl SetCommand {
    fn new(clave: String, valor: String) -> SetCommand {
        SetCommand { clave, valor }
    }
}

struct GetCommand {
    clave: String,
}

impl Command for GetCommand {
    fn ejecutar(
        &self,
        hash_map: &mut HashMap<String, String>,
    ) -> Result<ResultadoRedis, CommandError> {
        match hash_map.get(&self.clave) {
            None => Err(CommandError::GetError),
            Some(valor) => Ok(ResultadoRedis::Str(valor.to_string())),
        }
    }
}
impl GetCommand {
    fn new(clave: String) -> GetCommand {
        GetCommand { clave }
    }
}

pub fn crear_comando(arg_vec: &[String]) -> Box<dyn Command> {
    if *"SET" == arg_vec[0] {
        Box::new(SetCommand::new(
            arg_vec[1].to_string(),
            arg_vec[2].to_string(),
        ))
    } else {
        Box::new(GetCommand::new(arg_vec[1].to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fabrica_crea_un_set_comand_guardo_un_valor() {
        let mut hash_map = HashMap::<String, String>::new();

        let arg_vec = vec![
            "SET".to_string(),
            "una_clave".to_string(),
            "un_valor".to_string(),
        ];
        let set = crear_comando(&arg_vec);
        set.ejecutar(&mut hash_map).unwrap();

        assert!(hash_map.contains_key(&"una_clave".to_string()));
    }

    #[test]
    fn fabrica_crea_set_comand_guardo_dos_valores_con_la_misma_clave() {
        let mut hash_map = HashMap::<String, String>::new();

        //primer comando get//
        let arg_vec1 = vec![
            "SET".to_string(),
            "una_clave".to_string(),
            "un_valor".to_string(),
        ];
        let set1 = crear_comando(&arg_vec1);
        set1.ejecutar(&mut hash_map).unwrap();

        //segundo comando get//
        let arg_vec2 = vec![
            "SET".to_string(),
            "una_clave".to_string(),
            "otro_valor".to_string(),
        ];
        let set2 = crear_comando(&arg_vec2);
        set2.ejecutar(&mut hash_map).unwrap();

        assert_eq!(
            hash_map.get(&"una_clave".to_string()).unwrap(),
            &"otro_valor".to_string()
        );
    }
}