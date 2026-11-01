//! Checks the standard library on a development kit and reports one line
//! per check to the debug output: `PASS name` or `FAIL name: detail`, then
//! `stdcheck: N passed, M failed` and `stdcheck: done`.
//!
//! The file system checks work in `DEVKIT:\stdcheck\fs`. The network checks
//! need the address of a host peer as the first argument (`a.b.c.d:port`),
//! which `run.sh` starts: it echoes TCP lines and UDP datagrams, and answers
//! a `CONNECT port` line by connecting back to that port and sending `back`.

#![no_main]
#![feature(tcp_linger)]

use std::collections::hash_map::RandomState;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::hash::BuildHasher;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const ROOT: &str = r"\Device\Harddisk0\Partition1\DEVKIT\stdcheck\fs";

type Check = Result<(), String>;

#[derive(Default)]
struct Report {
    passed: u32,
    failed: u32,
}

impl Report {
    fn check(&mut self, name: &str, result: Check) {
        match result {
            Ok(()) => {
                self.passed += 1;
                println!("PASS {name}");
            }
            Err(detail) => {
                self.failed += 1;
                println!("FAIL {name}: {detail}");
            }
        }
    }
}

macro_rules! ensure {
    ($condition:expr, $($detail:tt)+) => {
        if !$condition {
            return Err(format!($($detail)+));
        }
    };
}

/// `?` for io results inside checks.
trait Context<T> {
    fn context(self, what: &str) -> Result<T, String>;
}

impl<T> Context<T> for std::io::Result<T> {
    fn context(self, what: &str) -> Result<T, String> {
        self.map_err(|error| format!("{what}: {error} ({:?})", error.kind()))
    }
}

fn expect_kind<T: std::fmt::Debug>(result: std::io::Result<T>, kind: ErrorKind) -> Check {
    match result {
        Err(error) if error.kind() == kind => Ok(()),
        other => Err(format!("expected {kind:?}, got {other:?}")),
    }
}

fn root() -> PathBuf {
    PathBuf::from(ROOT)
}

fn path_checks(report: &mut Report) {
    report.check("path components", (|| {
        let path = Path::new(r"\Device\Harddisk0\file.txt");
        ensure!(path.parent() == Some(Path::new(r"\Device\Harddisk0")), "parent {:?}", path.parent());
        ensure!(path.file_name() == Some("file.txt".as_ref()), "file name {:?}", path.file_name());
        ensure!(path.extension() == Some("txt".as_ref()), "extension {:?}", path.extension());
        let joined = Path::new(r"game:\data").join("save.dat");
        ensure!(joined == Path::new(r"game:\data\save.dat"), "join {joined:?}");
        ensure!(Path::new("a/b").components().count() == 2, "slash is not a separator");
        Ok(())
    })());
    report.check("path absolute", (|| {
        ensure!(Path::new(r"\Device\Harddisk0").is_absolute(), "object name is relative");
        ensure!(Path::new(r"game:\x").is_absolute(), "drive path is relative");
        ensure!(Path::new(r"data\x").is_relative(), "relative path is absolute");
        let replaced = Path::new(r"data\x").join(r"game:\y");
        ensure!(replaced == Path::new(r"game:\y"), "join of a drive path {replaced:?}");
        let absolute = std::path::absolute(r"game:\a\.\b").context("absolute")?;
        ensure!(absolute == Path::new(r"game:\a\b"), "absolute {absolute:?}");
        expect_kind(std::path::absolute(r"a\b"), ErrorKind::Unsupported)
    })());
}

