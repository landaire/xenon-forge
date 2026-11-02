//! Networking for the Xbox 360 over the winsock of XAM (`NetDll_*`).
//!
//! IPv4 only, without DNS. XNet security is bypassed at startup (a
//! development kit feature) so title sockets can reach ordinary hosts. The
//! system's sockets are marked insecure one by one instead
//! (`Socket::mark_insecure`). Sockets are blocking by default;
//! `set_nonblocking` flips `FIONBIO`.
//!
//! Socket options are those of the `SOL_SOCKET` and `IPPROTO_TCP` levels.
//! The `IPPROTO_IP` options (TTL and multicast) are unsupported, and so is
//! `take_error`: the stack rejects `SO_ERROR`. The local address of a socket
//! that was not bound to one reads as 0.0.0.0.

use crate::fmt;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, ToSocketAddrs};
use crate::sync::LazyLock;
use crate::sys::unsupported;
use crate::time::Duration;

use xbox360_sys::xam::{
    self, AddressFamily, FdSet, IoctlCommand, MessageFlags, Protocol, SOCKET_ERROR, ShutdownHow,
    SockAddrIn, SocketLevel, SocketOption, SocketType, TimeVal, XnCaller,
};

// std built for a system module (`--cfg xbox360_system`) opens its sockets as
// the system, so they do not belong to the running title.
#[cfg(xbox360_system)]
const CALLER: XnCaller = XnCaller::SystemApp;
#[cfg(not(xbox360_system))]
const CALLER: XnCaller = XnCaller::Title;
const SOCKADDR_LENGTH: i32 = size_of::<SockAddrIn>() as i32;

fn last_error() -> io::Error {
    io::Error::from_raw_os_error(xam::NetDll_WSAGetLastError(CALLER).0 as i32)
}

fn cvt(r: i32) -> io::Result<usize> {
    if r == SOCKET_ERROR { Err(last_error()) } else { Ok(r as usize) }
}

/// Brings up XNet, bypassing its security, and winsock. Returns the error
/// code of the first call that fails, or 0.
fn start_network() -> i32 {
    // XNetStartupParams: cfgSizeOfStruct = 0x0D,
    // cfgFlags = XNET_STARTUP_BYPASS_SECURITY (0x01).
    let mut params = [0u8; 16];
    params[0] = 0x0D;
    params[1] = 0x01;
    // SAFETY: `params` is a live local of the size the call reads.
    let code = unsafe { xam::NetDll_XNetStartup(CALLER, params.as_ptr()) };
    if code != 0 {
        return code;
    }
    let mut wsadata = [0u8; 512];
    // SAFETY: `wsadata` is a live local larger than a WSADATA.
    unsafe { xam::NetDll_WSAStartup(CALLER, 0x0202, wsadata.as_mut_ptr()).0 as i32 }
}

fn ensure_net_init() -> io::Result<()> {
    static STARTED: LazyLock<i32> = LazyLock::new(start_network);
    match *STARTED {
        0 => Ok(()),
        code => Err(io::Error::from_raw_os_error(code)),
    }
}

fn to_sockaddr(addr: &SocketAddr) -> io::Result<SockAddrIn> {
    match addr {
        // Binding a loopback address stopped a development kit in an
        // assertion of the network stack.
        SocketAddr::V4(v4) if v4.ip().is_loopback() => Err(io::const_error!(
            io::ErrorKind::AddrNotAvailable,
            "the Xbox 360 network stack has no loopback interface",
        )),
        SocketAddr::V4(v4) => Ok(SockAddrIn {
            family: AddressFamily::Inet.raw() as u16,
            // Network byte order is the native big-endian order.
            port: v4.port(),
            address: u32::from_be_bytes(v4.ip().octets()),
            zero: [0; 8],
        }),
        SocketAddr::V6(_) => Err(io::const_error!(
            io::ErrorKind::Unsupported,
            "IPv6 is not supported on xbox360",
        )),
    }
}

fn from_sockaddr(sa: &SockAddrIn) -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from(sa.address.to_be_bytes()), sa.port))
}

fn empty_sockaddr() -> SockAddrIn {
    SockAddrIn { family: 0, port: 0, address: 0, zero: [0; 8] }
}

/// A timeout in the milliseconds `SO_RCVTIMEO` and `SO_SNDTIMEO` take, with 0
/// for none. A zero duration is an error, as on other targets.
fn timeout_millis(timeout: Option<Duration>) -> io::Result<u32> {
    let Some(timeout) = timeout else { return Ok(0) };
    if timeout.is_zero() {
        return Err(io::const_error!(
            io::ErrorKind::InvalidInput,
            "cannot set a 0 duration timeout",
        ));
    }
    let millis = timeout.as_millis().min(u128::from(u32::MAX)) as u32;
    // A timeout below a millisecond waits for one.
    Ok(millis.max(1))
}

