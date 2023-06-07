use super::*;
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
    assert_eq!("Ok\r\nquick brown\r\n".to_string(), lines);

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
        "Err - failed to retrieve line 1000. There are only 4 lines available.\r\n".to_string(),
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
