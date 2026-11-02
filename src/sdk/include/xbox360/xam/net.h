/*
 * Winsock, XNet, HTTP and UPnP exports of xam.xex (NetDll_*) and the Xbox
 * LIVE logon exports (XNetLogon*).
 *
 * Every NetDll_* export takes an XNCALLER first. Multi-byte socket
 * addresses and ports are in network byte order.
 */

#ifndef XBOX360_XAM_NET_H
#define XBOX360_XAM_NET_H

#include <xbox360/xam/xamdef.h>

XBOX360_EXTERN_C_START

/* On whose behalf a network call is made. */
typedef enum _XNCALLER {
    XNCALLER_TITLE = 1,
    XNCALLER_SYSAPP = 2,
    XNCALLER_XBDM = 3,
} XNCALLER;

/*
 * Winsock
 */

/* A socket, or INVALID_SOCKET from a call that failed. */
typedef ULONG SOCKET, *PSOCKET;

#define INVALID_SOCKET ((SOCKET)~0)
/* Result of a call that has no count to return: 0 or this. */
#define SOCKET_ERROR   (-1)

/* Address family. */
#define AF_INET 2

/* Socket type. */
#define SOCK_STREAM 1
#define SOCK_DGRAM  2

/* Protocol, also an option level. */
#define IPPROTO_TCP 6
#define IPPROTO_UDP 17

/* Option level of IP options. */
#define IPPROTO_IP 0

/* What NetDll_shutdown disables. */
#define SD_RECEIVE 0
#define SD_SEND    1
#define SD_BOTH    2

/* NetDll_ioctlsocket commands. */
/* Sets non-blocking mode from a nonzero argument. */
#define FIONBIO  0x8004667EUL
/* Stores the number of bytes ready to read. */
#define FIONREAD 0x4004667FUL

/* Option level of socket options. */
#define SOL_SOCKET 0xFFFF

/* Options of level SOL_SOCKET. The console rejects SO_KEEPALIVE and
   SO_ERROR with WSAENOPROTOOPT. SO_DONTLINGER and SO_EXCLUSIVEADDRUSE can
   only be set. */
#define SO_REUSEADDR        0x0004
#define SO_KEEPALIVE        0x0008
#define SO_BROADCAST        0x0020
#define SO_LINGER           0x0080
#define SO_SNDBUF           0x1001
#define SO_RCVBUF           0x1002
#define SO_SNDTIMEO         0x1005
#define SO_RCVTIMEO         0x1006
#define SO_ERROR            0x1007
#define SO_TYPE             0x1008
#define SO_DONTLINGER       (~SO_LINGER)
#define SO_EXCLUSIVEADDRUSE (~SO_REUSEADDR)

/* Options of the console only. Their names are this SDK's; the official ones
   are unknown. */

/* Level SOL_SOCKET. */
/* Marks the socket insecure: its traffic bypasses XNet security. A title
   needs the XEX_SYSTEM_INSECURE_SOCKETS privilege, or setting it fails with
   WSAEACCES. */
#define SO_INSECURE 0x5801
/* Limits the socket to the local network: connect and sendto fail with
   WSAEHOSTUNREACH for an address that is not private (10/8, 172.16/12,
   192.168/16), on the console's subnet, loopback, broadcast or multicast. */
#define SO_LAN_ONLY 0x5802

/* Level IPPROTO_TCP. A TCP socket only; the MSS values are 88 to 1460. */
/* The peer's receive window, scaled. Can only be read. */
#define TCP_SEND_WINDOW 0x5802
/* The segment size of sends: 512 until the connection is established, then
   the peer's MSS, at most TCP_LOCAL_MSS. */
#define TCP_SEND_MSS    0x5803
/* The MSS that the socket announces and the most it sends. Unless set, the
   connection picks 1460 for an address on the console's subnet, 1324 for
   another, and 1264 for an XNet secure address. */