fn timeval(timeout: Duration) -> TimeVal {
    let seconds = timeout.as_secs().min(i32::MAX as u64) as i32;
    let microseconds = timeout.subsec_micros() as i32;
    if seconds == 0 && microseconds == 0 {
        TimeVal { seconds: 0, microseconds: 1 }
    } else {
        TimeVal { seconds, microseconds }
    }
}

/// Winsock `LINGER`.
#[repr(C)]
#[derive(Default)]
struct Linger {
    onoff: u16,
    seconds: u16,
}

fn length(buf: &[u8]) -> i32 {
    buf.len().min(i32::MAX as usize) as i32
}

struct Socket(xam::Socket);

impl Socket {
    fn new(ty: SocketType, proto: Protocol) -> io::Result<Socket> {
        ensure_net_init()?;
        // SAFETY: creating a socket has no pointer arguments.
        let s = unsafe { xam::NetDll_socket(CALLER, AddressFamily::Inet, ty, proto) };
        if s == xam::Socket::INVALID {
            return Err(last_error());
        }
        let sock = Socket(s);
        #[cfg(xbox360_system)]
        sock.mark_insecure()?;
        Ok(sock)
    }

    // XNET_STARTUP_BYPASS_SECURITY does not cover the system's sockets: on a
    // development kit their datagrams were accepted and never sent. SOL_SOCKET
    // option 0x5801 (`SO_INSECURE` of the in-tree `xbox360-sys`, newer than
    // the vendored 0.1.0) lets one exchange plain traffic with any host. A connection that a marked listener accepts worked
    // without it.
    #[cfg(xbox360_system)]
    fn mark_insecure(&self) -> io::Result<()> {
        const INSECURE: SocketOption = SocketOption(0x5801);
        self.set_option(SocketLevel::Socket, INSECURE, 1u32)
    }

    fn set_option<T>(&self, level: SocketLevel, option: SocketOption, value: T) -> io::Result<()> {
        // SAFETY: `self.0` is an open socket and `value` is a live local of the
        // length passed.
        cvt(unsafe {
            xam::NetDll_setsockopt(
                CALLER,
                self.0,
                level,
                option,
                (&raw const value).cast(),
                size_of::<T>() as i32,
            )
        })
        .map(drop)
    }

    fn option<T: Default>(&self, level: SocketLevel, option: SocketOption) -> io::Result<T> {
        let mut value = T::default();
        let mut length = size_of::<T>() as i32;
        // SAFETY: `self.0` is an open socket; `value` and `length` are live
        // locals and `length` is the size of `value`.
        cvt(unsafe {
            xam::NetDll_getsockopt(
                CALLER,
                self.0,
                level,
                option,
                (&raw mut value).cast(),
                &mut length,
            )
        })?;
        Ok(value)
    }

    fn set_flag(&self, level: SocketLevel, option: SocketOption, on: bool) -> io::Result<()> {
        self.set_option(level, option, u32::from(on))
    }

    // Winsock may store a 1-byte BOOL; it lands in the first byte, which
    // keeps the u32 nonzero.
    fn flag(&self, level: SocketLevel, option: SocketOption) -> io::Result<bool> {
        Ok(self.option::<u32>(level, option)? != 0)
    }

    fn set_timeout(&self, option: SocketOption, timeout: Option<Duration>) -> io::Result<()> {
        self.set_option(SocketLevel::Socket, option, timeout_millis(timeout)?)
    }

    fn timeout(&self, option: SocketOption) -> io::Result<Option<Duration>> {
        let millis: u32 = self.option(SocketLevel::Socket, option)?;
        Ok((millis != 0).then(|| Duration::from_millis(u64::from(millis))))
    }

    fn set_nonblocking(&self, nb: bool) -> io::Result<()> {
        let mut v: u32 = nb as u32;
        // SAFETY: `self.0` is an open socket and `v` is a live local.
        cvt(unsafe { xam::NetDll_ioctlsocket(CALLER, self.0, IoctlCommand::Fionbio, &mut v) })
            .map(drop)
    }

