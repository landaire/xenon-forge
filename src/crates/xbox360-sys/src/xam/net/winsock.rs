/// A Winsock socket, or [`Socket::INVALID`] from a call that failed.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Socket(u32);

impl Socket {
    /// `INVALID_SOCKET`.
    pub const INVALID: Self = Self(u32::MAX);

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Result of a Winsock call that has no count to return: 0 or this.
pub const SOCKET_ERROR: i32 = -1;

value_enum! {
    /// Address family of a socket.
    #[non_exhaustive]
    pub enum AddressFamily: u32 {
        /// `AF_INET`.
        Inet = 2,
    }
}

value_enum! {
    /// `SOCK_*` socket type.
    #[non_exhaustive]
    pub enum SocketType: u32 {
        Stream = 1,
        Datagram = 2,
    }
}

value_enum! {
    /// `IPPROTO_*` socket protocol.
    #[non_exhaustive]
    pub enum Protocol: u32 {
        Tcp = 6,
        Udp = 17,
    }
}

value_enum! {
    /// `SD_*`: what `NetDll_shutdown` disables.
    pub enum ShutdownHow: i32 {
        Receive = 0,
        Send = 1,
        Both = 2,
    }
}

value_enum! {
    /// `NetDll_ioctlsocket` command.
    #[non_exhaustive]
    pub enum IoctlCommand: u32 {
        /// `FIONBIO`: set non-blocking mode from a nonzero argument.
        Fionbio = 0x8004_667e,
        /// `FIONREAD`: store the number of bytes ready to read.
        Fionread = 0x4004_667f,
    }
}

value_enum! {
    /// Level of a socket option.
    #[non_exhaustive]
    pub enum SocketLevel: i32 {
        /// `SOL_SOCKET`.
        Socket = 0xffff,
        /// `IPPROTO_IP`.
        Ip = 0,
        /// `IPPROTO_TCP`.
        Tcp = 6,
    }
}

/// A socket option. The numbers of different levels overlap. Names that
/// are not Winsock's are this crate's; the console's own are unknown.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SocketOption(pub i32);

/// Level [`SocketLevel::Socket`].
impl SocketOption {
    pub const SO_REUSEADDR: Self = Self(0x4);
    /// The console rejects it with `WSAENOPROTOOPT`.
    pub const SO_KEEPALIVE: Self = Self(0x8);
    pub const SO_BROADCAST: Self = Self(0x20);
    pub const SO_LINGER: Self = Self(0x80);
    pub const SO_SNDBUF: Self = Self(0x1001);
    pub const SO_RCVBUF: Self = Self(0x1002);
    pub const SO_SNDTIMEO: Self = Self(0x1005);
    pub const SO_RCVTIMEO: Self = Self(0x1006);
    /// The console rejects it with `WSAENOPROTOOPT`.
    pub const SO_ERROR: Self = Self(0x1007);
    pub const SO_TYPE: Self = Self(0x1008);
    /// Can only be set.
    pub const SO_DONTLINGER: Self = Self(!0x80);
    /// Can only be set.
    pub const SO_EXCLUSIVEADDRUSE: Self = Self(!0x4);
    /// Marks the socket insecure: its traffic bypasses XNet security. A
    /// title needs the `XEX_SYSTEM_INSECURE_SOCKETS` privilege, or setting
    /// it fails with `WSAEACCES`.
    pub const SO_INSECURE: Self = Self(0x5801);
    /// Limits the socket to the local network: `connect` and `sendto` fail
    /// with `WSAEHOSTUNREACH` for an address that is not private (10/8,
    /// 172.16/12, 192.168/16), on the console's subnet, loopback, broadcast
    /// or multicast.
    pub const SO_LAN_ONLY: Self = Self(0x5802);
}

/// Level [`SocketLevel::Tcp`], for a TCP socket. The MSS values are 88 to
/// 1460.
impl SocketOption {
    pub const TCP_NODELAY: Self = Self(0x1);
    /// The peer's receive window, scaled. Can only be read.
    pub const TCP_SEND_WINDOW: Self = Self(0x5802);
    /// The segment size of sends: 512 until the connection is established,
    /// then the peer's MSS, at most [`Self::TCP_LOCAL_MSS`].
    pub const TCP_SEND_MSS: Self = Self(0x5803);
    /// The MSS that the socket announces and the most it sends. Unless set,
    /// the connection picks 1460 for an address on the console's subnet,
    /// 1324 for another, and 1264 for an XNet secure address.
    pub const TCP_LOCAL_MSS: Self = Self(0x5804);
    /// Nonzero offers selective acknowledgments (RFC 2018), which the
    /// connection uses if the peer permits them. Can only be set.
    pub const TCP_SACK: Self = Self(0x5805);
}

/// Level [`SocketLevel::Ip`].
impl SocketOption {
    /// Joins a multicast group: the value is the group address (the start
    /// of an `ip_mreq`). The socket must be bound and not TCP; it joins at
    /// most four groups. Can only be set.
    pub const IP_ADD_MEMBERSHIP: Self = Self(5);
    /// Leaves a multicast group that [`Self::IP_ADD_MEMBERSHIP`] joined.
    /// Can only be set.
    pub const IP_DROP_MEMBERSHIP: Self = Self(6);
    /// The time to live of sent packets, 1 to 255; 64 by default.
    pub const IP_TTL: Self = Self(7);
    /// The DSCP of sent packets: one byte, 0 to 63.
    pub const IP_DSCP: Self = Self(0x64);
}

flags! {
    /// `MSG_*` flags of sends and receives.
    pub struct MessageFlags(u32) {
        OOB = 0x1,
        PEEK = 0x2,
        DONTROUTE = 0x4,
    }
}

flags! {
    /// `FD_*` network events of `NetDll_WSAEventSelect`.
    pub struct NetworkEvents(u32) {
        READ = 0x1,
        WRITE = 0x2,
        OOB = 0x4,
        ACCEPT = 0x8,
        CONNECT = 0x10,
        CLOSE = 0x20,
    }
}

/// Sockets per select set.
pub const FD_SETSIZE: usize = 64;

/// A Winsock version (`MAKEWORD(major, minor)`) of `NetDll_WSAStartup`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WinsockVersion(u16);

impl WinsockVersion {
    pub const fn new(major: u8, minor: u8) -> Self {
        Self(((minor as u16) << 8) | major as u16)
    }
}

/// `WSADATA`, as `NetDll_WSAStartup` stores it.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WsaData {
    pub version: WinsockVersion,
    pub high_version: WinsockVersion,
    pub description: [u8; 257],
    pub system_status: [u8; 129],
    pub max_sockets: u16,
    pub max_udp_datagram: u16,
    pub vendor_info: u32,
}