#define TCP_LOCAL_MSS   0x5804
/* Nonzero offers selective acknowledgments (RFC 2018), which the connection
   uses if the peer permits them. Can only be set. */
#define TCP_SACK        0x5805

/* Level IPPROTO_IP. The DSCP of sent packets: one byte, 0 to 63. */
#define IP_DSCP 0x64

/* Option of level IPPROTO_TCP. */
#define TCP_NODELAY 0x0001

/* Options of level IPPROTO_IP, with their Winsock 1 numbers. IP_TTL takes 1
   to 255. The membership options take the group address (the start of an
   ip_mreq) on a bound socket that is not TCP, at most four groups per socket;
   they can only be set. */
#define IP_ADD_MEMBERSHIP  5
#define IP_DROP_MEMBERSHIP 6
#define IP_TTL             7

/* Flags of sends and receives. */
#define MSG_OOB       0x1
#define MSG_PEEK      0x2
#define MSG_DONTROUTE 0x4

/* Network events of NetDll_WSAEventSelect. */
#define FD_READ    0x01
#define FD_WRITE   0x02
#define FD_OOB     0x04
#define FD_ACCEPT  0x08
#define FD_CONNECT 0x10
#define FD_CLOSE   0x20

/* Sockets per select set. */
#define FD_SETSIZE 64

/* WSADATA, as NetDll_WSAStartup stores it. */
typedef struct WSAData {
    WORD wVersion;
    WORD wHighVersion;
    CHAR szDescription[257];
    CHAR szSystemStatus[129];
    USHORT iMaxSockets;
    USHORT iMaxUdpDg;
    PCHAR lpVendorInfo;
} WSADATA, *PWSADATA, *LPWSADATA;

C_ASSERT(__builtin_offsetof(WSADATA, iMaxSockets) == 0x186);
C_ASSERT(sizeof(WSADATA) == 0x190);

/* IPv4 address. */
typedef struct in_addr {
    ULONG s_addr;
} IN_ADDR, *PIN_ADDR;

C_ASSERT(sizeof(IN_ADDR) == 4);

/* Generic socket address. */
typedef struct sockaddr {
    USHORT sa_family;
    CHAR sa_data[14];
} SOCKADDR, *PSOCKADDR;

C_ASSERT(sizeof(SOCKADDR) == 16);

/* IPv4 socket address. */
typedef struct sockaddr_in {
    USHORT sin_family;
    USHORT sin_port;
    struct in_addr sin_addr;
    CHAR sin_zero[8];
} SOCKADDR_IN, *PSOCKADDR_IN;

C_ASSERT(sizeof(SOCKADDR_IN) == 16);

/* Descriptor set of NetDll_select. */
typedef struct fd_set {
    UINT fd_count;
    SOCKET fd_array[FD_SETSIZE];
} fd_set, *PFD_SET;

C_ASSERT(sizeof(fd_set) == 0x104);

/* Time interval. */
typedef struct timeval {
    LONG tv_sec;
    LONG tv_usec;
} TIMEVAL, *PTIMEVAL;

C_ASSERT(sizeof(TIMEVAL) == 8);

/*
 * XNet
 */

/* NetDll_XNetGetTitleXnAddr and NetDll_XNetGetDebugXnAddr results. */
#define XNET_GET_XNADDR_PENDING      0x0000
#define XNET_GET_XNADDR_NONE         0x0001
#define XNET_GET_XNADDR_ETHERNET     0x0002
#define XNET_GET_XNADDR_STATIC       0x0004
#define XNET_GET_XNADDR_DHCP         0x0008
#define XNET_GET_XNADDR_PPPOE        0x0010
#define XNET_GET_XNADDR_GATEWAY      0x0020
#define XNET_GET_XNADDR_DNS          0x0040
#define XNET_GET_XNADDR_ONLINE       0x0080
#define XNET_GET_XNADDR_TROUBLESHOOT 0x8000