    fn socket_addr(&self) -> io::Result<SocketAddr> {
        let mut sa = empty_sockaddr();
        let mut len = SOCKADDR_LENGTH;
        // SAFETY: `self.0` is an open socket; `sa` and `len` are live locals and `len`
        // is the size of `sa`.
        cvt(unsafe { xam::NetDll_getsockname(CALLER, self.0, (&raw mut sa).cast(), &mut len) })?;
        Ok(from_sockaddr(&sa))
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
        let mut sa = empty_sockaddr();
        let mut len = SOCKADDR_LENGTH;
        // SAFETY: `self.0` is an open socket; `sa` and `len` are live locals and `len`
        // is the size of `sa`.
        cvt(unsafe { xam::NetDll_getpeername(CALLER, self.0, (&raw mut sa).cast(), &mut len) })?;
        Ok(from_sockaddr(&sa))
    }

    fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        let sa = to_sockaddr(addr)?;
        // SAFETY: `self.0` is an open socket and `sa` is a live `SockAddrIn`.
        cvt(unsafe { xam::NetDll_connect(CALLER, self.0, (&raw const sa).cast(), SOCKADDR_LENGTH) })
            .map(drop)
    }

    /// Connects within `timeout`: a non-blocking connect, then a wait until
    /// the socket is writable (connected) or in error.
    fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> io::Result<()> {
        if timeout.is_zero() {
            return Err(io::const_error!(
                io::ErrorKind::InvalidInput,
                "cannot set a 0 duration timeout",
            ));
        }
        self.set_nonblocking(true)?;
        let result = match self.connect(addr) {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => self.wait_connected(timeout),
            other => other,
        };
        self.set_nonblocking(false)?;
        result
    }

    fn wait_connected(&self, timeout: Duration) -> io::Result<()> {
        let mut writable = FdSet::new();
        writable.count = 1;
        writable.sockets[0] = self.0;
        let mut failed = FdSet::new();
        failed.count = 1;
        failed.sockets[0] = self.0;
        let timeout = timeval(timeout);
        // SAFETY: the sets and `timeout` are live locals.
        let ready = cvt(unsafe {
            xam::NetDll_select(
                CALLER,
                1,
                crate::ptr::null_mut(),
                &mut writable,
                &mut failed,
                &timeout,
            )
        })?;
        if ready == 0 {
            return Err(io::const_error!(io::ErrorKind::TimedOut, "connection timed out"));
        }
        // The stack has no SO_ERROR to tell why a connection failed. On a
        // development kit a connection to a closed port put the socket in the
        // error set (and the write set).
        if failed.count != 0 {
            return Err(io::const_error!(io::ErrorKind::ConnectionRefused, "connection refused"));
        }
        Ok(())
    }

    fn bind(&self, addr: &SocketAddr) -> io::Result<()> {
        let sa = to_sockaddr(addr)?;
        // SAFETY: `self.0` is an open socket and `sa` is a live `SockAddrIn`.
        cvt(unsafe { xam::NetDll_bind(CALLER, self.0, (&raw const sa).cast(), SOCKADDR_LENGTH) })
            .map(drop)
    }

    fn recv(&self, buf: &mut [u8], flags: MessageFlags) -> io::Result<usize> {
        // SAFETY: `self.0` is an open socket and `buf` is writable for its length.
        cvt(unsafe { xam::NetDll_recv(CALLER, self.0, buf.as_mut_ptr(), length(buf), flags) })
    }

    fn send(&self, buf: &[u8]) -> io::Result<usize> {
        // SAFETY: `self.0` is an open socket and `buf` is readable for its length.
        cvt(unsafe {
            xam::NetDll_send(CALLER, self.0, buf.as_ptr(), length(buf), MessageFlags::empty())
        })
    }

    fn recv_from(&self, buf: &mut [u8], flags: MessageFlags) -> io::Result<(usize, SocketAddr)> {
        let mut sa = empty_sockaddr();
        let mut len = SOCKADDR_LENGTH;
        // SAFETY: `self.0` is an open socket; `buf` is writable for its length and
        // `sa`, `len` are live locals with `len` the size of `sa`.
        let r = unsafe {
            xam::NetDll_recvfrom(
                CALLER,
                self.0,
                buf.as_mut_ptr(),
                length(buf),
                flags,
                (&raw mut sa).cast(),
                &mut len,
            )
        };
        Ok((cvt(r)?, from_sockaddr(&sa)))
    }

    fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        let sa = to_sockaddr(addr)?;
        // SAFETY: `self.0` is an open socket, `buf` is readable for its length
        // and `sa` is a live `SockAddrIn`.
        cvt(unsafe {
            xam::NetDll_sendto(
                CALLER,
                self.0,
                buf.as_ptr(),
                length(buf),
                MessageFlags::empty(),
                (&raw const sa).cast(),
                SOCKADDR_LENGTH,
            )
        })
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        // SAFETY: `self.0` is owned by this `Socket` and is closed only here.
        unsafe { xam::NetDll_closesocket(CALLER, self.0) };
    }
}

