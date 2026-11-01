//! Compile-time checks that a typed import matches its C declaration in
//! [`ffi`](crate::ffi).

/// How the calling convention passes a value of `size` bytes: 0 for no
/// value, 4 for an integer or pointer of 4 bytes or less, which take one
/// register, or the size itself.
pub const fn class(size: usize) -> usize {
    match size {
        1..=4 => 4,
        size => size,
    }
}

/// The argument and result classes of a function pointer type.
pub trait Signature {
    const PARAMETERS: &'static [usize];
    const RESULT: usize;
}

macro_rules! signature {
    ($($parameter:ident),*) => {
        impl<R, $($parameter),*> Signature for unsafe extern "C" fn($($parameter),*) -> R {
            const PARAMETERS: &'static [usize] = &[$(class(size_of::<$parameter>())),*];
            const RESULT: usize = class(size_of::<R>());
        }
    };
    (variadic $($parameter:ident),+) => {
        signature!($($parameter),+);
        impl<R, $($parameter),+> Signature for unsafe extern "C" fn($($parameter,)+ ...) -> R {
            const PARAMETERS: &'static [usize] = &[$(class(size_of::<$parameter>())),+];
            const RESULT: usize = class(size_of::<R>());
        }
    };
}

signature!();
signature!(variadic A);
signature!(variadic A, B);
signature!(variadic A, B, C);
signature!(variadic A, B, C, D);
signature!(variadic A, B, C, D, E);
signature!(variadic A, B, C, D, E, F);
signature!(variadic A, B, C, D, E, F, G);
signature!(variadic A, B, C, D, E, F, G, H);
signature!(variadic A, B, C, D, E, F, G, H, I);
signature!(variadic A, B, C, D, E, F, G, H, I, J);

/// Whether the C declaration `_declaration` passes its arguments and result
/// with the classes `parameters` and `result`.
pub const fn matches<F: Signature>(_declaration: &F, parameters: &[usize], result: usize) -> bool {
    if F::PARAMETERS.len() != parameters.len() || F::RESULT != result {
        return false;
    }
    let mut index = 0;
    while index < parameters.len() {
        if F::PARAMETERS[index] != parameters[index] {
            return false;
        }
        index += 1;
    }
    true
}