/* NetDll_XNetGetEthernetLinkStatus flags. */
#define XNET_ETHERNET_LINK_ACTIVE      0x01
#define XNET_ETHERNET_LINK_100MBPS     0x02
#define XNET_ETHERNET_LINK_10MBPS      0x04
#define XNET_ETHERNET_LINK_FULL_DUPLEX 0x08
#define XNET_ETHERNET_LINK_HALF_DUPLEX 0x10

/* XNetStartupParams flags. */
/* Accept insecure connections, as development kits allow. */
#define XNET_STARTUP_BYPASS_SECURITY 0x01

/* Options of NetDll_XNetGetOpt and NetDll_XNetSetOpt. */
/* The XNetStartupParams the stack was started with. */
#define XNET_OPTID_STARTUP_PARAMS         0x0001
/* A ULONGLONG that changes with network traffic. */
#define XNET_OPTID_ACTIVITY_COUNTER       0x0004
/* Promiscuous receive. */
#define XNET_OPTID_PROMISCUOUS            0x1390
/* Link flags of the network interface. */
#define XNET_OPTID_LINK_FLAGS             0x1395
/* Reports the secondary network interface. */
#define XNET_OPTID_SECONDARY_NIC          0x2F00
/* A ULONG state of the USB wireless adapter; bit 0 is set while attached. */
#define XNET_OPTID_WIRELESS_ADAPTER_STATE 0x2AF9
/* The wireless security type and key, in a 0x48-byte record. */
#define XNET_OPTID_WIRELESS_KEY           0x2AFA
/* The wireless adapter information record. */
#define XNET_OPTID_WIRELESS_ADAPTER_INFO  0x2AFC
/* A count followed by at most ten wireless security capabilities. */
#define XNET_OPTID_WIRELESS_CAPABILITIES  0x2AFD
/* A 0x24-byte record with the SSID length and 32 SSID bytes. */
#define XNET_OPTID_WIRELESS_SSID          0x0D010102
/* A ULONG BSS type: 0 ad-hoc, 1 infrastructure. */
#define XNET_OPTID_WIRELESS_BSS_TYPE      0x0D010108
/* A ULONG radio mode: 1 802.11b, 2 802.11a, 3 802.11g, 4 any. */
#define XNET_OPTID_WIRELESS_MODE          0x0D010204
/* A LONG: received signal strength of the wireless adapter in dBm. */
#define XNET_OPTID_WIRELESS_RSSI          0x0D010206
/* Variable-length wireless scan records. */
#define XNET_OPTID_WIRELESS_SCAN_RESULTS  0x0D010217
/* Starts a wireless network scan. WSAEINPROGRESS means one is active. */
#define XNET_OPTID_WIRELESS_SCAN_START    0x0D01021A

/*
 * Network address of a console. Addresses and the port are in network byte
 * order.
 */
typedef struct _XNADDR {
    /* IP address; zero without a static or DHCP address. */
    IN_ADDR ina;
    /* Online IP address; zero when not online. */
    IN_ADDR inaOnline;
    WORD wPortOnline;
    BYTE abEnet[6];
    BYTE abOnline[20];
} XNADDR, *PXNADDR;

C_ASSERT(sizeof(XNADDR) == 0x24);

/*
 * Network stack settings of NetDll_XNetStartup. A zero field selects the
 * default of the network stack.
 */
typedef struct _XNetStartupParams {
    /* sizeof(XNetStartupParams), 13. */
    BYTE cfgSizeOfStruct;
    /* XNET_STARTUP_* flags. */
    BYTE cfgFlags;
    BYTE cfgSockMaxDgramSockets;
    BYTE cfgSockMaxStreamSockets;
    BYTE cfgSockDefaultRecvBufsizeInK;
    BYTE cfgSockDefaultSendBufsizeInK;
    BYTE cfgKeyRegMax;
    BYTE cfgSecRegMax;
    BYTE cfgQosDataLimitDiv4;
    BYTE cfgQosProbeTimeoutInSeconds;
    BYTE cfgQosProbeRetries;
    BYTE cfgQosSrvMaxSimultaneousResponses;
    BYTE cfgQosPairWaitTimeInSeconds;
} XNetStartupParams, *PXNetStartupParams;

