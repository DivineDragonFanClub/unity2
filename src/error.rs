use core::fmt;

pub type Il2CppResult<T> = ::core::result::Result<T, Il2CppError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Il2CppError {
    MissingClass {
        namespace: String,
        name: String,
    },
    MissingClassForType,
    MissingMethod {
        class: String,
        method: String,
        param_count: usize,
    },
    AmbiguousMethod {
        class: String,
        method: String,
        param_count: usize,
        overload_count: usize,
    },
    MissingField {
        class: String,
        field: String,
    },
    VtableIndexOutOfRange {
        class: String,
        index: usize,
        vtable_len: usize,
    },
    FailedInstantiation {
        class: String,
    },
    FailedArrayInstantiation,
    FailedGenericInstantiation {
        class: String,
    },
    FailedMethodInvocation {
        method: String,
    },
    FailedReflectionQuerying,
}

impl fmt::Display for Il2CppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingClass { namespace, name } => {
                write!(f, "class `{}.{}` not found", namespace, name)
            }
            Self::MissingClassForType => {
                f.write_str("could not resolve Il2CppType to an Il2CppClass")
            }
            Self::MissingMethod {
                class,
                method,
                param_count,
            } => write!(f, "method `{}::{}` with {} parameters not found", class, method, param_count),
            Self::AmbiguousMethod {
                class,
                method,
                param_count,
                overload_count,
            } => write!(
                f,
                "method `{}::{}` has {} overloads with {} parameters, disambiguate via offset or vtable_index",
                class, method, overload_count, param_count
            ),
            Self::MissingField { class, field } => {
                write!(f, "field `{}` not found on class `{}`", field, class)
            }
            Self::VtableIndexOutOfRange {
                class,
                index,
                vtable_len,
            } => write!(
                f,
                "vtable index {} out of range for `{}`, vtable has {} slots",
                index, class, vtable_len
            ),
            Self::FailedInstantiation { class } => {
                write!(f, "IL2CPP allocator returned null for `{}`", class)
            }
            Self::FailedArrayInstantiation => {
                f.write_str("IL2CPP array allocator returned null")
            }
            Self::FailedGenericInstantiation { class } => {
                write!(f, "IL2CPP generic instantiation failed for `{}`", class)
            }
            Self::FailedMethodInvocation { method } => {
                write!(f, "IL2CPP method `{}` returned null", method)
            }
            Self::FailedReflectionQuerying => {
                f.write_str("could not construct a System.Type reflection object")
            }
        }
    }
}

impl std::error::Error for Il2CppError {}
