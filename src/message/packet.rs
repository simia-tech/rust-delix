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
pub struct Packet {
    // message fields
    result: ::std::option::Option<Packet_Result>,
    message: ::protobuf::SingularField<::std::string::String>,
    payload: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl Packet {
    pub fn new() -> Packet {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Packet {
        static mut instance: ::protobuf::lazy::Lazy<Packet> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Packet,
        };
        unsafe {
            instance.get(|| {
                Packet {
                    result: ::std::option::Option::None,
                    message: ::protobuf::SingularField::none(),
                    payload: ::protobuf::SingularField::none(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // optional .message.Packet.Result result = 1;

    pub fn clear_result(&mut self) {
        self.result = ::std::option::Option::None;
    }

    pub fn has_result(&self) -> bool {
        self.result.is_some()
    }

    // Param is passed by value, moved
    pub fn set_result(&mut self, v: Packet_Result) {
        self.result = ::std::option::Option::Some(v);
    }

    pub fn get_result<'a>(&self) -> Packet_Result {
        self.result.unwrap_or(Packet_Result::Ok)
    }

    // optional string message = 2;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::string::String) {
        self.message = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.message.is_none() {
            self.message.set_default();
        };
        self.message.as_mut().unwrap()
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::string::String {
        self.message.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_message<'a>(&'a self) -> &'a str {
        match self.message.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required bytes payload = 3;

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

impl ::protobuf::Message for Packet {
    fn is_initialized(&self) -> bool {
        if self.payload.is_none() {
            return false;
        };
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
                    self.result = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.message.set_default();
                    try!(is.read_string_into(tmp))
                },
                3 => {
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
        for value in self.result.iter() {
            my_size += ::protobuf::rt::enum_size(1, *value);
        };
        for value in self.message.iter() {
            my_size += ::protobuf::rt::string_size(2, &value);
        };
        for value in self.payload.iter() {
            my_size += ::protobuf::rt::bytes_size(3, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.result {
            try!(os.write_enum(1, v as i32));
        };
        if let Some(v) = self.message.as_ref() {
            try!(os.write_string(2, &v));
        };
        if let Some(v) = self.payload.as_ref() {
            try!(os.write_bytes(3, &v));
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
        ::std::any::TypeId::of::<Packet>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Packet {
    fn new() -> Packet {
        Packet::new()
    }

    fn descriptor_static(_: ::std::option::Option<Packet>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_enum_accessor(
                    "result",
                    Packet::has_result,
                    Packet::get_result,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "message",
                    Packet::has_message,
                    Packet::get_message,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_bytes_accessor(
                    "payload",
                    Packet::has_payload,
                    Packet::get_payload,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Packet>(
                    "Packet",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Packet {
    fn clear(&mut self) {
        self.clear_result();
        self.clear_message();
        self.clear_payload();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for Packet {
    fn eq(&self, other: &Packet) -> bool {
        self.result == other.result &&
        self.message == other.message &&
        self.payload == other.payload &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for Packet {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Packet_Result {
    Ok = 1,
    NotFound = 2,
    PermissionDenied = 3,
    ConnectionRefused = 4,
    ConnectionReset = 5,
    ConnectionAborted = 6,
    NotConnected = 7,
    AddrInUse = 8,
    AddrNotAvailable = 9,
    BrokenPipe = 10,
    AlreadyExists = 11,
    WouldBlock = 12,
    InvalidInput = 13,
    InvalidData = 14,
    TimedOut = 15,
    WriteZero = 16,
    Other = 17,
    UnexpectedEof = 18,
}

impl ::protobuf::ProtobufEnum for Packet_Result {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Packet_Result> {
        match value {
            1 => ::std::option::Option::Some(Packet_Result::Ok),
            2 => ::std::option::Option::Some(Packet_Result::NotFound),
            3 => ::std::option::Option::Some(Packet_Result::PermissionDenied),
            4 => ::std::option::Option::Some(Packet_Result::ConnectionRefused),
            5 => ::std::option::Option::Some(Packet_Result::ConnectionReset),
            6 => ::std::option::Option::Some(Packet_Result::ConnectionAborted),
            7 => ::std::option::Option::Some(Packet_Result::NotConnected),
            8 => ::std::option::Option::Some(Packet_Result::AddrInUse),
            9 => ::std::option::Option::Some(Packet_Result::AddrNotAvailable),
            10 => ::std::option::Option::Some(Packet_Result::BrokenPipe),
            11 => ::std::option::Option::Some(Packet_Result::AlreadyExists),
            12 => ::std::option::Option::Some(Packet_Result::WouldBlock),
            13 => ::std::option::Option::Some(Packet_Result::InvalidInput),
            14 => ::std::option::Option::Some(Packet_Result::InvalidData),
            15 => ::std::option::Option::Some(Packet_Result::TimedOut),
            16 => ::std::option::Option::Some(Packet_Result::WriteZero),
            17 => ::std::option::Option::Some(Packet_Result::Other),
            18 => ::std::option::Option::Some(Packet_Result::UnexpectedEof),
            _ => ::std::option::Option::None
        }
    }

    fn enum_descriptor_static(_: Option<Packet_Result>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("Packet_Result", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for Packet_Result {
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x0c, 0x70, 0x61, 0x63, 0x6b, 0x65, 0x74, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x07,
    0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x22, 0x94, 0x03, 0x0a, 0x06, 0x50, 0x61, 0x63, 0x6b,
    0x65, 0x74, 0x12, 0x26, 0x0a, 0x06, 0x72, 0x65, 0x73, 0x75, 0x6c, 0x74, 0x18, 0x01, 0x20, 0x01,
    0x28, 0x0e, 0x32, 0x16, 0x2e, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2e, 0x50, 0x61, 0x63,
    0x6b, 0x65, 0x74, 0x2e, 0x52, 0x65, 0x73, 0x75, 0x6c, 0x74, 0x12, 0x0f, 0x0a, 0x07, 0x6d, 0x65,
    0x73, 0x73, 0x61, 0x67, 0x65, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09, 0x12, 0x0f, 0x0a, 0x07, 0x70,
    0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0x18, 0x03, 0x20, 0x02, 0x28, 0x0c, 0x22, 0xbf, 0x02, 0x0a,
    0x06, 0x52, 0x65, 0x73, 0x75, 0x6c, 0x74, 0x12, 0x06, 0x0a, 0x02, 0x4f, 0x6b, 0x10, 0x01, 0x12,
    0x0c, 0x0a, 0x08, 0x4e, 0x6f, 0x74, 0x46, 0x6f, 0x75, 0x6e, 0x64, 0x10, 0x02, 0x12, 0x14, 0x0a,
    0x10, 0x50, 0x65, 0x72, 0x6d, 0x69, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x44, 0x65, 0x6e, 0x69, 0x65,
    0x64, 0x10, 0x03, 0x12, 0x15, 0x0a, 0x11, 0x43, 0x6f, 0x6e, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x6f,
    0x6e, 0x52, 0x65, 0x66, 0x75, 0x73, 0x65, 0x64, 0x10, 0x04, 0x12, 0x13, 0x0a, 0x0f, 0x43, 0x6f,
    0x6e, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x52, 0x65, 0x73, 0x65, 0x74, 0x10, 0x05, 0x12,
    0x15, 0x0a, 0x11, 0x43, 0x6f, 0x6e, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x41, 0x62, 0x6f,
    0x72, 0x74, 0x65, 0x64, 0x10, 0x06, 0x12, 0x10, 0x0a, 0x0c, 0x4e, 0x6f, 0x74, 0x43, 0x6f, 0x6e,
    0x6e, 0x65, 0x63, 0x74, 0x65, 0x64, 0x10, 0x07, 0x12, 0x0d, 0x0a, 0x09, 0x41, 0x64, 0x64, 0x72,
    0x49, 0x6e, 0x55, 0x73, 0x65, 0x10, 0x08, 0x12, 0x14, 0x0a, 0x10, 0x41, 0x64, 0x64, 0x72, 0x4e,
    0x6f, 0x74, 0x41, 0x76, 0x61, 0x69, 0x6c, 0x61, 0x62, 0x6c, 0x65, 0x10, 0x09, 0x12, 0x0e, 0x0a,
    0x0a, 0x42, 0x72, 0x6f, 0x6b, 0x65, 0x6e, 0x50, 0x69, 0x70, 0x65, 0x10, 0x0a, 0x12, 0x11, 0x0a,
    0x0d, 0x41, 0x6c, 0x72, 0x65, 0x61, 0x64, 0x79, 0x45, 0x78, 0x69, 0x73, 0x74, 0x73, 0x10, 0x0b,
    0x12, 0x0e, 0x0a, 0x0a, 0x57, 0x6f, 0x75, 0x6c, 0x64, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x10, 0x0c,
    0x12, 0x10, 0x0a, 0x0c, 0x49, 0x6e, 0x76, 0x61, 0x6c, 0x69, 0x64, 0x49, 0x6e, 0x70, 0x75, 0x74,
    0x10, 0x0d, 0x12, 0x0f, 0x0a, 0x0b, 0x49, 0x6e, 0x76, 0x61, 0x6c, 0x69, 0x64, 0x44, 0x61, 0x74,
    0x61, 0x10, 0x0e, 0x12, 0x0c, 0x0a, 0x08, 0x54, 0x69, 0x6d, 0x65, 0x64, 0x4f, 0x75, 0x74, 0x10,
    0x0f, 0x12, 0x0d, 0x0a, 0x09, 0x57, 0x72, 0x69, 0x74, 0x65, 0x5a, 0x65, 0x72, 0x6f, 0x10, 0x10,
    0x12, 0x09, 0x0a, 0x05, 0x4f, 0x74, 0x68, 0x65, 0x72, 0x10, 0x11, 0x12, 0x11, 0x0a, 0x0d, 0x55,
    0x6e, 0x65, 0x78, 0x70, 0x65, 0x63, 0x74, 0x65, 0x64, 0x45, 0x6f, 0x66, 0x10, 0x12, 0x4a, 0x84,
    0x09, 0x0a, 0x06, 0x12, 0x04, 0x00, 0x00, 0x1c, 0x01, 0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03,
    0x00, 0x08, 0x0f, 0x0a, 0x2b, 0x0a, 0x02, 0x04, 0x00, 0x12, 0x04, 0x03, 0x00, 0x1c, 0x01, 0x1a,
    0x1f, 0x20, 0x50, 0x61, 0x63, 0x6b, 0x65, 0x74, 0x20, 0x64, 0x65, 0x66, 0x69, 0x6e, 0x65, 0x73,
    0x20, 0x72, 0x65, 0x61, 0x64, 0x65, 0x72, 0x20, 0x70, 0x61, 0x63, 0x6b, 0x65, 0x74, 0x2e, 0x0a,
    0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01, 0x12, 0x03, 0x03, 0x08, 0x0e, 0x0a, 0x0c, 0x0a, 0x04,
    0x04, 0x00, 0x04, 0x00, 0x12, 0x04, 0x04, 0x02, 0x17, 0x03, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00,
    0x04, 0x00, 0x01, 0x12, 0x03, 0x04, 0x07, 0x0d, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00,
    0x02, 0x00, 0x12, 0x03, 0x05, 0x04, 0x0b, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x00, 0x01, 0x12, 0x03, 0x05, 0x04, 0x06, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x00, 0x02, 0x12, 0x03, 0x05, 0x09, 0x0a, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x01, 0x12, 0x03, 0x06, 0x04, 0x11, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x01,
    0x01, 0x12, 0x03, 0x06, 0x04, 0x0c, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x01,
    0x02, 0x12, 0x03, 0x06, 0x0f, 0x10, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x02,
    0x12, 0x03, 0x07, 0x04, 0x19, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x02, 0x01,
    0x12, 0x03, 0x07, 0x04, 0x14, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x02, 0x02,
    0x12, 0x03, 0x07, 0x17, 0x18, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x03, 0x12,
    0x03, 0x08, 0x04, 0x1a, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x03, 0x01, 0x12,
    0x03, 0x08, 0x04, 0x15, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x03, 0x02, 0x12,
    0x03, 0x08, 0x18, 0x19, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x04, 0x12, 0x03,
    0x09, 0x04, 0x18, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x04, 0x01, 0x12, 0x03,
    0x09, 0x04, 0x13, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x04, 0x02, 0x12, 0x03,
    0x09, 0x16, 0x17, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x05, 0x12, 0x03, 0x0a,
    0x04, 0x1a, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x05, 0x01, 0x12, 0x03, 0x0a,
    0x04, 0x15, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x05, 0x02, 0x12, 0x03, 0x0a,
    0x18, 0x19, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x06, 0x12, 0x03, 0x0b, 0x04,
    0x15, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x06, 0x01, 0x12, 0x03, 0x0b, 0x04,
    0x10, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x06, 0x02, 0x12, 0x03, 0x0b, 0x13,
    0x14, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x07, 0x12, 0x03, 0x0c, 0x04, 0x12,
    0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x07, 0x01, 0x12, 0x03, 0x0c, 0x04, 0x0d,
    0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x07, 0x02, 0x12, 0x03, 0x0c, 0x10, 0x11,
    0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x08, 0x12, 0x03, 0x0d, 0x04, 0x19, 0x0a,
    0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x08, 0x01, 0x12, 0x03, 0x0d, 0x04, 0x14, 0x0a,
    0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x08, 0x02, 0x12, 0x03, 0x0d, 0x17, 0x18, 0x0a,
    0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x09, 0x12, 0x03, 0x0e, 0x04, 0x14, 0x0a, 0x0e,
    0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x09, 0x01, 0x12, 0x03, 0x0e, 0x04, 0x0e, 0x0a, 0x0e,
    0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x09, 0x02, 0x12, 0x03, 0x0e, 0x11, 0x13, 0x0a, 0x0d,
    0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x0a, 0x12, 0x03, 0x0f, 0x04, 0x17, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x0a, 0x01, 0x12, 0x03, 0x0f, 0x04, 0x11, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x0a, 0x02, 0x12, 0x03, 0x0f, 0x14, 0x16, 0x0a, 0x0d, 0x0a,
    0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x0b, 0x12, 0x03, 0x10, 0x04, 0x14, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x04, 0x00, 0x02, 0x0b, 0x01, 0x12, 0x03, 0x10, 0x04, 0x0e, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x04, 0x00, 0x02, 0x0b, 0x02, 0x12, 0x03, 0x10, 0x11, 0x13, 0x0a, 0x0d, 0x0a, 0x06,
    0x04, 0x00, 0x04, 0x00, 0x02, 0x0c, 0x12, 0x03, 0x11, 0x04, 0x16, 0x0a, 0x0e, 0x0a, 0x07, 0x04,
    0x00, 0x04, 0x00, 0x02, 0x0c, 0x01, 0x12, 0x03, 0x11, 0x04, 0x10, 0x0a, 0x0e, 0x0a, 0x07, 0x04,
    0x00, 0x04, 0x00, 0x02, 0x0c, 0x02, 0x12, 0x03, 0x11, 0x13, 0x15, 0x0a, 0x0d, 0x0a, 0x06, 0x04,
    0x00, 0x04, 0x00, 0x02, 0x0d, 0x12, 0x03, 0x12, 0x04, 0x15, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00,
    0x04, 0x00, 0x02, 0x0d, 0x01, 0x12, 0x03, 0x12, 0x04, 0x0f, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00,
    0x04, 0x00, 0x02, 0x0d, 0x02, 0x12, 0x03, 0x12, 0x12, 0x14, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00,
    0x04, 0x00, 0x02, 0x0e, 0x12, 0x03, 0x13, 0x04, 0x12, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04,
    0x00, 0x02, 0x0e, 0x01, 0x12, 0x03, 0x13, 0x04, 0x0c, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04,
    0x00, 0x02, 0x0e, 0x02, 0x12, 0x03, 0x13, 0x0f, 0x11, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04,
    0x00, 0x02, 0x0f, 0x12, 0x03, 0x14, 0x04, 0x13, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00,
    0x02, 0x0f, 0x01, 0x12, 0x03, 0x14, 0x04, 0x0d, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00,
    0x02, 0x0f, 0x02, 0x12, 0x03, 0x14, 0x10, 0x12, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00,
    0x02, 0x10, 0x12, 0x03, 0x15, 0x04, 0x0f, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x10, 0x01, 0x12, 0x03, 0x15, 0x04, 0x09, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x10, 0x02, 0x12, 0x03, 0x15, 0x0c, 0x0e, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x11, 0x12, 0x03, 0x16, 0x04, 0x17, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x11,
    0x01, 0x12, 0x03, 0x16, 0x04, 0x11, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x11,
    0x02, 0x12, 0x03, 0x16, 0x14, 0x16, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03,
    0x19, 0x02, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x04, 0x12, 0x03, 0x19, 0x02,
    0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x06, 0x12, 0x03, 0x19, 0x0b, 0x11, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x19, 0x12, 0x18, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x19, 0x1b, 0x1c, 0x0a, 0x0b, 0x0a, 0x04, 0x04,
    0x00, 0x02, 0x01, 0x12, 0x03, 0x1a, 0x02, 0x1e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01,
    0x04, 0x12, 0x03, 0x1a, 0x02, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x05, 0x12,
    0x03, 0x1a, 0x0b, 0x11, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x1a,
    0x12, 0x19, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03, 0x1a, 0x1c, 0x1d,
    0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x02, 0x12, 0x03, 0x1b, 0x02, 0x1d, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x02, 0x04, 0x12, 0x03, 0x1b, 0x02, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x00, 0x02, 0x02, 0x05, 0x12, 0x03, 0x1b, 0x0b, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02,
    0x02, 0x01, 0x12, 0x03, 0x1b, 0x11, 0x18, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x03,
    0x12, 0x03, 0x1b, 0x1b, 0x1c,
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