fn first_addr<A: ToSocketAddrs>(addr: A) -> io::Result<SocketAddr> {
    addr.to_socket_addrs()?
        .next()
        .ok_or_else(|| io::const_error!(io::ErrorKind::InvalidInput, "no addresses"))
}

pub struct TcpStream(Socket);

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        let mut last = None;
        for a in addr.to_socket_addrs()? {
            match Self::connect_addr(&a) {
                Ok(s) => return Ok(s),
                Err(e) => last = Some(e),
            }
        }
        Err(last.unwrap_or_else(|| io::const_error!(io::ErrorKind::InvalidInput, "no addresses")))
    }

    fn connect_addr(addr: &SocketAddr) -> io::Result<TcpStream> {
        let sock = Socket::new(SocketType::Stream, Protocol::Tcp)?;
        sock.connect(addr)?;
        Ok(TcpStream(sock))
    }

    pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
        let sock = Socket::new(SocketType::Stream, Protocol::Tcp)?;
        sock.connect_timeout(addr, timeout)?;
        Ok(TcpStream(sock))
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf, MessageFlags::empty())
    }

    pub fn read_buf(&self, mut cursor: BorrowedCursor<'_, u8>) -> io::Result<()> {
        // SAFETY: the network stack only writes to the buffer.
        let buffer = unsafe { cursor.as_mut() };
        // SAFETY: `recv` writes the buffer and reads none of it.
        let slice = unsafe {
            crate::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast::<u8>(), buffer.len())
        };
        let n = self.read(slice)?;
        // SAFETY: the network stack initialized `n` bytes.
        unsafe { cursor.advance(n) };
        Ok(())
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        crate::io::default_read_vectored(|b| self.read(b), bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        crate::io::default_write_vectored(|b| self.write(b), bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf, MessageFlags::PEEK)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let how = match how {
            Shutdown::Read => ShutdownHow::Receive,
            Shutdown::Write => ShutdownHow::Send,
            Shutdown::Both => ShutdownHow::Both,
        };
        // SAFETY: `self.0.0` is an open socket.
        cvt(unsafe { xam::NetDll_shutdown(CALLER, self.0.0, how) }).map(drop)
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        unsupported()
    }
    pub fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.0.set_timeout(SocketOption::SO_RCVTIMEO, t)
    }
    pub fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.0.set_timeout(SocketOption::SO_SNDTIMEO, t)
    }
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.timeout(SocketOption::SO_RCVTIMEO)
    }
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.timeout(SocketOption::SO_SNDTIMEO)
    }
    pub fn set_linger(&self, l: Option<Duration>) -> io::Result<()> {
        let linger = Linger {
            onoff: l.is_some() as u16,
            seconds: l.map_or(0, |l| l.as_secs().min(u64::from(u16::MAX)) as u16),
        };
        self.0.set_option(SocketLevel::Socket, SocketOption::SO_LINGER, linger)
    }
    pub fn linger(&self) -> io::Result<Option<Duration>> {
        let linger: Linger = self.0.option(SocketLevel::Socket, SocketOption::SO_LINGER)?;
        Ok((linger.onoff != 0).then(|| Duration::from_secs(u64::from(linger.seconds))))
    }
    pub fn set_nodelay(&self, v: bool) -> io::Result<()> {
        self.0.set_flag(SocketLevel::Tcp, SocketOption::TCP_NODELAY, v)
    }
    pub fn nodelay(&self) -> io::Result<bool> {
        self.0.flag(SocketLevel::Tcp, SocketOption::TCP_NODELAY)
    }
    pub fn set_keepalive(&self, v: bool) -> io::Result<()> {
        self.0.set_flag(SocketLevel::Socket, SocketOption::SO_KEEPALIVE, v)
    }
    pub fn keepalive(&self) -> io::Result<bool> {
        self.0.flag(SocketLevel::Socket, SocketOption::SO_KEEPALIVE)
    }
    pub fn set_ttl(&self, _t: u32) -> io::Result<()> {
        unsupported()
    }
    pub fn ttl(&self) -> io::Result<u32> {
        unsupported()
    }
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        unsupported()
    }
    pub fn set_nonblocking(&self, nb: bool) -> io::Result<()> {
        self.0.set_nonblocking(nb)
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TcpStream(socket={})", self.0.0.raw())
    }
}

