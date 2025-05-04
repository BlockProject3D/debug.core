// Copyright (c) 2025, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum FieldValue<'a> {
    Int(i64),
    UInt(u64),
    Float(f32),
    Double(f64),
    String(&'a str),
    Debug(&'a dyn Debug),
    Boolean(bool),
}

impl<'a> Display for FieldValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Int(v) => write!(f, "{}", v),
            FieldValue::UInt(v) => write!(f, "{}", v),
            FieldValue::Float(v) => write!(f, "{}", v),
            FieldValue::Double(v) => write!(f, "{}", v),
            FieldValue::String(v) => f.write_str(v),
            FieldValue::Debug(v) => write!(f, "{:?}", v),
            FieldValue::Boolean(v) => write!(f, "{:?}", v),
        }
    }
}

pub struct Field<'a> {
    name: &'a str,
    value: FieldValue<'a>,
}

impl<'a> Field<'a> {
    pub fn new(name: &'a str, value: impl Into<FieldValue<'a>>) -> Self {
        Self {
            name,
            value: value.into(),
        }
    }

    pub fn new_debug(name: &'a str, value: &'a dyn Debug) -> Self {
        Self {
            name,
            value: FieldValue::Debug(value),
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn value(&self) -> &FieldValue<'a> {
        &self.value
    }
}

macro_rules! impl_into_field_value {
    // Would've preferred expr, but turns out expr is useless in macros, so let's not use it.
    ($($t: ty => $func: ident),*) => {
        $(
            impl<'a> From<$t> for FieldValue<'a> {
                fn from(value: $t) -> Self {
                    FieldValue::$func(value as _)
                }
            }
        )*
    };
}

impl_into_field_value! {
    u8 => UInt,
    u16 => UInt,
    u32 => UInt,
    u64 => UInt,
    usize => UInt,
    i8 => Int,
    i16 => Int,
    i32 => Int,
    i64 => Int,
    isize => Int,
    f32 => Float,
    f64 => Double,
    bool => Boolean
}

impl<'a> From<&'a str> for FieldValue<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(value)
    }
}

pub struct FieldSet<'a, const N: usize>([Field<'a>; N]);

impl<'a, const N: usize> FieldSet<'a, N> {
    pub fn new(fields: [Field<'a>; N]) -> Self {
        Self(fields)
    }
}

impl<'a, const N: usize> AsRef<[Field<'a>]> for FieldSet<'a, N> {
    fn as_ref(&self) -> &[Field<'a>] {
        &self.0
    }
}

#[macro_export]
macro_rules! field {
    ($name: ident) => {
        $crate::field::Field::new(stringify!($name), $name)
    };
    (?$name: ident) => {
        $crate::field::Field::new_debug(stringify!($name), &$name)
    };
    ($name: ident = $value: expr) => {
        $crate::field::Field::new(stringify!($name), $value)
    };
    ($name: ident = ?$value: expr) => {
        $crate::field::Field::new_debug(stringify!($name), &$value)
    };
}

#[macro_export]
macro_rules! fields {
    ($({$($field: tt)*})*) => {
        [$(
            $crate::field!($($field)*),
        )*]
    };
}