impl WsaData {
    /// Zeroed storage for `NetDll_WSAStartup` to fill.
    pub const fn new() -> Self {
        Self {
            version: WinsockVersion(0),
            high_version: WinsockVersion(0),
            description: [0; 257],
            system_status: [0; 129],
            max_sockets: 0,
            max_udp_datagram: 0,
            vendor_info: 0,
        }
    }
}

impl Default for WsaData {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    assert!(core::mem::offset_of!(WsaData, max_sockets) == 0x186);
    assert!(core::mem::size_of::<WsaData>() == 0x190);
};

/// IPv4 socket address. Port and address are in network byte order.
#[repr(C)]
pub struct SockAddrIn {
    pub family: u16,
    pub port: u16,
    pub address: u32,
    pub zero: [u8; 8],
}

/// Winsock descriptor set.
#[repr(C)]
pub struct FdSet {
    pub count: u32,
    pub sockets: [Socket; FD_SETSIZE],
}

impl FdSet {
    pub const fn new() -> Self {
        Self {
            count: 0,
            sockets: [Socket(0); FD_SETSIZE],
        }
    }
}

impl Default for FdSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Winsock time interval.
#[repr(C)]
pub struct TimeVal {
    pub seconds: i32,
    pub microseconds: i32,
}

impl TimeVal {
    pub const fn from_milliseconds(milliseconds: u32) -> Self {
        Self {
            seconds: (milliseconds / 1000) as i32,
            microseconds: ((milliseconds % 1000) * 1000) as i32,
        }
    }
}
