use std::{
    net::{
        TcpListener, TcpStream,
        IpAddr, Ipv4Addr, SocketAddr
    },
    thread,
    io::{Write, Read, BufReader, BufRead},
    path::PathBuf,
    fs::File,
    sync::mpsc::{
        channel, Receiver, Sender,
    } ,
};
use log::{info, error};

static PORT : u16 = 10497;

use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct LineServer {
    #[arg(short, long)]
    pub line_file: PathBuf,

    #[arg(short, long, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT))]
    bind_addr: SocketAddr,
}

enum Messages {
    Shutdown,
    Quit(i8),
}

impl LineServer {
    pub fn run(&self) -> Result<(), anyhow::Error> {
        let (tx, rx) : (Sender<Messages>, Receiver<Messages>) = channel();
        let rx = rx.into_iter();

        info!("Running server or {}", self.bind_addr);
        let listener = TcpListener::bind(self.bind_addr)?;
        let mut threads = Vec::new();
        for stream in listener.incoming() {
            let stream = stream?;
            let line_file = self.line_file.clone();
            let thread_tx = tx.clone();
            let thread = thread::spawn(move || {
                handle_client(stream, line_file, thread_tx)
            });
            threads.push(thread);
        }
        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    let args = LineServer::parse();
    env_logger::init();
    let _ = args.run()?;
    Ok(())
}

fn handle_client(mut stream: TcpStream, line_file: PathBuf, tx: Sender<Messages>) -> Result<(), anyhow::Error> {
    // TODO: This loads all lines into memory which might not be ideal.
    let file = File::open(&line_file)?;
    let file = BufReader::new(file);
    let lines : Vec<String> = file.lines().into_iter().collect::<Result<Vec<String>, std::io::Error>>()?;

    let mut command = String::new();
    let mut buffer = [0; 9];
    let mut commands : Vec<String> = Vec::new();
    loop {
        let len = stream.read(&mut buffer)?;
        let val = String::from_utf8(buffer[0..len].to_vec())?;
        if val.contains('\n') {
            for line in val.split('\n') {
                command.push_str(line.clone());
                commands.push(command);
                command = String::new();
            }
        }
        for command in &commands {
            if command.starts_with("GET") {
                let mut command = command.split(' ');
                if command.next() != Some("GET") {
                    error!("Command should start with GET");
                }
                if let Some(line_num) = command.next() {
                    info!("Retrieving line num {line_num}");
                    if let Ok(line_num) = line_num.parse::<u16>() {
                        if let Some(line) = lines.get(line_num as usize) {
                            stream.write(format!("Ok\r\n{line}\r\n").as_bytes())?;
                        } else {
                            stream.write("Err\r\n".as_bytes())?;
                        }
                    } else {
                        stream.write("Err\r\n".as_bytes())?;
                    }
                }
            } else if command.starts_with("SHUTDOWN") {
                // TODO: Make this quit the whole application.
                tx.send(Messages::Shutdown)?;
                return Ok(());
            } else if command.starts_with("QUIT") {
                tx.send(Messages::Quit(0))?;
                return Ok(())
            } else {
            }
        }
        if val.contains('\n') {
            commands = Vec::new();
        }
    }
}


#[test]
fn single_client_test() {
    env_logger::init();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT);
    let server = LineServer {
        line_file: PathBuf::from("./example.txt"),
        bind_addr: addr,
    };
    let thread = thread::spawn(move || {
        server.run()
    });

    let mut stream = TcpStream::connect(addr).expect("Failed to connect to server");

    stream.write("GET 1".as_bytes()).expect("Failed to write to socket");
    let mut buf = [0; 128];
    let len = stream.read(&mut buf).expect("Failed to read from stream");
    let lines = String::from_utf8(buf[0..len].to_vec()).expect("Failed to parse lines");
    assert_eq!("OK\r\nquick brown".to_string(), lines);
    stream.write("SHUTDOWN".as_bytes()).expect("Failed to write to socket");

    //let _ = thread.join().expect("Server ended in error!");


}