C_ASSERT(__builtin_offsetof(XNetStartupParams, cfgSockDefaultRecvBufsizeInK) == 4);
C_ASSERT(sizeof(XNetStartupParams) == 13);

/*
 * Saved network configuration, the 0x1EC-byte record of
 * NetDll_XnpLoadConfigParams, NetDll_XnpSaveConfigParams and
 * NetDll_XnpConfig. Fields without a known meaning are reserved; a caller
 * zeroes them.
 */
typedef struct _XNetConfigParams {
    BYTE Reserved0[0x4C];
    /* Meaning of each bit unknown; a debug monitor sets 0xC with a static
       address, subnet and gateway. */
    USHORT Flags;
    BYTE Reserved1[6];
    /* Static IP address, network byte order. */
    ULONG Address;
    /* Subnet mask, network byte order. */
    ULONG Subnet;
    /* Default gateway, network byte order. */
    ULONG Gateway;
    BYTE Reserved2[8];
    /* DHCP host name, NUL-terminated. */
    CHAR HostName[0x184];
} XNetConfigParams, *PXNetConfigParams;

C_ASSERT(__builtin_offsetof(XNetConfigParams, Flags) == 0x4C);
C_ASSERT(__builtin_offsetof(XNetConfigParams, Address) == 0x54);
C_ASSERT(__builtin_offsetof(XNetConfigParams, Subnet) == 0x58);
C_ASSERT(__builtin_offsetof(XNetConfigParams, Gateway) == 0x5C);
C_ASSERT(__builtin_offsetof(XNetConfigParams, HostName) == 0x68);
C_ASSERT(sizeof(XNetConfigParams) == 0x1EC);

/*
 * Winsock exports
 */

/** Initializes Winsock. VersionRequested is MAKEWORD(major, minor). */
XAM_IMPORT(0x001) INT NetDll_WSAStartup(XNCALLER Caller, WORD VersionRequested, PWSADATA Data);

/** Creates a socket. Returns the socket or INVALID_SOCKET. */
XAM_IMPORT(0x003) SOCKET NetDll_socket(XNCALLER Caller, INT AddressFamily, INT Type, INT Protocol);

/** Closes a socket. */
XAM_IMPORT(0x004) INT NetDll_closesocket(XNCALLER Caller, SOCKET Socket);

/** Disables sends, receives or both (SD_*) on a socket. */
XAM_IMPORT(0x005) INT NetDll_shutdown(XNCALLER Caller, SOCKET Socket, INT How);

/** Controls a socket mode. */
XAM_IMPORT(0x006) INT NetDll_ioctlsocket(XNCALLER Caller, SOCKET Socket, ULONG Command, PULONG Argument);

/** Sets a socket option. */
XAM_IMPORT(0x007) INT NetDll_setsockopt(XNCALLER Caller, SOCKET Socket, INT Level, INT OptionName,
                                        PCSTR OptionValue, INT OptionLength);

/** Reads a socket option. */
XAM_IMPORT(0x008) INT NetDll_getsockopt(XNCALLER Caller, SOCKET Socket, INT Level, INT OptionName,
                                        PCHAR OptionValue, PINT OptionLength);

/** Stores the local address of a socket. */
XAM_IMPORT(0x009) INT NetDll_getsockname(XNCALLER Caller, SOCKET Socket, struct sockaddr *Name,
                                         PINT NameLength);

/** Stores the remote address of a connected socket. */
XAM_IMPORT(0x00A) INT NetDll_getpeername(XNCALLER Caller, SOCKET Socket, struct sockaddr *Name,
                                         PINT NameLength);

