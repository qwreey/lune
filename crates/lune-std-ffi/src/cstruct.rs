#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use libffi::low::ffi_abi_FFI_DEFAULT_ABI;
use libffi::middle::{Cif, Type};
use libffi::raw::ffi_get_struct_offsets;
use std::vec::Vec;

use crate::association::{get_association, set_association};
use crate::ctype::{libffi_types_from_table, type_name_from_userdata};

use super::ctype::CType;

pub struct CStruct {
    libffi_cif: Cif,
    libffi_type: Type,
    fields: Vec<Type>,
    offsets: Vec<usize>,
    size: usize,
}

const CSTRUCT_INNER: &str = "__cstruct_inner";

impl CStruct {
    pub fn new(fields: Vec<Type>) -> Self {
        let libffi_type = Type::structure(fields.clone());
        let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());
        let size = unsafe { (*libffi_type.as_raw_ptr()).size };
        let mut offsets = Vec::<usize>::with_capacity(fields.len());
        unsafe {
            ffi_get_struct_offsets(
                ffi_abi_FFI_DEFAULT_ABI,
                libffi_type.as_raw_ptr(),
                offsets.as_mut_ptr(),
            );
            offsets.set_len(offsets.capacity());
        }

        Self {
            libffi_cif: libffi_cfi,
            libffi_type,
            fields,
            offsets,
            size,
        }
    }

    pub fn from_lua_table<'lua>(
        lua: &'lua Lua,
        table: LuaTable<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let fields = libffi_types_from_table(&table)?;
        let cstruct = lua.create_userdata(Self::new(fields))?;
        table.set_readonly(true);
        set_association(lua, CSTRUCT_INNER, cstruct.clone(), table)?;
        Ok(cstruct)
    }

    // Stringify cstruct for pretty printing something like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(userdata: &LuaAnyUserData) -> LuaResult<String> {
        let field: LuaValue = userdata.get("inner")?;
        if field.is_table() {
            let table = field
                .as_table()
                .ok_or(LuaError::external("failed to get inner table."))?;

            // iterate for field
            let mut result = String::from(" ");
            for i in 0..table.raw_len() {
                let child: LuaAnyUserData = table.raw_get(i + 1)?;
                result.push_str(format!("{}, ", type_name_from_userdata(&child)?).as_str());
            }

            // size of
            result.push_str(format!("size = {} ", userdata.borrow::<CStruct>()?.size).as_str());
            Ok(result)
        } else {
            Ok(String::from("unnamed"))
        }
    }

    pub fn get_type(&self) -> Type {
        self.libffi_type.clone()
    }

    // Get byte offset of nth field
    pub fn offset(&self, index: usize) -> LuaResult<usize> {
        let offset = self
            .offsets
            .get(index)
            .ok_or(LuaError::external("Out of index"))?
            .to_owned();
        Ok(offset)
    }
}

impl LuaUserData for CStruct {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));

        // Simply pass in the locked table used when first creating this object.
        // By strongly referencing the table, the types inside do not disappear
        // and the user can read the contents as needed. (good recycling!)
        fields.add_field_function_get("inner", |lua, this: LuaAnyUserData| {
            let table: LuaValue = get_association(lua, CSTRUCT_INNER, this)?
                // It shouldn't happen.
                .ok_or(LuaError::external("inner field not found"))?;
            Ok(table)
        });
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("offset", |_, this, index: usize| {
            let offset = this.offset(index)?;
            Ok(offset)
        });
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CType::pointer(lua, &this)?;
            Ok(pointer)
        });
        methods.add_meta_function(LuaMetaMethod::ToString, |_, this: LuaAnyUserData| {
            let result = CStruct::stringify(&this)?;
            Ok(result)
        });
    }
}