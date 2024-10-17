use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::CType;
use crate::ffi::{NativeConvert, NativeData, NativeSignedness};

impl NativeSignedness for CType<i128> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl NativeConvert for CType<i128> {
    unsafe fn luavalue_into<'lua>(
        &self,
        _lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn NativeData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
        let value: i128 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<i128>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(data_handle.get_pointer().byte_offset(offset).cast::<i128>()) = value;
        }
        Ok(())
    }
    unsafe fn luavalue_from<'lua>(
        &self,
        lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn NativeData>,
    ) -> LuaResult<LuaValue<'lua>> {
        let value = unsafe { (*data_handle.get_pointer().byte_offset(offset).cast::<i128>()).into_lua(lua)? };
        Ok(value)
    }
}