fn fs_checks(report: &mut Report) {
    let root = root();
    report.check("fs setup", (|| {
        match fs::remove_dir_all(&root) {
            Err(error) if error.kind() != ErrorKind::NotFound => {
                return Err(format!("remove_dir_all of an earlier run: {error}"));
            }
            _ => {}
        }
        fs::create_dir_all(root.join(r"deep\er")).context("create_dir_all")?;
        ensure!(root.join(r"deep\er").is_dir(), "created directory is missing");
        expect_kind(fs::create_dir(root.join("deep")), ErrorKind::AlreadyExists)
    })());

    let file = root.join("file.txt");
    report.check("fs write and read", (|| {
        fs::write(&file, "hello").context("write")?;
        let text = fs::read_to_string(&file).context("read")?;
        ensure!(text == "hello", "read {text:?}");
        let slashed = ROOT.replace('\\', "/") + "/file.txt";
        let text = fs::read_to_string(&slashed).context("read with /")?;
        ensure!(text == "hello", "read with / {text:?}");
        Ok(())
    })());

    report.check("fs metadata", (|| {
        let metadata = fs::metadata(&file).context("file metadata")?;
        ensure!(metadata.len() == 5 && metadata.is_file() && !metadata.is_dir(), "file {metadata:?}");
        let modified = metadata.modified().context("modified")?;
        let age = SystemTime::now().duration_since(modified).unwrap_or_else(|e| e.duration());
        ensure!(age < Duration::from_secs(3600), "modified {age:?} from now");
        let metadata = fs::metadata(&root).context("directory metadata")?;
        ensure!(metadata.is_dir() && !metadata.is_file(), "directory {metadata:?}");
        ensure!(fs::exists(&file).context("exists")?, "file does not exist");
        ensure!(!fs::exists(root.join("missing")).context("exists")?, "missing file exists");
        expect_kind(fs::metadata(root.join(r"missing\deeper")), ErrorKind::NotFound)
    })());

    report.check("fs drive path", (|| {
        let metadata = fs::metadata(r"game:\default.xex").context("game:\\default.xex")?;
        ensure!(metadata.is_file() && metadata.len() > 0, "{metadata:?}");
        Ok(())
    })());

    report.check("fs append", (|| {
        let mut appender = OpenOptions::new().append(true).open(&file).context("open")?;
        appender.write_all(b"ab").context("write")?;
        appender.seek(SeekFrom::Start(0)).context("seek")?;
        appender.write_all(b"cd").context("write after seek")?;
        drop(appender);
        let text = fs::read_to_string(&file).context("read")?;
        ensure!(text == "helloabcd", "content {text:?}");
        Ok(())
    })());

    report.check("fs seek and tell", (|| {
        let mut f = OpenOptions::new().read(true).write(true).open(&file).context("open")?;
        ensure!(f.seek(SeekFrom::End(-2)).context("seek end")? == 7, "seek end");
        ensure!(f.stream_position().context("position")? == 7, "position");
        let mut tail = String::new();
        f.read_to_string(&mut tail).context("read")?;
        ensure!(tail == "cd", "tail {tail:?}");
        let far = 5 << 30;
        ensure!(f.seek(SeekFrom::Start(far)).context("seek past 4 GiB")? == far, "seek past 4 GiB");
        ensure!(f.stream_position().context("position")? == far, "position past 4 GiB");
        expect_kind(f.seek(SeekFrom::Current(-(far as i64) - 1)), ErrorKind::InvalidInput)?;
        f.seek(SeekFrom::Start(1)).context("seek")?;
        let clone = f.try_clone().context("try_clone")?;
        ensure!((&clone).stream_position().context("clone position")? == 1, "clone position");
        Ok(())
    })());

    report.check("fs set_len", (|| {
        let f = OpenOptions::new().write(true).open(&file).context("open")?;
        f.set_len(3).context("shrink")?;
        ensure!(fs::read(&file).context("read")? == b"hel", "shrunk content");
        f.set_len(6).context("grow")?;
        let grown = fs::read(&file).context("read")?;
        ensure!(grown == b"hel\0\0\0", "grown content {grown:?}");
        Ok(())
    })());

    report.check("fs open errors", (|| {
        expect_kind(OpenOptions::new().write(true).create_new(true).open(&file), ErrorKind::AlreadyExists)?;
        expect_kind(File::open(root.join("missing")), ErrorKind::NotFound)?;
        expect_kind(File::open(&root), ErrorKind::IsADirectory)?;
        expect_kind(OpenOptions::new().open(&file), ErrorKind::InvalidInput)?;
        expect_kind(OpenOptions::new().read(true).create(true).open(&file), ErrorKind::InvalidInput)
    })());

    report.check("fs unsupported", (|| {
        let f = File::open(&file).context("open")?;
        expect_kind(f.lock(), ErrorKind::Unsupported)?;
        expect_kind(fs::canonicalize(&file), ErrorKind::Unsupported)?;
        expect_kind(fs::hard_link(&file, root.join("link")), ErrorKind::Unsupported)
    })());

    report.check("fs read large", (|| {
        let data: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        let large = root.join("large.bin");
        fs::write(&large, &data).context("write")?;
        let read = fs::read(&large).context("read")?;
        ensure!(read == data, "read {} bytes, wrote {}", read.len(), data.len());
        let copy = root.join("copy.bin");
        let copied = fs::copy(&large, &copy).context("copy")?;
        ensure!(copied == data.len() as u64, "copied {copied}");
        ensure!(fs::read(&copy).context("read copy")? == data, "copy differs");
        Ok(())
    })());

    report.check("fs read_dir", (|| {
        let mut names = Vec::new();
        for entry in fs::read_dir(&root).context("read_dir")? {
            let entry = entry.context("entry")?;
            let name = entry.file_name().into_string().map_err(|n| format!("name {n:?}"))?;
            ensure!(entry.path() == root.join(&name), "path {:?}", entry.path());
            let metadata = fs::metadata(entry.path()).context("metadata of the entry path")?;
            let kind = entry.file_type().context("file type")?;
            ensure!(kind.is_dir() == metadata.is_dir(), "{name}: file type {kind:?}");
            ensure!(entry.metadata().context("entry metadata")?.len() == metadata.len(), "{name}: length");
            names.push(name);
        }
        names.sort();
        ensure!(names == ["copy.bin", "deep", "file.txt", "large.bin"], "names {names:?}");
        let empty = fs::read_dir(root.join(r"deep\er")).context("read_dir of an empty directory")?;
        ensure!(empty.count() == 0, "empty directory has entries");
        expect_kind(fs::read_dir(&file), ErrorKind::NotADirectory)
    })());

    report.check("fs rename", (|| {
        let renamed = root.join("renamed.bin");
        fs::rename(root.join("copy.bin"), &renamed).context("rename")?;
        ensure!(!root.join("copy.bin").exists() && renamed.exists(), "rename did not move the file");
        fs::rename(&renamed, root.join("large.bin")).context("rename over a file")?;
        ensure!(!renamed.exists(), "replaced source still exists");
        fs::rename(root.join("deep"), root.join("moved")).context("rename a directory")?;
        ensure!(root.join(r"moved\er").is_dir(), "moved directory is missing");
        expect_kind(fs::rename(root.join("missing"), root.join("other")), ErrorKind::NotFound)
    })());

    report.check("fs set_times", (|| {
        let time = UNIX_EPOCH + Duration::from_secs(1_500_000_000);
        let f = OpenOptions::new().write(true).open(&file).context("open")?;
        f.set_times(fs::FileTimes::new().set_modified(time)).context("set_times")?;
        let modified = fs::metadata(&file).context("metadata")?.modified().context("modified")?;
        let off = modified.duration_since(time).unwrap_or_else(|e| e.duration());
        ensure!(off <= Duration::from_secs(2), "modified {off:?} from the time set");
        Ok(())
    })());

    report.check("fs permissions", (|| {
        let mut permissions = fs::metadata(&file).context("metadata")?.permissions();
        ensure!(!permissions.readonly(), "new file is read-only");
        permissions.set_readonly(true);
        match fs::set_permissions(&file, permissions.clone()) {
            // FATX keeps no read-only attribute.
            Err(error) if error.kind() == ErrorKind::Unsupported => {}
            Err(error) => return Err(format!("set read-only: {error}")),
            Ok(()) => ensure!(
                fs::metadata(&file).context("metadata")?.permissions().readonly(),
                "set read-only reported success"
            ),
        }
        permissions.set_readonly(false);
        fs::set_permissions(&file, permissions).context("clear read-only")
    })());

    report.check("fs remove", (|| {
        expect_kind(fs::remove_file(root.join("moved")), ErrorKind::IsADirectory)?;
        expect_kind(fs::remove_dir(&file), ErrorKind::NotADirectory)?;
        expect_kind(fs::remove_dir(root.join("moved")), ErrorKind::DirectoryNotEmpty)?;
        fs::remove_file(&file).context("remove_file")?;
        ensure!(!file.exists(), "removed file exists");
        expect_kind(fs::remove_file(&file), ErrorKind::NotFound)?;
        fs::remove_dir(root.join(r"moved\er")).context("remove_dir")?;
        fs::remove_dir_all(&root).context("remove_dir_all")?;
        ensure!(!root.exists(), "root exists after remove_dir_all");
        Ok(())
    })());
}