/** Binds a socket to a local address. */
XAM_IMPORT(0x00B) INT NetDll_bind(XNCALLER Caller, SOCKET Socket, const struct sockaddr *Name,
                                  INT NameLength);

/** Connects a socket to a remote address. */
XAM_IMPORT(0x00C) INT NetDll_connect(XNCALLER Caller, SOCKET Socket, const struct sockaddr *Name,
                                     INT NameLength);

/** Places a socket in the listening state. */
XAM_IMPORT(0x00D) INT NetDll_listen(XNCALLER Caller, SOCKET Socket, INT Backlog);

/** Accepts a connection. Returns the new socket or INVALID_SOCKET. */
XAM_IMPORT(0x00E) SOCKET NetDll_accept(XNCALLER Caller, SOCKET Socket, struct sockaddr *Address,
                                       PINT AddressLength);

/** Waits until sockets are readable, writable or in error. Returns the number of ready sockets. */
XAM_IMPORT(0x00F) INT NetDll_select(XNCALLER Caller, INT Count, fd_set *ReadFds, fd_set *WriteFds,
                                    fd_set *ExceptFds, const struct timeval *Timeout);

/** Receives from a connected socket. Returns the byte count or SOCKET_ERROR. */
XAM_IMPORT(0x012) INT NetDll_recv(XNCALLER Caller, SOCKET Socket, PCHAR Buffer, INT Length, INT Flags);

/** Receives a datagram and its source address. */
XAM_IMPORT(0x014) INT NetDll_recvfrom(XNCALLER Caller, SOCKET Socket, PCHAR Buffer, INT Length, INT Flags,
                                      struct sockaddr *From, PINT FromLength);

/** Sends on a connected socket. Returns the byte count or SOCKET_ERROR. */
XAM_IMPORT(0x016) INT NetDll_send(XNCALLER Caller, SOCKET Socket, PCSTR Buffer, INT Length, INT Flags);

/** Sends a datagram to an address. */
XAM_IMPORT(0x018) INT NetDll_sendto(XNCALLER Caller, SOCKET Socket, PCSTR Buffer, INT Length, INT Flags,
                                    const struct sockaddr *To, INT ToLength);

/** Returns the last Winsock error of the current thread. */
XAM_IMPORT(0x01B) INT NetDll_WSAGetLastError(XNCALLER Caller);

/**
 * Associates network events (FD_*) of a socket with an event object. A NULL
 * EventObject and no events end the association.
 */
XAM_IMPORT(0x023) INT NetDll_WSAEventSelect(XNCALLER Caller, SOCKET Socket, HANDLE EventObject,
                                            LONG NetworkEvents);

/*
 * XNet exports
 */

/** Initializes the network stack. Params may be NULL. */
XAM_IMPORT(0x033) INT NetDll_XNetStartup(XNCALLER Caller, const XNetStartupParams *Params);

/** Fills a buffer with random bytes. */
XAM_IMPORT(0x035) INT NetDll_XNetRandom(XNCALLER Caller, PBYTE Buffer, UINT Length);

/** Stores the title XNADDR. Returns XNET_GET_XNADDR_* flags. */
XAM_IMPORT(0x049) DWORD NetDll_XNetGetTitleXnAddr(XNCALLER Caller, PXNADDR Address);

/** Stores the debug XNADDR. Returns XNET_GET_XNADDR_* flags. */
XAM_IMPORT(0x04A) DWORD NetDll_XNetGetDebugXnAddr(XNCALLER Caller, PXNADDR Address);

/** Returns the link status (XNET_ETHERNET_LINK_*) of the network interface. */
XAM_IMPORT(0x04B) DWORD NetDll_XNetGetEthernetLinkStatus(XNCALLER Caller);

/** Reads a network stack option (XNET_OPTID_*). */
XAM_IMPORT(0x04E) INT NetDll_XNetGetOpt(XNCALLER Caller, DWORD OptionId, PBYTE Value, PINT ValueLength);

