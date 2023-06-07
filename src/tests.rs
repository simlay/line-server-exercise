use super::*;
use futures::future::join_all;
use pretty_assertions::assert_eq;
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn single_client_test() {
    let port = DEFAULT_PORT + 1;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let server = LineServer {
        line_file: PathBuf::from("./example.txt"),
        bind_addr: addr,
    };
    let thread = tokio::spawn(async move { server.run().await });

    // Trial and error shows a slight delay to let the listener start is required.
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");

    stream
        .write_all("GET 1\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let mut buf = [0; 128];
    let len = stream
        .read(&mut buf)
        .await
        .expect("Failed to read from stream");
    let lines = String::from_utf8(buf[0..len].to_vec()).expect("Failed to parse lines");
    assert_eq!("Ok\r\nthe\r\n".to_string(), lines);

    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");
    stream
        .write_all("SHUTDOWN\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let _ = thread.await.expect("Server ended in error!");
}

#[tokio::test]
async fn client_test_errors() {
    let port = DEFAULT_PORT + 2;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let server = LineServer {
        line_file: PathBuf::from("./example.txt"),
        bind_addr: addr,
    };
    let thread = tokio::spawn(async move { server.run().await });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");
    info!("Connected to server");

    let mut buf = [0; 128];
    stream
        .write_all("GET aoeu\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let len = stream
        .read(&mut buf)
        .await
        .expect("Failed to read from stream");
    let lines = String::from_utf8(buf[0..len].to_vec()).expect("Failed to parse lines");
    assert_eq!(
        "Err - invalid digit found in string. Is AOEU an unsigned integer under 65536?\r\n"
            .to_string(),
        lines
    );

    stream
        .write_all("GET 1000\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let len = stream
        .read(&mut buf)
        .await
        .expect("Failed to read from stream");
    let lines = String::from_utf8(buf[0..len].to_vec()).expect("Failed to parse lines");
    assert_eq!(
        "Err - failed to retrieve line 999. There are only 4 lines available.\r\n".to_string(),
        lines
    );

    stream
        .write_all("GET aoeu\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let len = stream
        .read(&mut buf)
        .await
        .expect("Failed to read from stream");
    let lines = String::from_utf8(buf[0..len].to_vec()).expect("Failed to parse lines");
    assert_eq!(
        "Err - invalid digit found in string. Is AOEU an unsigned integer under 65536?\r\n"
            .to_string(),
        lines
    );

    stream
        .write_all("THIS_COMMAND_DOES_NOT_EXIST\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let len = stream
        .read(&mut buf)
        .await
        .expect("Failed to read from stream");
    let lines = String::from_utf8(buf[0..len].to_vec()).expect("Failed to parse lines");
    assert_eq!("Err - THIS_COMMAND_DOES_NOT_EXIST is an invalid command. `GET nnnn | QUIT | SHUTDOWN` are valid commands.\r\n".to_string(), lines);

    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");
    stream
        .write_all("SHUTDOWN\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let _ = thread.await.expect("Server ended in error!");
}

#[tokio::test]
async fn single_client_quit() {
    let port = DEFAULT_PORT + 3;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let server = LineServer {
        line_file: PathBuf::from("./example.txt"),
        bind_addr: addr,
    };
    let thread = tokio::spawn(async move { server.run().await });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");
    stream
        .write_all("QUIT\n".as_bytes())
        .await
        .expect("Failed to write to socket");

    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");

    stream
        .write_all("SHUTDOWN\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let _ = thread.await.expect("Server ended in error!");
}

async fn exhausted_client(addr: SocketAddr, file: PathBuf, num_lines: usize) -> Result<(), String> {
    let file = File::open(file).expect("Failed to open file");
    let file = std::io::BufReader::new(file);
    let lines: Vec<String> = file
        .lines()
        .collect::<Result<Vec<String>, std::io::Error>>()
        .expect("Failed to get lines from file");
    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");
    for i in 1..num_lines {
        stream
            .write_all(format!("GET {i}\n").as_bytes())
            .await
            .expect("Failed to write to socket");
        let mut buf = [0; 128];
        let len = stream
            .read(&mut buf)
            .await
            .expect("Failed to read from stream");
        let line = String::from_utf8(buf[0..len].to_vec()).expect("Failed to parse lines");
        assert!(line.starts_with("Ok\r\n"));
        let expected_line = lines
            .get(i - 1)
            .expect("Failed to get line from file")
            .clone();
        assert_eq!(format!("Ok\r\n{expected_line}\r\n"), line);
    }
    stream
        .write_all("QUIT\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    Ok(())
}

#[tokio::test]
async fn one_hundred_clients() {
    let port = DEFAULT_PORT + 4;
    let line_file = PathBuf::from("./example_large.txt");
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let server = LineServer {
        line_file: line_file.clone(),
        bind_addr: addr,
    };
    let thread = tokio::spawn(async move { server.run().await });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let mut threads = Vec::new();
    for _ in 0..100 {
        let line_file = line_file.clone();
        threads.push(tokio::spawn(async move {
            exhausted_client(addr, line_file, 400).await
        }));
    }
    let out = join_all(threads)
        .await
        .into_iter()
        .collect::<Result<Vec<Result<(), String>>, _>>();
    let out = out
        .expect("Failed to join all client threads")
        .into_iter()
        .collect::<Result<Vec<()>, String>>();
    out.expect("One of the clients had an error");

    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");

    stream
        .write_all("SHUTDOWN\n".as_bytes())
        .await
        .expect("Failed to write to socket");
    let _ = thread.await.expect("Server ended in error!");
}
