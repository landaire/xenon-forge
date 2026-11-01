use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroU16;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

/// An export ordinal, as `XexGetProcedureAddress` takes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ordinal(NonZeroU16);

impl Ordinal {
    /// `None` for 0, which no export has.
    pub const fn new(ordinal: u16) -> Option<Self> {
        match NonZeroU16::new(ordinal) {
            Some(ordinal) => Some(Self(ordinal)),
            None => None,
        }
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

/// A data import slot. Before any code of the module runs, the loader
/// replaces the import record in the slot with the address of the export,
/// a `T` that the kernel keeps for the life of the system. Safe code can
/// only read the slot.
#[repr(transparent)]
pub struct ImportedVariable<T: 'static> {
    address: AtomicU32,
    _export: PhantomData<&'static T>,
}

impl<T> ImportedVariable<T>
where
    T: Sync + 'static,
{
    /// The export. `None` when the loader left the import unresolved:
    /// Xenia stores 0xDxxxBEEF in the slots of exports it does not provide,
    /// 0xF00DF00D for an ordinal it does not list, and 0xDEADC0DE for a
    /// function export imported as a variable.
    pub fn get(&self) -> Option<&'static T> {
        let address = resolved_address(self.address.load(Ordering::Relaxed))?;

        // SAFETY: the loader stored the address of the export, a `T` that
        // the kernel keeps for the life of the system.
        Some(unsafe { &*core::ptr::with_exposed_provenance::<T>(address as usize) })
    }

    /// The export, without the check of [`ImportedVariable::get`]. For code
    /// that checks the import once, for example at module start, and then
    /// reads it often.
    ///
    /// # Safety
    ///
    /// The loader must have resolved the import: [`ImportedVariable::get`]
    /// returns `Some` for it. On the console kernel the loader resolves
    /// every import it accepts.
    #[inline]
    pub unsafe fn get_unchecked(&self) -> &'static T {
        let address = self.address.load(Ordering::Relaxed);

        // SAFETY: the caller guarantees that the loader stored the address
        // of the export, a `T` that the kernel keeps for the life of the
        // system.
        unsafe { &*core::ptr::with_exposed_provenance::<T>(address as usize) }
    }
}

impl<T> fmt::Debug for ImportedVariable<T>
where
    T: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let address = self.address.load(Ordering::Relaxed);

        write!(f, "ImportedVariable({address:#010x})")
    }
}

/// Expand to the given items when this crate is built for the console
/// target (or for its documentation with `--cfg xbox360_docs`), and to
/// nothing otherwise. The macros of this crate gate what they generate
/// through it, so the crate that invokes them needs no cfg of its own.
#[cfg(any(target_os = "xbox360", xbox360_docs))]
#[doc(hidden)]
#[macro_export]
macro_rules! __on_console {
    ($($item:tt)*) => {
        $($item)*
    };
}

#[cfg(not(any(target_os = "xbox360", xbox360_docs)))]
#[doc(hidden)]
#[macro_export]
macro_rules! __on_console {
    ($($item:tt)*) => {};
}

