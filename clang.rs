use std::libc::*;
use std::{cast, ptr, str, to_bytes, uint, vec};

pub use ll = clangll;
use clangll::*;

// Cursor
pub struct Cursor(CXCursor);

pub type CursorVisitor<'self> = &'self fn(c: &Cursor, p: &Cursor) -> Enum_CXChildVisitResult;

impl Cursor {
    // common
    pub fn spelling(&self) -> ~str {
        unsafe {
            String(clang_getCursorSpelling(**self)).to_str()
        }
    }

    pub fn kind(&self) -> Enum_CXCursorKind {
        unsafe {
            clang_getCursorKind(**self)
        }
    }

    pub fn location(&self) -> SourceLocation {
        unsafe {
            SourceLocation(clang_getCursorLocation(**self))
        }
    }

    pub fn cur_type(&self) -> Type {
        unsafe {
            Type(clang_getCursorType(**self))
        }
    }

    pub fn definition(&self) -> Cursor {
        unsafe {
            Cursor(clang_getCursorDefinition(**self))
        }
    }

    pub fn visit(&self, func: CursorVisitor) {
        unsafe {
            let data = cast::transmute::<&CursorVisitor, CXClientData>(&func);
            clang_visitChildren(**self, visit_children, data);
        };
    }

    // enum
    pub fn enum_type(&self) -> Type {
        unsafe {
            Type(clang_getEnumDeclIntegerType(**self))
        }
    }

    pub fn enum_val(&self) -> int {
        unsafe {
            clang_getEnumConstantDeclValue(**self) as int
        }
    }

    // typedef
    pub fn typedef_type(&self) -> Type {
        unsafe {
            Type(clang_getTypedefDeclUnderlyingType(**self))
        }
    }

    // function, variable
    pub fn linkage(&self) -> Enum_CXLinkageKind {
        unsafe {
            clang_getCursorLinkage(**self)
        }
    }

    // function
    pub fn args(&self) -> ~[Cursor] {
        unsafe {
            let num = clang_Cursor_getNumArguments(**self) as uint;
            let mut args = ~[];
            for uint::range(0, num) |i| {
                args.push(Cursor(clang_Cursor_getArgument(**self, i as c_uint)));
            }
            return args;
        }
    }

    pub fn ret_type(&self) -> Type {
        unsafe {
            Type(clang_getCursorResultType(**self))
        }
    }
}

extern fn visit_children(cur: CXCursor, parent: ll::CXCursor,
                         data: CXClientData) -> ll::Enum_CXChildVisitResult {
    unsafe {
        let func = cast::transmute::<CXClientData, &CursorVisitor>(data);
        return (*func)(&Cursor(cur), &Cursor(parent));
    }
}

impl Eq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        unsafe {
            clang_equalCursors(**self, **other) == 1
        }
    }

    fn ne(&self, other: &Cursor) -> bool {
        return !self.eq(other);
    }
}

impl IterBytes for Cursor {
    fn iter_bytes(&self, lsb0: bool, f: to_bytes::Cb) -> bool {
        [(self.kind as int),
         (self.xdata as int),
         (self.data[0] as int),
         (self.data[1] as int),
         (self.data[2] as int)
        ].iter_bytes(lsb0, f)
    }
}

// type
pub struct Type(CXType);

impl Type {
    // common
    pub fn kind(&self) -> Enum_CXTypeKind {
        return (*self).kind;
    }

    pub fn declaration(&self) -> Cursor {
        unsafe {
            Cursor(clang_getTypeDeclaration(**self))
        }
    }

    pub fn is_const(&self) -> bool {
        unsafe {
            clang_isConstQualifiedType(**self) == 1
        }
    }

    // pointer
    pub fn pointee_type(&self) -> Type {
        unsafe {
            Type(clang_getPointeeType(**self))
        }
    }

    // array
    pub fn elem_type(&self) -> Type {
        unsafe {
            Type(clang_getArrayElementType(**self))
        }
    }

    pub fn array_size(&self) -> uint {
        unsafe {
            clang_getArraySize(**self) as uint
        }
    }

    // typedef
    pub fn canonical_type(&self) -> Type {
        unsafe {
            Type(clang_getCanonicalType(**self))
        }
    }

