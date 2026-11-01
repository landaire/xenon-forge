//! Multi-thread tokio on the Xbox 360: a work-stealing runtime with two
//! workers runs async tasks that each await a timer, and `spawn_blocking`
//! does an HTTP GET over `std::net` to the web server at the address of the
//! first argument. The title prints the results to the debug output and
//! ends.
//!
//! Start the server on the host, then run the title with its address:
//!
//! ```text
//! python3 src/examples/tokweb/host_server.py
//! bin/buck2 run //src/examples/tokweb:tokweb -- 192.168.1.10:8080
//! ```

#![no_main]

use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::time::Duration;

/// The status line of `GET /` from `host`, and the length of the response.
fn http_get(host: &str) -> std::io::Result<(String, usize)> {
    let mut stream = TcpStream::connect(host)?;
    write!(stream, "GET / HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n")?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    let status = response.split(|&b| b == b'\r').next().unwrap_or_default();
    Ok((String::from_utf8_lossy(status).into_owned(), response.len()))
}

fn main_body() {
    let Some(host) = std::env::args().nth(1) else {
        println!("tokweb: pass the web server address, such as 192.168.1.10:8080");
        return;
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_time()
        .build()
        .unwrap();

    runtime.block_on(async {
        // Fan out async tasks across the workers.
        let sum = Arc::new(AtomicU32::new(0));
        let handles: Vec<_> = (1..=8u32)
            .map(|value| {
                let sum = Arc::clone(&sum);
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    sum.fetch_add(value * value, Ordering::SeqCst);
                })
            })
            .collect();
        for handle in handles {
            handle.await.unwrap();
        }
        // 1 + 4 + ... + 64.
        println!("tokweb: 8 tasks, sum of squares {} (expected 204)", sum.load(Ordering::SeqCst));

        // The blocking pool runs on real threads.
        let request = host.clone();
        match tokio::task::spawn_blocking(move || http_get(&request)).await.unwrap() {
            Ok((status, length)) => println!("tokweb: GET http://{host}/: {status}, {length} bytes"),
            Err(error) => println!("tokweb: GET http://{host}/: {error}"),
        }
    });
}

xbox360::sys::bridged!(#[unsafe(no_mangle)] pub _start, main);

extern "C" fn main(_context: u32) -> u32 {
    main_body();
    println!("tokweb: done");
    xbox360::title::terminate()
}
