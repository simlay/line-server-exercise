#[cfg(test)]
mod tests;

use clap::Parser;
use log::{debug, error, info};
use std::{
    collections::HashMap,
    fs::File,
    io::BufRead,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast::{channel, Receiver, Sender},
    task::JoinHandle,
};

pub static DEFAULT_PORT: u16 = 10497;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct LineServer {
    #[arg(short, long)]
    pub line_file: PathBuf,

    #[arg(short, long, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), DEFAULT_PORT))]
    pub bind_addr: SocketAddr,
}

#[derive(Clone, Debug)]
enum Messages {
    Shutdown,
    Quit(u16),
}

impl LineServer {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let (tx, mut rx): (Sender<Messages>, Receiver<Messages>) = channel(1000);

        info!("Running server or {}", self.bind_addr);
        let listener = TcpListener::bind(self.bind_addr).await?;
        let mut threads: HashMap<u16, JoinHandle<_>> = HashMap::new();
        let mut client_id: u16 = 0;
        let file = File::open(&self.line_file)?;
        let file = std::io::BufReader::new(file);
        let lines: Vec<String> = file
            .lines()
            .collect::<Result<Vec<String>, std::io::Error>>()?;
        let lines = Arc::new(lines);
        loop {
            tokio::select! {
                client = listener.accept() => {
                    let (stream, addr) = client?;
                    info!("New client from {addr}");

                    let thread_tx = tx.clone();
                    let lines = lines.clone();
                    let handle = tokio::spawn(async move {
                        handle_client(stream, thread_tx, client_id, lines).await
                    });
                    debug!("Spawned thread");
                    threads.insert(client_id, handle);
                    client_id += 1;
                },
                msg = rx.recv() => {
                    let msg = msg?;
                    match msg {
                        Messages::Shutdown => {
                            return Ok(());
                        },
                        Messages::Quit(ref client_id) => {
                            if let Some(thread) = threads.remove(client_id) {
                                thread.await??;
                            } else {
                                error!("{client_id} not in thread dictionary! This is a bug!");
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn handle_client(
    stream: TcpStream,
    tx: Sender<Messages>,
    client_id: u16,
    lines: Arc<Vec<String>>,
) -> Result<(), anyhow::Error> {
    debug!("Reading file");

    let mut command = String::new();
    let mut stream = BufReader::new(stream);
    loop {
        debug!("Waiting for new comands");
        stream.read_line(&mut command).await?;

        command = command
            .strip_suffix('\n')
            .unwrap_or("")
            .to_string()
            .to_uppercase();

        // This could probably be better done with a Serde enum.
        if command.starts_with("GET") {
            let mut command = command.split(' ');
            if command.next() != Some("GET") {
                error!("Command should start with GET");
            }
            if let Some(line_num_string) = command.next() {
                info!("Retrieving line num {line_num_string}");
                match line_num_string.parse::<u16>() {
                    Ok(line_num) => {
                        if let Some(line) = lines.get(line_num as usize) {
                            stream
                                .write_all(format!("Ok\r\n{line}\r\n").as_bytes())
                                .await?;
                        } else {
                            stream.write_all(
                                format!("Err - failed to retrieve line {line_num}. There are only {} lines available.\r\n", lines.len()).as_bytes()).await?;
                        }
                    }
                    Err(e) => {
                        stream.write_all(format!("Err - {e}. Is {line_num_string} an unsigned integer under 65536?\r\n").as_bytes()).await?;
                    }
                }
            }
        } else if command.starts_with("SHUTDOWN") {
            tx.send(Messages::Shutdown)?;
            stream.shutdown().await?;
            return Ok(());
        } else if command.starts_with("QUIT") {
            stream.shutdown().await?;
            tx.send(Messages::Quit(client_id))?;
            return Ok(());
        } else {
            let _ = stream.write(
               format!("Err - {command} is an invalid command. `GET nnnn | QUIT | SHUTDOWN` are valid commands.\r\n").as_bytes()).await?;
        }
        command = String::new();
    }
}
