//! The imports that the headers annotate, read with libclang.

use std::collections::BTreeMap;
use std::path::Path;

use clang::Clang;
use clang::Entity;
use clang::EntityKind;
use clang::Index;
use clang::Type;
use clang::TypeKind;
use clang::diagnostic::Severity;

/// Library names of `XBOX360_IMPORT` and their index in the import
/// records, the position of the library in xex-link's `LIBRARIES`.
const LIBRARIES: [(&str, u32); 3] = [("xboxkrnl", 0), ("xam", 1), ("xbdm", 2)];

const ANNOTATION: &str = "xbox360_import,";

pub struct Import {
    pub name: String,
    pub library: u32,
    pub ordinal: u16,
    pub kind: ImportKind,
}

pub enum ImportKind {
    Function(Function),
    /// A data import slot, `__imp_<export>`.
    Variable,
}

impl Import {
    /// The import record: `type<<24 | library<<16 | ordinal`, type 1 for a
    /// function thunk and 0 for a data slot.
    pub fn record(&self) -> u32 {
        let kind = match self.kind {
            ImportKind::Function(_) => 1,
            ImportKind::Variable => 0,
        };
        (kind << 24) | (self.library << 16) | u32::from(self.ordinal)
    }
}

pub struct Function {
    pub parameters: Vec<Scalar>,
    pub variadic: bool,
    pub result: Option<Scalar>,
    pub noreturn: bool,
}

/// A C argument or result, by how the calling conventions pass it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scalar {
    /// An integer, enum or pointer of 1, 2 or 4 bytes.
    Int {
        size: u8,
        signed: bool,
    },
    /// An 8-byte integer.
    Int64,
    Float,
    Double,
}

/// Every import declared in `header` or the headers under `include` that
/// it includes, sorted by name.
pub fn scan(header: &Path, include: &Path, arguments: &[String]) -> Result<Vec<Import>, String> {
    let clang = Clang::new()?;
    let index = Index::new(&clang, false, false);
    let unit = index
        .parser(header)
        .arguments(arguments)
        .skip_function_bodies(true)
        .parse()
        .map_err(|error| format!("{}: {error}", header.display()))?;

    let mut failed = false;
    for diagnostic in unit.get_diagnostics() {
        if diagnostic.get_severity() >= Severity::Warning {
            eprintln!("{diagnostic}");
            failed = true;
        }
    }
    if failed {
        return Err(format!("{} does not compile cleanly", header.display()));
    }

    let mut imports = BTreeMap::<String, Import>::new();
    let mut ordinals = BTreeMap::<(u32, u16), String>::new();
    for entity in unit.get_entity().get_children() {
        if !matches!(
            entity.get_kind(),
            EntityKind::FunctionDecl | EntityKind::VarDecl
        ) {
            continue;
        }
        let Some((library, ordinal)) = annotation(&entity)? else {
            continue;
        };
        let name = entity.get_name().ok_or("an import without a name")?;
        let location = describe(&entity, include);
        let kind = match entity.get_kind() {
            EntityKind::FunctionDecl => ImportKind::Function(
                function(&entity).map_err(|error| format!("{location}: {name}: {error}"))?,
            ),
            _ => {
                variable(&entity, &name).map_err(|error| format!("{location}: {name}: {error}"))?;
                ImportKind::Variable
            }
        };
        let import = Import {
            name: name.clone(),
            library,
            ordinal,
            kind,
        };

        if let Some(previous) = imports.get(&name) {
            if (previous.library, previous.ordinal) != (library, ordinal) {
                return Err(format!("{location}: {name} is declared with two ordinals"));
            }
            continue;
        }
        if let Some(other) = ordinals.insert((library, ordinal), name.clone()) {
            return Err(format!(
                "{location}: {name} and {other} have the same ordinal {ordinal:#x}"
            ));
        }
        imports.insert(name, import);
    }

    Ok(imports.into_values().collect())
}

fn describe(entity: &Entity, include: &Path) -> String {
    let Some(location) = entity.get_location() else {
        return "<unknown>".to_owned();
    };
    let location = location.get_file_location();
    let path = location
        .file
        .map(|file| file.get_path())
        .unwrap_or_default();
    let path = path.strip_prefix(include).unwrap_or(&path);
    format!("{}:{}", path.display(), location.line)
}