fn random_checks(report: &mut Report) {
    report.check("random hash keys", (|| {
        // Each thread seeds its keys once; fixed keys give equal hashes.
        let here = RandomState::new().hash_one(1u32);
        let there = std::thread::spawn(|| RandomState::new().hash_one(1u32))
            .join()
            .map_err(|_| "thread panicked".to_string())?;
        ensure!(here != there, "two threads hashed with the same keys");
        Ok(())
    })());
}

fn net_checks(report: &mut Report, host: SocketAddr) {
    report.check("net loopback", (|| {
        expect_kind(UdpSocket::bind("127.0.0.1:0"), ErrorKind::AddrNotAvailable)?;
        expect_kind("example.com:80".to_socket_addrs(), ErrorKind::Unsupported)
    })());

    let stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(error) => {
            report.check("net tcp connect", Err(format!("{error}")));
            return;
        }
    };
    report.check("net tcp echo", (|| {
        let mut stream = &stream;
        stream.write_all(b"hello\n").context("write")?;
        let mut line = [0u8; 6];
        stream.read_exact(&mut line).context("read")?;
        ensure!(&line == b"hello\n", "echo {line:?}");
        ensure!(stream.peer_addr().context("peer_addr")? == host, "peer address");
        // The stack reports 0.0.0.0 for the address of an unbound socket.
        let local = stream.local_addr().context("local_addr")?;
        ensure!(local.port() != 0, "local address {local}");
        Ok(())
    })());

    report.check("net tcp options", (|| {
        stream.set_nodelay(true).context("set_nodelay")?;
        ensure!(stream.nodelay().context("nodelay")?, "nodelay off after set");
        stream.set_nodelay(false).context("set_nodelay")?;
        ensure!(!stream.nodelay().context("nodelay")?, "nodelay on after clear");
        stream.set_linger(Some(Duration::from_secs(2))).context("set_linger")?;
        ensure!(stream.linger().context("linger")? == Some(Duration::from_secs(2)), "linger");
        stream.set_linger(None).context("clear linger")?;
        ensure!(stream.linger().context("linger")?.is_none(), "linger after clear");
        stream.set_write_timeout(Some(Duration::from_millis(1500))).context("set_write_timeout")?;
        let timeout = stream.write_timeout().context("write_timeout")?;
        ensure!(timeout == Some(Duration::from_millis(1500)), "write timeout {timeout:?}");
        expect_kind(stream.set_read_timeout(Some(Duration::ZERO)), ErrorKind::InvalidInput)?;
        expect_kind(stream.take_error(), ErrorKind::Unsupported)?;
        expect_kind(stream.ttl(), ErrorKind::Unsupported)
    })());

    report.check("net tcp read timeout", (|| {
        stream.set_read_timeout(Some(Duration::from_millis(300))).context("set_read_timeout")?;
        let timeout = stream.read_timeout().context("read_timeout")?;
        ensure!(timeout == Some(Duration::from_millis(300)), "read timeout {timeout:?}");
        let start = Instant::now();
        let result = (&stream).read(&mut [0u8; 8]);
        let waited = start.elapsed();
        ensure!(
            matches!(&result, Err(e) if matches!(e.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock)),
            "read {result:?}"
        );
        ensure!(waited >= Duration::from_millis(250) && waited < Duration::from_secs(3), "waited {waited:?}");
        stream.set_read_timeout(None).context("clear read timeout")?;
        ensure!(stream.read_timeout().context("read_timeout")?.is_none(), "read timeout after clear");
        Ok(())
    })());

    report.check("net tcp nonblocking", (|| {
        stream.set_nonblocking(true).context("set_nonblocking")?;
        let result = (&stream).read(&mut [0u8; 8]);
        stream.set_nonblocking(false).context("clear nonblocking")?;
        expect_kind(result, ErrorKind::WouldBlock)
    })());

    report.check("net tcp shutdown", (|| {
        stream.shutdown(std::net::Shutdown::Write).context("shutdown")?;
        let mut rest = Vec::new();
        (&stream).read_to_end(&mut rest).context("read to end")?;
        ensure!(rest.is_empty(), "data after shutdown {rest:?}");
        Ok(())
    })());
    drop(stream);

    report.check("net tcp connect_timeout", (|| {
        let refused = SocketAddr::new(host.ip(), host.port() + 1);
        expect_kind(
            TcpStream::connect_timeout(&refused, Duration::from_secs(5)),
            ErrorKind::ConnectionRefused,
        )?;
        let unroutable: SocketAddr = "10.255.255.1:9".parse().unwrap();
        let start = Instant::now();
        expect_kind(TcpStream::connect_timeout(&unroutable, Duration::from_millis(500)), ErrorKind::TimedOut)?;
        let waited = start.elapsed();
        ensure!(waited >= Duration::from_millis(400) && waited < Duration::from_secs(3), "waited {waited:?}");
        expect_kind(TcpStream::connect_timeout(&host, Duration::ZERO), ErrorKind::InvalidInput)?;
        let mut stream = TcpStream::connect_timeout(&host, Duration::from_secs(5)).context("connect")?;
        stream.write_all(b"again\n").context("write")?;
        let mut line = [0u8; 6];
        stream.read_exact(&mut line).context("read")?;
        ensure!(&line == b"again\n", "echo {line:?}");
        Ok(())
    })());

    report.check("net tcp listener", (|| {
        let listener = TcpListener::bind("0.0.0.0:0").context("bind")?;
        let port = listener.local_addr().context("local_addr")?.port();
        let mut control = TcpStream::connect(host).context("connect")?;
        writeln!(control, "CONNECT {port}").context("write")?;
        let (mut accepted, peer) = listener.accept().context("accept")?;
        ensure!(peer.ip() == host.ip(), "peer {peer}");
        let mut line = [0u8; 5];
        accepted.read_exact(&mut line).context("read")?;
        ensure!(&line == b"back\n", "line {line:?}");
        Ok(())
    })());

    report.check("net udp", (|| {
        let socket = UdpSocket::bind("0.0.0.0:0").context("bind")?;
        expect_kind(socket.peer_addr(), ErrorKind::NotConnected)?;
        socket.set_read_timeout(Some(Duration::from_secs(2))).context("set_read_timeout")?;
        socket.send_to(b"datagram", host).context("send_to")?;
        let mut buf = [0u8; 32];
        let (n, from) = socket.recv_from(&mut buf).context("recv_from")?;
        ensure!(&buf[..n] == b"datagram" && from == host, "{:?} from {from}", &buf[..n]);
        socket.connect(host).context("connect")?;
        ensure!(socket.peer_addr().context("peer_addr")? == host, "peer address");
        socket.send(b"connected").context("send")?;
        let n = socket.recv(&mut buf).context("recv")?;
        ensure!(&buf[..n] == b"connected", "{:?}", &buf[..n]);
        socket.set_read_timeout(Some(Duration::from_millis(200))).context("set_read_timeout")?;
        let start = Instant::now();
        let result = socket.recv(&mut buf);
        ensure!(
            matches!(&result, Err(e) if matches!(e.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock)),
            "recv {result:?}"
        );
        ensure!(start.elapsed() >= Duration::from_millis(150), "waited {:?}", start.elapsed());
        expect_kind(socket.multicast_ttl_v4(), ErrorKind::Unsupported)
    })());

    report.check("net udp broadcast", (|| {
        let socket = UdpSocket::bind("0.0.0.0:0").context("bind")?;
        let everyone = SocketAddr::from(([255, 255, 255, 255], host.port()));
        ensure!(!socket.broadcast().context("broadcast")?, "broadcast on by default");
        expect_kind(socket.send_to(b"broadcast", everyone), ErrorKind::PermissionDenied)?;
        socket.set_broadcast(true).context("set_broadcast")?;
        ensure!(socket.broadcast().context("broadcast")?, "broadcast off after set");
        socket.send_to(b"broadcast", everyone).context("send_to with broadcast")?;
        Ok(())
    })());
}


fn main_body() {
    let mut report = Report::default();
    path_checks(&mut report);
    fs_checks(&mut report);
    random_checks(&mut report);
    match std::env::args().nth(1).map(|a| a.parse::<SocketAddr>()) {
        Some(Ok(host)) => net_checks(&mut report, host),
        Some(Err(error)) => report.check("net host argument", Err(format!("{error}"))),
        None => println!("stdcheck: no host argument, network checks skipped"),
    }
    println!("stdcheck: {} passed, {} failed", report.passed, report.failed);
    println!("stdcheck: done");
}

xbox360::sys::bridged!(#[unsafe(no_mangle)] pub _start, main);

extern "C" fn main(_context: u32) -> u32 {
    main_body();
    xbox360::title::terminate()
}
