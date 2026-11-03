//! Structures of the kernel cryptography exports.

/// `XeCryptDesKey` key schedule: 16 rounds of two words.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DesState(pub [u32; 32]);

const _: () = assert!(core::mem::size_of::<DesState>() == 0x80);

/// Console type field of the console certificate, as `XeKeysGetConsoleType`
/// stores it. The low 24 bits are a type code (1 on a development kit);
/// which codes mean a development kit, a test kit or a retail console is
/// not established.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CertificateConsoleType(pub u32);

impl CertificateConsoleType {
    /// Bit set when the certificate is signed with public key 0x3A rather
    /// than 0x3C.
    pub const KEY_3A: u32 = 0x8000_0000;

    /// The low 24 bits.
    pub const fn code(self) -> u32 {
        self.0 & 0x00ff_ffff
    }

    /// Whether the certificate is signed with public key 0x3A rather than
    /// 0x3C.
    pub const fn signed_with_key_3a(self) -> bool {
        self.0 & Self::KEY_3A != 0
    }
}