/// Declare a typed import: an `extern "C"` function that links to the
/// function of the thunk library with the same name, and a compile-time
/// check that its arguments and result pass like those of the C
/// declaration in [`ffi`](crate::ffi): the same count, variadic or not,
/// and per value the same class (no value, 4 bytes or less, or the size).
///
/// ```ignore
/// import!(
///     /// Closes a handle.
///     pub NtClose(handle: Handle) -> NtStatus
/// );
/// ```
///
/// With `safe` after the visibility, as in an `unsafe extern` block, the
/// function is safe to call. Declare an import `safe` only when no argument
/// is an address, handle or object that the export uses or releases, and
/// the export has no effect that can break the memory safety of other
/// code. The check cannot tell an integer from a floating-point value of
/// the same size.
macro_rules! import {
    ($(#[$meta:meta])* $vis:vis safe $name:ident($($arg:ident: $argty:ty),* $(,)?) -> !) => {
        $crate::import::import!(@declare [$(#[$meta])*] [$vis safe] $name($($arg: $argty),*) [-> !] [$($arg)*] [!] [0]);
    };
    ($(#[$meta:meta])* $vis:vis $name:ident($($arg:ident: $argty:ty),* $(,)?) -> !) => {
        $crate::import::import!(@declare [$(#[$meta])* #[doc = $crate::import::import!(@safety)]] [$vis] $name($($arg: $argty),*) [-> !] [$($arg)*] [!] [0]);
    };
    ($(#[$meta:meta])* $vis:vis safe $name:ident($($arg:ident: $argty:ty),* $(,)?) -> $ret:ty) => {
        $crate::import::import!(@declare [$(#[$meta])*] [$vis safe] $name($($arg: $argty),*) [-> $ret] [$($arg)*] [_] [$crate::signature::class(::core::mem::size_of::<$ret>())]);
    };
    ($(#[$meta:meta])* $vis:vis $name:ident($($arg:ident: $argty:ty),+, ...) -> $ret:ty) => {
        $crate::__on_console! {
            unsafe extern "C" {
                $(#[$meta])*
                #[doc = $crate::import::import!(@safety)]
                #[allow(clashing_extern_declarations)]
                $vis fn $name($($arg: $argty),+, ...) -> $ret;
            }
            $crate::import::import!(@check $name [$($arg: $argty),+] [$($crate::import::import!(@infer $arg),)+ ...] [_] [$crate::signature::class(::core::mem::size_of::<$ret>())]);
        }
    };
    ($(#[$meta:meta])* $vis:vis $name:ident($($arg:ident: $argty:ty),* $(,)?) -> $ret:ty) => {
        $crate::import::import!(@declare [$(#[$meta])* #[doc = $crate::import::import!(@safety)]] [$vis] $name($($arg: $argty),*) [-> $ret] [$($arg)*] [_] [$crate::signature::class(::core::mem::size_of::<$ret>())]);
    };
    (@declare [$($attr:tt)*] [$($qualifiers:tt)*] $name:ident($($arg:ident: $argty:ty),*) [$($ret:tt)*] [$($infer:ident)*] [$cast:ty] [$class:expr]) => {
        $crate::__on_console! {
            unsafe extern "C" {
                $($attr)*
                #[allow(clashing_extern_declarations)]
                $($qualifiers)* fn $name($($arg: $argty),*) $($ret)*;
            }
            $crate::import::import!(@check $name [$($arg: $argty),*] [$($crate::import::import!(@infer $infer)),*] [$cast] [$class]);
        }
    };
    (@check $name:ident [$($arg:ident: $argty:ty),*] [$($infer:tt)*] [$cast:ty] [$class:expr]) => {
        const _: () = ::core::assert!(
            $crate::signature::matches(
                &($crate::ffi::$name as unsafe extern "C" fn($($infer)*) -> $cast),
                &[$($crate::signature::class(::core::mem::size_of::<$argty>())),*],
                $class,
            ),
            ::core::concat!("the arguments or result of ", ::core::stringify!($name), " differ from its C declaration"),
        );
    };
    (@infer $arg:ident) => {
        _
    };
    (@safety) => {
        "\n\n# Safety\n\nThe arguments and the calling context must meet the requirements of the export: \
         each address is valid for the accesses the export makes for as long as it uses it, \
         and each handle or object is one that the caller may use or release as the export does."
    };
}
pub(crate) use import;

/// Declare a typed data import `$name: $type`: an [`ImportedVariable`]
/// view of the slot `$slot` that the thunk library defines and
/// [`ffi`](crate::ffi) declares. The loader replaces the import record in
/// the slot with the address of the export.
///
/// ```ignore
/// import_variable!(
///     /// Version of the running kernel.
///     pub safe XboxKrnlVersion: KernelVersion,
///     slot = __imp_XboxKrnlVersion
/// );
/// ```
///
/// The static is `unsafe` to read. With `safe` after the visibility, as in
/// an `unsafe extern` block, it is safe to read, and [`ImportedVariable`]
/// then hands out the export as a `&'static $type` without `unsafe`.
/// Declare an import `safe` only when `$type` describes the export as the
/// kernel keeps it: a valid value for every bit pattern the kernel stores,
/// and an atomic for every field that the kernel or another processor
/// changes.
macro_rules! import_variable {
    ($(#[$meta:meta])* $vis:vis safe $name:ident: $type:ty, slot = $slot:ident) => {
        $crate::import::import_variable!(@slot [$(#[$meta])*] [$vis safe] $name: $type, $slot);
    };
    ($(#[$meta:meta])* $vis:vis $name:ident: $type:ty, slot = $slot:ident) => {
        $crate::import::import_variable!(@slot [$(#[$meta])*] [$vis] $name: $type, $slot);
    };
    (@slot [$($attr:tt)*] [$($qualifiers:tt)*] $name:ident: $type:ty, $slot:ident) => {
        $crate::__on_console! {
            unsafe extern "C" {
                $($attr)*
                #[link_name = ::core::stringify!($slot)]
                #[allow(clashing_extern_declarations)]
                $($qualifiers)* static $name: $crate::ImportedVariable<$type>;
            }
            // The slot is declared in the headers.
            const _: () = {
                let _ = || unsafe { $crate::ffi::$slot };
            };
        }
    };
}
pub(crate) use import_variable;

/// `address` unless it is a value Xenia leaves in the slot of a data import
/// it does not resolve.
const fn resolved_address(address: u32) -> Option<u32> {
    if address & 0xf000_ffff == 0xd000_beef || address == 0xf00d_f00d || address == 0xdead_c0de {
        None
    } else {
        Some(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_data_import_has_no_address() {
        assert_eq!(resolved_address(0xd000_beef), None);
        assert_eq!(resolved_address(0xd123_beef), None);
        assert_eq!(resolved_address(0x801f_beef), Some(0x801f_beef));
        assert_eq!(resolved_address(0xf00d_f00d), None);
        assert_eq!(resolved_address(0xdead_c0de), None);
    }
}
