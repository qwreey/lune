#![allow(clippy::cargo_common_metadata)]

use std::borrow::Borrow;

use libffi::middle::Type;
use mlua::prelude::*;

use crate::association::{get_association, set_association};
use crate::carr::CArr;
use crate::ctype::{type_name_from_userdata, type_userdata_stringify};

const POINTER_INNER: &str = "__pointer_inner";

pub struct CPtr();

impl CPtr {
    // Create pointer type with '.inner' field
    // inner can be CArr, CType or CStruct
    pub fn from_lua_userdata<'lua>(
        lua: &'lua Lua,
        inner: &LuaAnyUserData,
    ) -> LuaResult<LuaValue<'lua>> {
        let value = Self().into_lua(lua)?;

        set_association(lua, POINTER_INNER, value.borrow(), inner)?;

        Ok(value)
    }

    // Stringify CPtr with inner ctype
    pub fn stringify(userdata: &LuaAnyUserData) -> LuaResult<String> {
        let inner: LuaValue = userdata.get("inner")?;

        if inner.is_userdata() {
            let inner = inner
                .as_userdata()
                .ok_or(LuaError::external("failed to get inner type userdata."))?;
            Ok(format!(
                " <{}({})> ",
                type_name_from_userdata(inner),
                type_userdata_stringify(inner)?,
            ))
        } else {
            Err(LuaError::external("failed to get inner type userdata."))
        }
    }

    // Return void*
    pub fn get_type() -> Type {
        Type::pointer()
    }
}

impl LuaUserData for CPtr {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, _| Ok(size_of::<usize>()));
        fields.add_field_function_get("inner", |lua, this| {
            let inner = get_association(lua, POINTER_INNER, this)?
                .ok_or(LuaError::external("inner type not found"))?;
            Ok(inner)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::from_lua_userdata(lua, &this)?;
            Ok(pointer)
        });
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            let carr = CArr::from_lua_userdata(lua, &this, length)?;
            Ok(carr)
        });
        methods.add_meta_function(LuaMetaMethod::ToString, |_, this: LuaAnyUserData| {
            let name: Result<String, LuaError> = CPtr::stringify(&this);
            Ok(name)
        });
    }
}
