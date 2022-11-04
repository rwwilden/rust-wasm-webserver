use bytecodec::DecodeExt;
use httpcodec::{HttpVersion, ReasonPhrase, Request, RequestDecoder, Response, StatusCode};
use std::io::{Read, Write};
#[cfg(feature = "std")]
use std::net::{Shutdown, TcpListener, TcpStream};
#[cfg(not(feature = "std"))]
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};


fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or(String::from("8080")).parse::<u16>().unwrap();

    println!("Going to bind to port {port}");
    let listener = match TcpListener::bind(("0.0.0.0", port), false) {
        Ok(listener) => listener,
        Err(e) => {
            panic!("{:?}", e);
        }
    };
    println!("Bound to port {port}");

    loop {
        let stream = match listener.accept(false) {
            Ok((stream, _)) => stream,
            Err(e) => {
                panic!("{:?}", e);
            }
        };
        println!("Accepted client");

        let _ = handle_client(stream);
    }
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];
    let mut data = Vec::new();

    loop {
        let n = stream.read(&mut buf)?;
        data.extend_from_slice(&buf[0..n]);
        if n < 1024 {
            break;
        }
    }

    let mut decoder =
        RequestDecoder::<httpcodec::BodyDecoder<bytecodec::bytes::Utf8Decoder>>::default();

    let req = match decoder.decode_from_bytes(data.as_slice()) {
        Ok(req) => handle_http(req),
        Err(e) => Err(e)
    };

    let r = match req {
        Ok(r) => r,
        Err(e) => {
            let err = format!("{:?}", e);
            Response::new(
                HttpVersion::V1_0,
                StatusCode::new(500).unwrap(),
                ReasonPhrase::new(err.as_str()).unwrap(),
                err.clone()
            )
        }
    };

    let write_buf = r.to_string();
    stream.write(write_buf.as_bytes())?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn handle_http(req: Request<String>) -> bytecodec::Result<Response<String>> {
    Ok(Response::new(
        HttpVersion::V1_0,
        StatusCode::new(200)?,
        ReasonPhrase::new("")?,
        format!("echo: {}", req.body())
    ))
}