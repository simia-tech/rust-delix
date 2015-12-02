// This file is generated. Do not edit
// @generated

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Kind {
    IntroductionMessage = 1,
}

impl ::protobuf::ProtobufEnum for Kind {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Kind> {
        match value {
            1 => ::std::option::Option::Some(Kind::IntroductionMessage),
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
    0x0a, 0x0a, 0x6b, 0x69, 0x6e, 0x64, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x07, 0x6d, 0x65,
    0x73, 0x73, 0x61, 0x67, 0x65, 0x2a, 0x1f, 0x0a, 0x04, 0x4b, 0x69, 0x6e, 0x64, 0x12, 0x17, 0x0a,
    0x13, 0x49, 0x6e, 0x74, 0x72, 0x6f, 0x64, 0x75, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x4d, 0x65, 0x73,
    0x73, 0x61, 0x67, 0x65, 0x10, 0x01, 0x4a, 0x53, 0x0a, 0x06, 0x12, 0x04, 0x00, 0x00, 0x04, 0x01,
    0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x00, 0x08, 0x0f, 0x0a, 0x0a, 0x0a, 0x02, 0x05, 0x00,
    0x12, 0x04, 0x02, 0x00, 0x04, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x05, 0x00, 0x01, 0x12, 0x03, 0x02,
    0x05, 0x09, 0x0a, 0x0b, 0x0a, 0x04, 0x05, 0x00, 0x02, 0x00, 0x12, 0x03, 0x03, 0x02, 0x1a, 0x0a,
    0x0c, 0x0a, 0x05, 0x05, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x03, 0x02, 0x15, 0x0a, 0x0c, 0x0a,
    0x05, 0x05, 0x00, 0x02, 0x00, 0x02, 0x12, 0x03, 0x03, 0x18, 0x19,
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
