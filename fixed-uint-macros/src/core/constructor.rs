// Copyright 2018 Rust-NumExt Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Constructor for Uint.

use proc_macro2::TokenStream;
use std::cell::Cell;
use std::iter::FromIterator;

use parsed;

use super::utils;

pub struct UintInformation {
    pub name: String,
    pub derived: bool,
    pub bits_size: u64,
    pub bytes_size: u64,
    pub unit_bits_size: u64,
    pub unit_bytes_size: u64,
    pub unit_amount: u64,
}

impl ::std::convert::From<parsed::UintDefinition> for UintInformation {
    fn from(data: parsed::UintDefinition) -> Self {
        let parsed::UintDefinition { name, attrs } = data;
        let derived = attrs.derived;
        // bits size for the whole unsigned integer
        let bits_size = attrs.size;
        // bytes size for the whole unsigned integer
        let bytes_size = attrs.size / 8;
        // bits size for the unit of the unsigned integer
        let unit_bits_size = attrs.unit_size;
        // bytes size for the unit of the unsigned integer
        let unit_bytes_size = attrs.unit_size / 8;
        // how many units in an unsigned integer
        let unit_amount = attrs.size / attrs.unit_size;
        Self {
            name,
            derived,
            bits_size,
            bytes_size,
            unit_bits_size,
            unit_bytes_size,
            unit_amount,
        }
    }
}

pub struct UintTokenStreams {
    pub name: TokenStream,
    pub bits_size: TokenStream,
    pub bytes_size: TokenStream,
    pub unit_bits_size: TokenStream,
    pub unit_bytes_size: TokenStream,
    pub unit_amount: TokenStream,
    pub unit_suffix: TokenStream,
    pub double_unit_suffix: TokenStream,
    pub inner_type: TokenStream,
    pub bytes_type: TokenStream,
    pub error_name: TokenStream,
}

impl<'a> ::std::convert::From<&'a UintInformation> for UintTokenStreams {
    fn from(info: &UintInformation) -> Self {
        let name = utils::ident_to_ts(info.name.as_ref());
        let bits_size = utils::pure_uint_to_ts(info.bits_size);
        let bytes_size = utils::pure_uint_to_ts(info.bytes_size);
        let unit_bits_size = utils::pure_uint_to_ts(info.unit_bits_size);
        let unit_bytes_size = utils::pure_uint_to_ts(info.unit_bytes_size);
        let unit_amount = utils::pure_uint_to_ts(info.unit_amount);

        let unit_suffix = utils::uint_suffix_to_ts(info.unit_bits_size);
        let double_unit_suffix = utils::uint_suffix_to_ts(info.unit_bits_size * 2);

        let inner_type = quote!([#unit_suffix; #unit_amount]);
        let bytes_type = quote!([u8; #bytes_size]);

        let error_name = utils::ident_to_ts("FixedUintError");

        Self {
            name,
            bits_size,
            bytes_size,
            unit_bits_size,
            unit_bytes_size,
            unit_amount,
            unit_suffix,
            double_unit_suffix,
            inner_type,
            bytes_type,
            error_name,
        }
    }
}

pub struct UintConstructor {
    // Raw data of uint definition
    pub info: UintInformation,
    // Cache TokenStreams
    pub ts: UintTokenStreams,

    // Outputs
    uint_common: Cell<Vec<TokenStream>>,
    // Outputs (define methods)
    defuns: Cell<Vec<TokenStream>>,
    // Outputs (implement traits)
    implts: Cell<Vec<TokenStream>>,

    // Outputs
    common: Cell<Vec<TokenStream>>,
    // Outputs (errors)
    errors: Cell<Vec<TokenStream>>,
}

impl UintConstructor {
    pub fn new(data: parsed::UintDefinition) -> Self {
        let info: UintInformation = data.into();
        let ts: UintTokenStreams = (&info).into();
        let uint_common = Cell::new(Vec::new());
        let defuns = Cell::new(Vec::new());
        let implts = Cell::new(Vec::new());
        let common = Cell::new(Vec::new());
        let errors = Cell::new(Vec::new());
        UintConstructor {
            info,
            ts,
            uint_common,
            defuns,
            implts,
            common,
            errors,
        }
    }

    fn defstruct(&self) {
        if self.info.derived {
            return;
        }
        let name = &self.ts.name;
        let inner_type = &self.ts.inner_type;
        let part = quote!(
            pub struct #name (#inner_type);
        );
        self.attach_uint(part);
    }

    fn deferror(&self) {
        let error_name = &self.ts.error_name;
        let part = {
            let errors = self.errors.take();
            if errors.is_empty() {
                quote!()
            } else {
                let errors = TokenStream::from_iter(errors);
                quote!(
                    #[derive(Debug, Fail)]
                    pub enum #error_name {
                        #errors
                    }
                )
            }
        };
        self.attach_common(part);
    }

    pub fn output(&self) -> (TokenStream, TokenStream) {
        self.defstruct();
        self.deferror();
        let name = &self.ts.name;
        let uint_common = TokenStream::from_iter(self.uint_common.take());
        let defuns = TokenStream::from_iter(self.defuns.take());
        let implts = TokenStream::from_iter(self.implts.take());
        let one_uint = quote!(
            #uint_common

            impl #name {
                #defuns
            }

            #implts
        );
        let common = TokenStream::from_iter(self.common.take());
        (one_uint, common)
    }

    pub fn clear(&self) {
        let _ = self.uint_common.take();
        let _ = self.defuns.take();
        let _ = self.implts.take();
        let _ = self.common.take();
        let _ = self.errors.take();
    }

    pub fn attach_uint(&self, part: TokenStream) {
        let mut o = self.uint_common.take();
        o.push(part);
        self.uint_common.set(o);
    }

    pub fn defun(&self, part: TokenStream) {
        let mut o = self.defuns.take();
        o.push(part);
        self.defuns.set(o);
    }

    pub fn implt(&self, part: TokenStream) {
        let mut o = self.implts.take();
        o.push(part);
        self.implts.set(o);
    }

    pub fn attach_common(&self, part: TokenStream) {
        let mut o = self.common.take();
        o.push(part);
        self.common.set(o);
    }

    pub fn error(&self, part: TokenStream) {
        let mut o = self.errors.take();
        o.push(part);
        self.errors.set(o);
    }
}