/** Writes a network stack option (XNET_OPTID_*). */
XAM_IMPORT(0x04F) INT NetDll_XNetSetOpt(XNCALLER Caller, DWORD OptionId, PCBYTE Value, DWORD ValueLength);

/**
 * NetDll_XNetStartup with a third argument. The 2.0.6717 dashboard calls it
 * in place of NetDll_XNetStartup when XamGetSystemVersion is 0x200A3200 or
 * more, with Version 0x2017F400, and treats the result as that of
 * NetDll_XNetStartup.
 */
XAM_IMPORT(0x050) INT NetDll_XNetStartupVersion(XNCALLER Caller, const XNetStartupParams *Params,
                                                DWORD Version);

/** Reads the saved network configuration, a 0x1EC-byte record, into Params. */
XAM_IMPORT(0x065) HRESULT NetDll_XnpLoadConfigParams(XNCALLER Caller, PXNetConfigParams Params);

/**
 * Saves the 0x1EC-byte network configuration record Params. Returns
 * E_PENDING while the save has not finished; the 2.0.6717 dashboard calls it
 * again until it does.
 */
XAM_IMPORT(0x066) HRESULT NetDll_XnpSaveConfigParams(XNCALLER Caller, const XNetConfigParams *Params);

/** Applies a network configuration. The meaning of Flags is unknown; a debug monitor passes 0. */
XAM_IMPORT(0x068) INT NetDll_XnpConfig(XNCALLER Caller, const XNetConfigParams *Config, DWORD Flags);

/** Stores the network configuration status. The layout of the record is unknown. */
XAM_IMPORT(0x069) INT NetDll_XnpGetConfigStatus(XNCALLER Caller, PVOID Status);

/**
 * Installs or removes Ethernet frame transmit and receive callbacks. The
 * callback signatures and the meaning of each Flags bit are unknown; a debug
 * monitor's network simulator sets 0x2 and a frame capture sets none.
 */
XAM_IMPORT(0x06D) INT NetDll_XnpEthernetInterceptSetCallbacks(XNCALLER Caller, PVOID Transmit, PVOID Receive,
                                                              PVOID Context, DWORD Flags);

/** Transmits a raw Ethernet frame. */
XAM_IMPORT(0x06E) DWORD NetDll_XnpEthernetInterceptXmit(XNCALLER Caller, PBYTE Frame, DWORD Length);

/** Delivers a raw Ethernet frame to the network stack. */
XAM_IMPORT(0x06F) DWORD NetDll_XnpEthernetInterceptRecv(XNCALLER Caller, PBYTE Frame, DWORD Length);

/** Stores the list of open sockets. Count holds the capacity on entry and the count on return. */
XAM_IMPORT(0x07A) DWORD NetDll_XnpGetActiveSocketList(XNCALLER Caller, PVOID Buffer, PDWORD Count);

/** Informs the network stack that the system time changed. */
XAM_IMPORT(0x07B) DWORD NetDll_XnpNoteSystemTime(XNCALLER Caller);

/*
 * HTTP exports. Session, connection and request handles are DWORD values; 0
 * means failure.
 */

/** Initializes the HTTP client. */
XAM_IMPORT(0x0C9) BOOL NetDll_XHttpStartup(XNCALLER Caller, DWORD Reserved, DWORD Parameters);

/** Shuts down the HTTP client. */
XAM_IMPORT(0x0CA) BOOL NetDll_XHttpShutdown(XNCALLER Caller);

/** Opens an HTTP session. Returns the session handle or 0. */
XAM_IMPORT(0x0CB) DWORD NetDll_XHttpOpen(XNCALLER Caller, PCSTR UserAgent, DWORD AccessType, DWORD ProxyName,
                                         DWORD ProxyBypass, DWORD Flags);