/// The library index and ordinal of an `XBOX360_IMPORT` annotation.
fn annotation(entity: &Entity) -> Result<Option<(u32, u16)>, String> {
    let Some(text) = entity
        .get_children()
        .into_iter()
        .filter(|child| child.get_kind() == EntityKind::AnnotateAttr)
        .filter_map(|child| child.get_name())
        .find(|text| text.starts_with(ANNOTATION))
    else {
        return Ok(None);
    };
    let (library, ordinal) = text[ANNOTATION.len()..]
        .split_once(',')
        .ok_or_else(|| format!("malformed annotation `{text}`"))?;
    let library = LIBRARIES
        .iter()
        .find(|(name, _)| *name == library)
        .map(|(_, index)| *index)
        .ok_or_else(|| format!("unknown import library `{library}`"))?;
    let ordinal = match ordinal
        .strip_prefix("0x")
        .or_else(|| ordinal.strip_prefix("0X"))
    {
        Some(hex) => u16::from_str_radix(hex, 16),
        None => ordinal.parse(),
    }
    .map_err(|_| format!("ordinal `{ordinal}` is not a 16-bit number"))?;
    if ordinal == 0 {
        return Err("ordinal 0".to_owned());
    }
    Ok(Some((library, ordinal)))
}

fn function(entity: &Entity) -> Result<Function, String> {
    let ty = entity.get_type().ok_or("no type")?;
    let parameters = ty
        .get_argument_types()
        .ok_or("no prototype")?
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            scalar(parameter).map_err(|error| format!("parameter {}: {error}", index + 1))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result = ty.get_result_type().ok_or("no result type")?;
    let result = if result.get_canonical_type().get_kind() == TypeKind::Void {
        None
    } else {
        Some(scalar(&result).map_err(|error| format!("result: {error}"))?)
    };
    // libclang does not expose the attribute; the type names it.
    let noreturn = ty.get_display_name().contains("__attribute__((noreturn))");
    Ok(Function {
        parameters,
        variadic: ty.is_variadic(),
        result,
        noreturn,
    })
}

fn variable(entity: &Entity, name: &str) -> Result<(), String> {
    if !name.starts_with("__imp_") {
        return Err("a data import is declared as its slot, `__imp_<export>`".to_owned());
    }
    let ty = entity.get_type().ok_or("no type")?;
    if ty.get_canonical_type().get_kind() != TypeKind::Pointer {
        return Err("the slot of a data import holds a pointer to the export".to_owned());
    }
    Ok(())
}

fn scalar(ty: &Type) -> Result<Scalar, String> {
    let canonical = ty.get_canonical_type();
    let size = canonical
        .get_sizeof()
        .map_err(|error| format!("{}: {error:?}", ty.get_display_name()))?;
    let integer = |signed: bool| match size {
        1 | 2 | 4 => Ok(Scalar::Int {
            size: size as u8,
            signed,
        }),
        8 => Ok(Scalar::Int64),
        _ => Err(format!("{}: {size}-byte integer", ty.get_display_name())),
    };
    match canonical.get_kind() {
        TypeKind::Pointer | TypeKind::BlockPointer => integer(false),
        TypeKind::Bool => integer(false),
        TypeKind::Enum => {
            let declaration = canonical
                .get_declaration()
                .ok_or("enum without a declaration")?;
            let underlying = declaration
                .get_enum_underlying_type()
                .ok_or("enum without an underlying type")?;
            integer(underlying.is_signed_integer())
        }
        TypeKind::Float => Ok(Scalar::Float),
        TypeKind::Double => Ok(Scalar::Double),
        _ if canonical.is_signed_integer() => integer(true),
        _ if canonical.is_unsigned_integer() => integer(false),
        TypeKind::Record | TypeKind::ConstantArray => Err(format!(
            "{} passes by value; pass a pointer, or the integer of its size that the console ABI uses",
            ty.get_display_name()
        )),
        kind => Err(format!(
            "{}: unsupported type kind {kind:?}",
            ty.get_display_name()
        )),
    }
}
