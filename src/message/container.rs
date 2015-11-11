// This file is generated. Do not edit
// @generated

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(Clone,Default)]
pub struct Container {
    // message fields
    kind: ::std::option::Option<Kind>,
    payload: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl Container {
    pub fn new() -> Container {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Container {
        static mut instance: ::protobuf::lazy::Lazy<Container> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Container,
        };
        unsafe {
            instance.get(|| {
                Container {
                    kind: ::std::option::Option::None,
                    payload: ::protobuf::SingularField::none(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // optional .message.Kind kind = 1;

    pub fn clear_kind(&mut self) {
        self.kind = ::std::option::Option::None;
    }

    pub fn has_kind(&self) -> bool {
        self.kind.is_some()
    }

    // Param is passed by value, moved
    pub fn set_kind(&mut self, v: Kind) {
        self.kind = ::std::option::Option::Some(v);
    }

    pub fn get_kind<'a>(&self) -> Kind {
        self.kind.unwrap_or(Kind::NodeAddMessage)
    }

    // optional bytes payload = 2;

    pub fn clear_payload(&mut self) {
        self.payload.clear();
    }

    pub fn has_payload(&self) -> bool {
        self.payload.is_some()
    }

    // Param is passed by value, moved
    pub fn set_payload(&mut self, v: ::std::vec::Vec<u8>) {
        self.payload = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_payload<'a>(&'a mut self) -> &'a mut ::std::vec::Vec<u8> {
        if self.payload.is_none() {
            self.payload.set_default();
        };
        self.payload.as_mut().unwrap()
    }

    // Take field
    pub fn take_payload(&mut self) -> ::std::vec::Vec<u8> {
        self.payload.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_payload<'a>(&'a self) -> &'a [u8] {
        match self.payload.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }
}

impl ::protobuf::Message for Container {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_enum());
                    self.kind = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.payload.set_default();
                    try!(is.read_bytes_into(tmp))
                },
                _ => {
                    let unknown = try!(is.read_unknown(wire_type));
                    self.mut_unknown_fields().add_value(field_number, unknown);
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.kind.iter() {
            my_size += ::protobuf::rt::enum_size(1, *value);
        };
        for value in self.payload.iter() {
            my_size += ::protobuf::rt::bytes_size(2, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.kind {
            try!(os.write_enum(1, v as i32));
        };
        if let Some(v) = self.payload.as_ref() {
            try!(os.write_bytes(2, &v));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields<'s>(&'s self) -> &'s ::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields<'s>(&'s mut self) -> &'s mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<Container>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Container {
    fn new() -> Container {
        Container::new()
    }

    fn descriptor_static(_: ::std::option::Option<Container>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_enum_accessor(
                    "kind",
                    Container::has_kind,
                    Container::get_kind,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_bytes_accessor(
                    "payload",
                    Container::has_payload,
                    Container::get_payload,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Container>(
                    "Container",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Container {
    fn clear(&mut self) {
        self.clear_kind();
        self.clear_payload();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for Container {
    fn eq(&self, other: &Container) -> bool {
        self.kind == other.kind &&
        self.payload == other.payload &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for Container {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Kind {
    NodeAddMessage = 1,
}

impl ::protobuf::ProtobufEnum for Kind {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Kind> {
        match value {
            1 => ::std::option::Option::Some(Kind::NodeAddMessage),
            _ => ::std::option::Option::None
        }
    }

    fn enum_descriptor_static(_: Option<Kind>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("Kind", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for Kind {
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x1b, 0x73, 0x72, 0x63, 0x2f, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2f, 0x63, 0x6f,
    0x6e, 0x74, 0x61, 0x69, 0x6e, 0x65, 0x72, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x07, 0x6d,
    0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x22, 0x39, 0x0a, 0x09, 0x43, 0x6f, 0x6e, 0x74, 0x61, 0x69,
    0x6e, 0x65, 0x72, 0x12, 0x1b, 0x0a, 0x04, 0x6b, 0x69, 0x6e, 0x64, 0x18, 0x01, 0x20, 0x01, 0x28,
    0x0e, 0x32, 0x0d, 0x2e, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2e, 0x4b, 0x69, 0x6e, 0x64,
    0x12, 0x0f, 0x0a, 0x07, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0x18, 0x02, 0x20, 0x01, 0x28,
    0x0c, 0x2a, 0x1a, 0x0a, 0x04, 0x4b, 0x69, 0x6e, 0x64, 0x12, 0x12, 0x0a, 0x0e, 0x4e, 0x6f, 0x64,
    0x65, 0x41, 0x64, 0x64, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x10, 0x01, 0x4a, 0xf5, 0x01,
    0x0a, 0x06, 0x12, 0x04, 0x00, 0x00, 0x09, 0x01, 0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x00,
    0x08, 0x0f, 0x0a, 0x0a, 0x0a, 0x02, 0x05, 0x00, 0x12, 0x04, 0x02, 0x00, 0x04, 0x01, 0x0a, 0x0a,
    0x0a, 0x03, 0x05, 0x00, 0x01, 0x12, 0x03, 0x02, 0x05, 0x09, 0x0a, 0x0b, 0x0a, 0x04, 0x05, 0x00,
    0x02, 0x00, 0x12, 0x03, 0x03, 0x02, 0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x05, 0x00, 0x02, 0x00, 0x01,
    0x12, 0x03, 0x03, 0x02, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x05, 0x00, 0x02, 0x00, 0x02, 0x12, 0x03,
    0x03, 0x13, 0x14, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x00, 0x12, 0x04, 0x06, 0x00, 0x09, 0x01, 0x0a,
    0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01, 0x12, 0x03, 0x06, 0x08, 0x11, 0x0a, 0x0b, 0x0a, 0x04, 0x04,
    0x00, 0x02, 0x00, 0x12, 0x03, 0x07, 0x02, 0x19, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00,
    0x04, 0x12, 0x03, 0x07, 0x02, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x06, 0x12,
    0x03, 0x07, 0x0b, 0x0f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x07,
    0x10, 0x14, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x07, 0x17, 0x18,
    0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x08, 0x02, 0x1d, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x01, 0x04, 0x12, 0x03, 0x08, 0x02, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x00, 0x02, 0x01, 0x05, 0x12, 0x03, 0x08, 0x0b, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02,
    0x01, 0x01, 0x12, 0x03, 0x08, 0x11, 0x18, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x03,
    0x12, 0x03, 0x08, 0x1b, 0x1c,
];

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