/** Closes an HTTP handle. */
XAM_IMPORT(0x0CC) BOOL NetDll_XHttpCloseHandle(XNCALLER Caller, DWORD Handle);

/** Opens an HTTP connection to a server. Returns the connection handle or 0. */
XAM_IMPORT(0x0CD) DWORD NetDll_XHttpConnect(XNCALLER Caller, DWORD Session, PCSTR ServerName, DWORD ServerPort,
                                            DWORD Flags);

/** Installs a status callback on an HTTP handle. The callback signature is unknown. */
XAM_IMPORT(0x0CE) DWORD NetDll_XHttpSetStatusCallback(XNCALLER Caller, DWORD Handle, DWORD Callback,
                                                      DWORD NotificationFlags, DWORD Reserved);

/** Opens an HTTP request. Returns the request handle or 0. */
XAM_IMPORT(0x0CF) DWORD NetDll_XHttpOpenRequest(XNCALLER Caller, DWORD Connection, PCSTR Verb, PCSTR ObjectName,
                                                DWORD Version, DWORD Referrer, DWORD AcceptTypes, DWORD Flags);

/** Sends an HTTP request. */
XAM_IMPORT(0x0D1) BOOL NetDll_XHttpSendRequest(XNCALLER Caller, DWORD Request, DWORD Headers, DWORD HeadersLength,
                                               DWORD Optional, DWORD OptionalLength, DWORD TotalLength,
                                               DWORD Context);

/** Waits for the response to an HTTP request. */
XAM_IMPORT(0x0D2) BOOL NetDll_XHttpReceiveResponse(XNCALLER Caller, DWORD Request, DWORD Reserved);

/** Reads response header information. */
XAM_IMPORT(0x0D3) BOOL NetDll_XHttpQueryHeaders(XNCALLER Caller, DWORD Request, DWORD InfoLevel, DWORD Name,
                                                PDWORD Buffer, PDWORD BufferLength, DWORD Index);

/** Reads response body data. The byte count is stored at BytesRead. */
XAM_IMPORT(0x0D4) BOOL NetDll_XHttpReadData(XNCALLER Caller, DWORD Request, PVOID Buffer, DWORD Length,
                                            PDWORD BytesRead);

/** Sets an option on an HTTP handle. */
XAM_IMPORT(0x0D7) BOOL NetDll_XHttpSetOption(XNCALLER Caller, DWORD Handle, DWORD Option, PDWORD Buffer,
                                             DWORD BufferLength);

/** Processes pending HTTP work for a session. */
XAM_IMPORT(0x0D8) BOOL NetDll_XHttpDoWork(XNCALLER Caller, DWORD Session, DWORD Timeout);

/*
 * UPnP exports
 */

/** Starts the UPnP client. */
XAM_IMPORT(0x0FB) INT NetDll_UpnpStartup(XNCALLER Caller);

/** Stops the UPnP client. */
XAM_IMPORT(0x0FC) INT NetDll_UpnpCleanup(XNCALLER Caller);

/*
 * Xbox LIVE logon exports
 */

/** Stores the Xbox LIVE logon status and extended status. */
XAM_IMPORT(0x13B) DWORD XNetLogonGetExtendedStatus(PDWORD Status, PDWORD ExtendedStatus);

/**
 * Discards the cached Xbox LIVE tickets. The 2.0.6717 dashboard passes 0; the
 * meaning of the argument is unknown.
 */
XAM_IMPORT(0x13C) DWORD XNetLogonClearTicketCaches(DWORD Unknown);

/* Ordinal 0x139 from Xenia's export table; 0x1C6 is XamGetWCNConfigFile. */
/** Stores a NUL-terminated Xbox LIVE service DNS name in Buffer. */
XAM_IMPORT(0x139) DWORD XNetLogonGetDnsString(DWORD Index, PCHAR Buffer, DWORD Capacity);

XBOX360_EXTERN_C_END

#endif
