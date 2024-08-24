use libffi::middle::Type;
use mlua::prelude::*;

use crate::association::{get_association, set_association};
use crate::cptr::CPtr;
use crate::ctype::{
    libffi_type_ensured_size, libffi_type_from_userdata, type_userdata_stringify, CType,
};

// This is a series of some type.
// It provides the final size and the offset of the index,
// but does not allow multidimensional arrays because of API complexity.
// However, multidimensional arrays are not impossible to implement
// because they are a series of transcribed one-dimensional arrays.

// See: https://stackoverflow.com/a/43525176

// Padding after each field inside the struct is set to next field can follow the alignment.
// There is no problem even if you create a struct with n fields of a single type within the struct. Array adheres to the condition that there is no additional padding between each element. Padding to a struct is padding inside the struct. Simply think of the padding byte as a trailing unnamed field.

const CARR_INNER: &str = "__carr_inner";

pub struct CArr {
    libffi_type: Type,
    struct_type: Type,
    length: usize,
    field_size: usize,
    size: usize,
}

impl CArr {
    pub fn new(libffi_type: Type, length: usize) -> LuaResult<Self> {
        let struct_type = Type::structure(vec![libffi_type.clone(); length]);
        let field_size = libffi_type_ensured_size(libffi_type.as_raw_ptr())?;

        Ok(Self {
            libffi_type,
            struct_type,
            length,
            field_size,
            size: field_size * length,
        })
    }

    pub fn from_lua_userdata<'lua>(
        lua: &'lua Lua,
        luatype: &LuaAnyUserData<'lua>,
        length: usize,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let fields = libffi_type_from_userdata(luatype)?;
        let carr = lua.create_userdata(Self::new(fields, length)?)?;

        set_association(lua, CARR_INNER, carr.clone(), luatype)?;
        Ok(carr)
    }

    pub fn get_type(&self) -> Type {
        self.libffi_type.clone()
    }

    // Stringify cstruct for pretty printing something like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(userdata: &LuaAnyUserData) -> LuaResult<String> {
        let inner: LuaValue = userdata.get("inner")?;
        let carr = userdata.borrow::<CArr>()?;
        if inner.is_userdata() {
            let inner = inner
                .as_userdata()
                .ok_or(LuaError::external("failed to get inner type userdata."))?;
            Ok(format!(
                " {} ; {} ",
                type_userdata_stringify(inner)?,
                carr.length
            ))
        } else {
            Err(LuaError::external("failed to get inner type userdata."))
        }
    }
}

impl LuaUserData for CArr {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
        fields.add_field_method_get("length", |_, this| Ok(this.length));
        fields.add_field_function_get("inner", |lua, this: LuaAnyUserData| {
            let inner: LuaValue = get_association(lua, CARR_INNER, this)?
                // It shouldn't happen.
                .ok_or(LuaError::external("inner field not found"))?;
            Ok(inner)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("offset", |_, this, offset: isize| {
            if this.length > (offset as usize) && offset >= 0 {
                Ok(this.field_size * (offset as usize))
            } else {
                Err(LuaError::external("Out of index"))
            }
        });
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::from_lua_userdata(lua, &this)?;
            Ok(pointer)
        });
        methods.add_meta_function(LuaMetaMethod::ToString, |_, this: LuaAnyUserData| {
            let result = CArr::stringify(&this)?;
            Ok(result)
        });
    }
}
