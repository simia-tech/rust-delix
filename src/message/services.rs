// This file is generated. Do not edit
// @generated

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;
use super::service::Service;

#[derive(Clone,Default)]
pub struct Services {
    // message fields
    services: ::protobuf::RepeatedField<Service>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl Services {
    pub fn new() -> Services {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Services {
        static mut instance: ::protobuf::lazy::Lazy<Services> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Services,
        };
        unsafe {
            instance.get(|| {
                Services {
                    services: ::protobuf::RepeatedField::new(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // repeated .message.Service services = 1;

    pub fn clear_services(&mut self) {
        self.services.clear();
    }

    // Param is passed by value, moved
    pub fn set_services(&mut self, v: ::protobuf::RepeatedField<Service>) {
        self.services = v;
    }

    // Mutable pointer to the field.
    pub fn mut_services<'a>(&'a mut self) -> &'a mut ::protobuf::RepeatedField<Service> {
        &mut self.services
    }

    // Take field
    pub fn take_services(&mut self) -> ::protobuf::RepeatedField<Service> {
        ::std::mem::replace(&mut self.services, ::protobuf::RepeatedField::new())
    }

    pub fn get_services<'a>(&'a self) -> &'a [Service] {
        &self.services
    }
}

impl ::protobuf::Message for Services {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    try!(::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.services));
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
        for value in self.services.iter() {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in self.services.iter() {
            try!(os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited));
            try!(os.write_raw_varint32(v.get_cached_size()));
            try!(v.write_to_with_cached_sizes(os));
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
        ::std::any::TypeId::of::<Services>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Services {
    fn new() -> Services {
        Services::new()
    }

    fn descriptor_static(_: ::std::option::Option<Services>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_repeated_message_accessor(
                    "services",
                    Services::get_services,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Services>(
                    "Services",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Services {
    fn clear(&mut self) {
        self.clear_services();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for Services {
    fn eq(&self, other: &Services) -> bool {
        self.services == other.services &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for Services {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x0e, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f,
    0x12, 0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x1a, 0x0d, 0x73, 0x65, 0x72, 0x76, 0x69,
    0x63, 0x65, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x22, 0x2e, 0x0a, 0x08, 0x53, 0x65, 0x72, 0x76,
    0x69, 0x63, 0x65, 0x73, 0x12, 0x22, 0x0a, 0x08, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73,
    0x18, 0x01, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x10, 0x2e, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65,
    0x2e, 0x53, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x4a, 0xc0, 0x01, 0x0a, 0x06, 0x12, 0x04, 0x00,
    0x00, 0x07, 0x01, 0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x00, 0x08, 0x0f, 0x0a, 0x09, 0x0a,
    0x02, 0x03, 0x00, 0x12, 0x03, 0x02, 0x07, 0x16, 0x0a, 0x50, 0x0a, 0x02, 0x04, 0x00, 0x12, 0x04,
    0x05, 0x00, 0x07, 0x01, 0x1a, 0x44, 0x20, 0x53, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73, 0x20,
    0x69, 0x73, 0x20, 0x74, 0x68, 0x65, 0x20, 0x70, 0x61, 0x63, 0x6b, 0x65, 0x74, 0x20, 0x74, 0x68,
    0x61, 0x74, 0x20, 0x69, 0x73, 0x20, 0x73, 0x65, 0x6e, 0x64, 0x20, 0x74, 0x6f, 0x20, 0x74, 0x65,
    0x6c, 0x6c, 0x20, 0x61, 0x20, 0x6e, 0x6f, 0x64, 0x65, 0x20, 0x61, 0x62, 0x6f, 0x75, 0x74, 0x20,
    0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73, 0x2e, 0x0a, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00,
    0x01, 0x12, 0x03, 0x05, 0x08, 0x10, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03,
    0x06, 0x02, 0x20, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x04, 0x12, 0x03, 0x06, 0x02,
    0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x06, 0x12, 0x03, 0x06, 0x0b, 0x12, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x06, 0x13, 0x1b, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x06, 0x1e, 0x1f,
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