pub struct TcpListener(Socket);

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        let a = first_addr(addr)?;
        let sock = Socket::new(SocketType::Stream, Protocol::Tcp)?;
        sock.bind(&a)?;
        // SAFETY: `sock.0` is an open, bound socket.
        cvt(unsafe { xam::NetDll_listen(CALLER, sock.0, 128) })?;
        Ok(TcpListener(sock))
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let mut sa = empty_sockaddr();
        let mut len = SOCKADDR_LENGTH;
        // SAFETY: `self.0.0` is a listening socket; `sa` and `len` are live locals and
        // `len` is the size of `sa`.
        let s = unsafe { xam::NetDll_accept(CALLER, self.0.0, (&raw mut sa).cast(), &mut len) };
        if s == xam::Socket::INVALID {
            return Err(last_error());
        }
        Ok((TcpStream(Socket(s)), from_sockaddr(&sa)))
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        unsupported()
    }
    pub fn set_ttl(&self, _t: u32) -> io::Result<()> {
        unsupported()
    }
    pub fn ttl(&self) -> io::Result<u32> {
        unsupported()
    }
    pub fn set_only_v6(&self, _v: bool) -> io::Result<()> {
        unsupported()
    }
    /// IPv4 sockets accept no IPv6 connections.
    pub fn only_v6(&self) -> io::Result<bool> {
        Ok(false)
    }
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        unsupported()
    }
    pub fn set_nonblocking(&self, nb: bool) -> io::Result<()> {
        self.0.set_nonblocking(nb)
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TcpListener(socket={})", self.0.0.raw())
    }
}

pub struct UdpSocket(Socket);

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
        let a = first_addr(addr)?;
        let sock = Socket::new(SocketType::Datagram, Protocol::Udp)?;
        sock.bind(&a)?;
        Ok(UdpSocket(sock))
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }
    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf, MessageFlags::empty())
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf, MessageFlags::PEEK)
    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        self.0.send_to(buf, addr)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf, MessageFlags::empty())
    }
    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf, MessageFlags::PEEK)
    }
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        self.0.connect(&first_addr(addr)?)
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        unsupported()
    }
    pub fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.0.set_timeout(SocketOption::SO_RCVTIMEO, t)
    }
    pub fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.0.set_timeout(SocketOption::SO_SNDTIMEO, t)
    }
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.timeout(SocketOption::SO_RCVTIMEO)
    }
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.timeout(SocketOption::SO_SNDTIMEO)
    }
    pub fn set_broadcast(&self, v: bool) -> io::Result<()> {
        self.0.set_flag(SocketLevel::Socket, SocketOption::SO_BROADCAST, v)
    }
    pub fn broadcast(&self) -> io::Result<bool> {
        self.0.flag(SocketLevel::Socket, SocketOption::SO_BROADCAST)
    }
    pub fn set_multicast_loop_v4(&self, _v: bool) -> io::Result<()> {
        unsupported()
    }
    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        unsupported()
    }
    pub fn set_multicast_ttl_v4(&self, _v: u32) -> io::Result<()> {
        unsupported()
    }
    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        unsupported()
    }
    pub fn set_multicast_loop_v6(&self, _v: bool) -> io::Result<()> {
        unsupported()
    }
    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        unsupported()
    }
    pub fn join_multicast_v4(&self, _m: &Ipv4Addr, _i: &Ipv4Addr) -> io::Result<()> {
        unsupported()
    }
    pub fn join_multicast_v6(&self, _m: &Ipv6Addr, _i: u32) -> io::Result<()> {
        unsupported()
    }
    pub fn leave_multicast_v4(&self, _m: &Ipv4Addr, _i: &Ipv4Addr) -> io::Result<()> {
        unsupported()
    }
    pub fn leave_multicast_v6(&self, _m: &Ipv6Addr, _i: u32) -> io::Result<()> {
        unsupported()
    }
    pub fn set_ttl(&self, _t: u32) -> io::Result<()> {
        unsupported()
    }
    pub fn ttl(&self) -> io::Result<u32> {
        unsupported()
    }
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        unsupported()
    }
    pub fn set_nonblocking(&self, nb: bool) -> io::Result<()> {
        self.0.set_nonblocking(nb)
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UdpSocket(socket={})", self.0.0.raw())
    }
}

/// Numeric-only host resolution (no DNS): parses "a.b.c.d".
pub struct LookupHost {
    addr: Option<SocketAddr>,
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        self.addr.take()
    }
}

pub fn lookup_host(host: &str, port: u16) -> io::Result<LookupHost> {
    use crate::str::FromStr;
    let ip = Ipv4Addr::from_str(host.trim()).map_err(|_| {
        io::const_error!(io::ErrorKind::Unsupported, "DNS is not supported on xbox360")
    })?;
    Ok(LookupHost { addr: Some(SocketAddr::V4(SocketAddrV4::new(ip, port))) })
}