    // function
    pub fn is_variadic(&self) -> bool {
        unsafe {
            clang_isFunctionTypeVariadic(**self) == 1
        }
    }

    pub fn arg_types(&self) -> ~[Type] {
        unsafe {
            let num = clang_getNumArgTypes(**self) as uint;
            let mut args = ~[];
            for uint::range(0, num) |i| {
                args.push(Type(clang_getArgType(**self, i as c_uint)));
            }
            return args;
        }
    }

    pub fn ret_type(&self) -> Type {
        unsafe {
            Type(clang_getResultType(**self))
        }
    }
}

// SourceLocation
pub struct SourceLocation(CXSourceLocation);

impl SourceLocation {
    pub fn location(&self) -> (File, uint, uint, uint) {
        unsafe {
            let mut file = ptr::mut_null();
            let mut line = 0;
            let mut col = 0;
            let mut off = 0;
            clang_getSpellingLocation(**self, ptr::to_mut_unsafe_ptr(&mut file),
                                          ptr::to_mut_unsafe_ptr(&mut line),
                                          ptr::to_mut_unsafe_ptr(&mut col),
                                          ptr::to_mut_unsafe_ptr(&mut off));
            return (File(file), line as uint, col as uint, off as uint);
        }
    }
}

// File
pub struct File(CXFile);

impl File {
    pub fn name(&self) -> ~str {
        unsafe {
            String(clang_getFileName(**self)).to_str()
        }
    }
}

// String
pub struct String(CXString);

impl ToStr for String {
    fn to_str(&self) -> ~str {
        unsafe {
            let c_str = clang_getCString(**self) as *c_char;
            str::raw::from_c_str(c_str)
        }
    }
}

// Index
pub struct Index(CXIndex);

impl Index {
    pub fn create(pch: bool, diag: bool) -> Index {
        unsafe {
            Index(clang_createIndex(pch as c_int, diag as c_int))
        }
    }

    pub fn dispose(&self) {
        unsafe {
            clang_disposeIndex(**self);
        }
    }
}

// TranslationUnit
pub struct TranslationUnit(CXTranslationUnit);

impl TranslationUnit {
    pub fn parse(ix: &Index, file: &str, cmd_args: &[~str],
                 unsaved: &[UnsavedFile], opts: uint) -> TranslationUnit {
        let fname = str::as_c_str(file, |f| f);
        let c_args = cmd_args.map(|s| str::as_c_str(*s, |cs| cs));
        let mut c_unsaved = unsaved.map(|f| **f);
        let tu = unsafe {
            clang_parseTranslationUnit(**ix, fname,
                                       vec::raw::to_ptr(c_args),
                                       c_args.len() as c_int,
                                       vec::raw::to_mut_ptr(c_unsaved),
                                       c_unsaved.len() as c_uint,
                                       opts as c_uint)
        };
        TranslationUnit(tu)
    }

    pub fn diags(&self) -> ~[Diagnostic] {
        unsafe {
            let num = clang_getNumDiagnostics(**self) as uint;
            let mut diags = ~[];
            for uint::range(0, num) |i| {
                diags.push(Diagnostic(clang_getDiagnostic(**self, i as c_uint)));
            }
            return diags;
        }
    }

    pub fn cursor(&self) -> Cursor {
        unsafe {
            Cursor(clang_getTranslationUnitCursor(**self))
        }
    }

    pub fn dispose(&self) {
        unsafe {
            clang_disposeTranslationUnit(**self);
        }
    }
}

// Diagnostic
pub struct Diagnostic(CXDiagnostic);

impl Diagnostic {
    pub fn default_opts() -> uint {
        unsafe {
            clang_defaultDiagnosticDisplayOptions() as uint
        }
    }

    pub fn format(&self, opts: uint) -> ~str {
        unsafe {
            String(clang_formatDiagnostic(**self, opts as c_uint)).to_str()
        }
    }

    pub fn severity(&self) -> Enum_CXDiagnosticSeverity {
        unsafe {
            clang_getDiagnosticSeverity(**self)
        }
    }
}

// UnsavedFile
pub struct UnsavedFile(Struct_CXUnsavedFile);
