use std::net::TcpStream;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;


pub struct Redis {
    direccion : String
}

impl Redis {
    
    pub fn new(host: &str, port: &str) -> Self {
        Redis {
            direccion: host.to_string() + ":" + port,
        }
    }
    
    pub fn iniciar(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.direccion)?;

        for stream in listener.incoming() {
            if let Ok(mut socket) = stream {
                
                let reader = BufReader::new(socket.try_clone()?);
                let mut lines = reader.lines();
                
                while let Some(line) = lines.next() {
                    println!("Recibido: {:?}", line);
                    
                    self.manejar_comando(line?, socket.try_clone()?); 
                }
            }
        }
        Ok(())
    }

    fn manejar_comando(&self, entrada: String, mut stream: TcpStream) -> std::io::Result<()>{
        if "ping" == entrada {
            stream.write("PONG".as_bytes())?;
            stream.write("\n".as_bytes())?;
        }
        Ok(())
    }
}
