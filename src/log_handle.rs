use std::io::Write;
use std::fs::OpenOptions;
use std::fs;

struct LogHandle {
    file_name: String
}

fn crear_log(accion: Vec<&str>) -> String{

    if accion.len() == 2{
        "Succesfull --> ".to_owned()+&"Client: ".to_owned() + accion[1] + " --> Send: " + accion[0]
   
    } else if accion.len() == 3{
        accion[2].to_owned() + " --> " + &"Client: ".to_owned() + accion[1] + &" --> Send: ".to_owned() + accion[0]

    }else{
        "ErrorLogFormat".to_string()
    }
}

impl LogHandle {
    
    fn new(path: String) -> LogHandle{
        LogHandle{
            file_name: path
        }
    }

    fn escribir_accion(&self, accion: Vec<&str>){

        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&self.file_name)
        .unwrap();

        let log = crear_log(accion);
    if let Err(e) = writeln!(file, "{}", log) {
        eprintln!("Error al escribir log_file: {}", e);
    }

    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_handle_guarda_info_del_comando_ejecutado(){
        
        let log_handle = LogHandle::new("log_test".to_string());
        let accion = vec!["PING","124.23.12.53"];

        log_handle.escribir_accion(accion);
       
        let contents = fs::read_to_string("log_test").unwrap();

        assert_eq!(contents, "Succesfull --> Client: 124.23.12.53 --> Send: PING\n");
	}

    #[test]
    fn log_handle_guarda_info_del_comando_ejecutado_que_dio_erro(){
        
        let log_handle = LogHandle::new("log_test".to_string());
        let accion = vec!["GET","124.23.12.53","ParseError"];
        
        log_handle.escribir_accion(accion);

        let contents = fs::read_to_string("log_test").unwrap();
        
        assert_eq!(contents, "Succesfull --> Client: 124.23.12.53 --> Send: PING\nParseError --> Client: 124.23.12.53 --> Send: GET\n");
    }
}