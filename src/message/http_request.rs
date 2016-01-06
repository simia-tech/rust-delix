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
pub struct HttpRequest {
    // message fields
    method: ::std::option::Option<HttpRequest_Method>,
    path: ::protobuf::SingularField<::std::string::String>,
    version: ::std::option::Option<HttpRequest_Version>,
    headers: ::protobuf::RepeatedField<HttpRequest_Header>,
    body: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl HttpRequest {
    pub fn new() -> HttpRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static HttpRequest {
        static mut instance: ::protobuf::lazy::Lazy<HttpRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HttpRequest,
        };
        unsafe {
            instance.get(|| {
                HttpRequest {
                    method: ::std::option::Option::None,
                    path: ::protobuf::SingularField::none(),
                    version: ::std::option::Option::None,
                    headers: ::protobuf::RepeatedField::new(),
                    body: ::protobuf::SingularField::none(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // optional .message.HttpRequest.Method method = 1;

    pub fn clear_method(&mut self) {
        self.method = ::std::option::Option::None;
    }

    pub fn has_method(&self) -> bool {
        self.method.is_some()
    }

    // Param is passed by value, moved
    pub fn set_method(&mut self, v: HttpRequest_Method) {
        self.method = ::std::option::Option::Some(v);
    }

    pub fn get_method<'a>(&self) -> HttpRequest_Method {
        self.method.unwrap_or(HttpRequest_Method::OPTIONS)
    }

    // optional string path = 2;

    pub fn clear_path(&mut self) {
        self.path.clear();
    }

    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }

    // Param is passed by value, moved
    pub fn set_path(&mut self, v: ::std::string::String) {
        self.path = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_path<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.path.is_none() {
            self.path.set_default();
        };
        self.path.as_mut().unwrap()
    }

    // Take field
    pub fn take_path(&mut self) -> ::std::string::String {
        self.path.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_path<'a>(&'a self) -> &'a str {
        match self.path.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // optional .message.HttpRequest.Version version = 3;

    pub fn clear_version(&mut self) {
        self.version = ::std::option::Option::None;
    }

    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }

    // Param is passed by value, moved
    pub fn set_version(&mut self, v: HttpRequest_Version) {
        self.version = ::std::option::Option::Some(v);
    }

    pub fn get_version<'a>(&self) -> HttpRequest_Version {
        self.version.unwrap_or(HttpRequest_Version::V09)
    }

    // repeated .message.HttpRequest.Header headers = 4;

    pub fn clear_headers(&mut self) {
        self.headers.clear();
    }

    // Param is passed by value, moved
    pub fn set_headers(&mut self, v: ::protobuf::RepeatedField<HttpRequest_Header>) {
        self.headers = v;
    }

    // Mutable pointer to the field.
    pub fn mut_headers<'a>(&'a mut self) -> &'a mut ::protobuf::RepeatedField<HttpRequest_Header> {
        &mut self.headers
    }

    // Take field
    pub fn take_headers(&mut self) -> ::protobuf::RepeatedField<HttpRequest_Header> {
        ::std::mem::replace(&mut self.headers, ::protobuf::RepeatedField::new())
    }

    pub fn get_headers<'a>(&'a self) -> &'a [HttpRequest_Header] {
        &self.headers
    }

    // optional bytes body = 5;

    pub fn clear_body(&mut self) {
        self.body.clear();
    }

    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }

    // Param is passed by value, moved
    pub fn set_body(&mut self, v: ::std::vec::Vec<u8>) {
        self.body = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_body<'a>(&'a mut self) -> &'a mut ::std::vec::Vec<u8> {
        if self.body.is_none() {
            self.body.set_default();
        };
        self.body.as_mut().unwrap()
    }

    // Take field
    pub fn take_body(&mut self) -> ::std::vec::Vec<u8> {
        self.body.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    pub fn get_body<'a>(&'a self) -> &'a [u8] {
        match self.body.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }
}

impl ::protobuf::Message for HttpRequest {
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
                    self.method = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.path.set_default();
                    try!(is.read_string_into(tmp))
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_enum());
                    self.version = ::std::option::Option::Some(tmp);
                },
                4 => {
                    try!(::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.headers));
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.body.set_default();
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
        for value in self.method.iter() {
            my_size += ::protobuf::rt::enum_size(1, *value);
        };
        for value in self.path.iter() {
            my_size += ::protobuf::rt::string_size(2, &value);
        };
        for value in self.version.iter() {
            my_size += ::protobuf::rt::enum_size(3, *value);
        };
        for value in self.headers.iter() {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        for value in self.body.iter() {
            my_size += ::protobuf::rt::bytes_size(5, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.method {
            try!(os.write_enum(1, v as i32));
        };
        if let Some(v) = self.path.as_ref() {
            try!(os.write_string(2, &v));
        };
        if let Some(v) = self.version {
            try!(os.write_enum(3, v as i32));
        };
        for v in self.headers.iter() {
            try!(os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited));
            try!(os.write_raw_varint32(v.get_cached_size()));
            try!(v.write_to_with_cached_sizes(os));
        };
        if let Some(v) = self.body.as_ref() {
            try!(os.write_bytes(5, &v));
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
        ::std::any::TypeId::of::<HttpRequest>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for HttpRequest {
    fn new() -> HttpRequest {
        HttpRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<HttpRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_enum_accessor(
                    "method",
                    HttpRequest::has_method,
                    HttpRequest::get_method,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "path",
                    HttpRequest::has_path,
                    HttpRequest::get_path,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_enum_accessor(
                    "version",
                    HttpRequest::has_version,
                    HttpRequest::get_version,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_message_accessor(
                    "headers",
                    HttpRequest::get_headers,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_bytes_accessor(
                    "body",
                    HttpRequest::has_body,
                    HttpRequest::get_body,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HttpRequest>(
                    "HttpRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for HttpRequest {
    fn clear(&mut self) {
        self.clear_method();
        self.clear_path();
        self.clear_version();
        self.clear_headers();
        self.clear_body();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for HttpRequest {
    fn eq(&self, other: &HttpRequest) -> bool {
        self.method == other.method &&
        self.path == other.path &&
        self.version == other.version &&
        self.headers == other.headers &&
        self.body == other.body &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

#[derive(Clone,Default)]
pub struct HttpRequest_Header {
    // message fields
    name: ::protobuf::SingularField<::std::string::String>,
    value: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl HttpRequest_Header {
    pub fn new() -> HttpRequest_Header {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static HttpRequest_Header {
        static mut instance: ::protobuf::lazy::Lazy<HttpRequest_Header> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HttpRequest_Header,
        };
        unsafe {
            instance.get(|| {
                HttpRequest_Header {
                    name: ::protobuf::SingularField::none(),
                    value: ::protobuf::SingularField::none(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // optional string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.name.is_none() {
            self.name.set_default();
        };
        self.name.as_mut().unwrap()
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        self.name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_name<'a>(&'a self) -> &'a str {
        match self.name.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // optional string value = 2;

    pub fn clear_value(&mut self) {
        self.value.clear();
    }

    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }

    // Param is passed by value, moved
    pub fn set_value(&mut self, v: ::std::string::String) {
        self.value = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_value<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.value.is_none() {
            self.value.set_default();
        };
        self.value.as_mut().unwrap()
    }

    // Take field
    pub fn take_value(&mut self) -> ::std::string::String {
        self.value.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_value<'a>(&'a self) -> &'a str {
        match self.value.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }
}

impl ::protobuf::Message for HttpRequest_Header {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.name.set_default();
                    try!(is.read_string_into(tmp))
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.value.set_default();
                    try!(is.read_string_into(tmp))
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
        for value in self.name.iter() {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        for value in self.value.iter() {
            my_size += ::protobuf::rt::string_size(2, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.name.as_ref() {
            try!(os.write_string(1, &v));
        };
        if let Some(v) = self.value.as_ref() {
            try!(os.write_string(2, &v));
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
        ::std::any::TypeId::of::<HttpRequest_Header>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for HttpRequest_Header {
    fn new() -> HttpRequest_Header {
        HttpRequest_Header::new()
    }

    fn descriptor_static(_: ::std::option::Option<HttpRequest_Header>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "name",
                    HttpRequest_Header::has_name,
                    HttpRequest_Header::get_name,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "value",
                    HttpRequest_Header::has_value,
                    HttpRequest_Header::get_value,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HttpRequest_Header>(
                    "HttpRequest_Header",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for HttpRequest_Header {
    fn clear(&mut self) {
        self.clear_name();
        self.clear_value();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for HttpRequest_Header {
    fn eq(&self, other: &HttpRequest_Header) -> bool {
        self.name == other.name &&
        self.value == other.value &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for HttpRequest_Header {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum HttpRequest_Method {
    OPTIONS = 1,
    GET = 2,
    POST = 3,
    PUT = 4,
    DELETE = 5,
    HEAD = 6,
    TRACE = 7,
    CONNECT = 8,
    PATCH = 9,
}

impl ::protobuf::ProtobufEnum for HttpRequest_Method {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<HttpRequest_Method> {
        match value {
            1 => ::std::option::Option::Some(HttpRequest_Method::OPTIONS),
            2 => ::std::option::Option::Some(HttpRequest_Method::GET),
            3 => ::std::option::Option::Some(HttpRequest_Method::POST),
            4 => ::std::option::Option::Some(HttpRequest_Method::PUT),
            5 => ::std::option::Option::Some(HttpRequest_Method::DELETE),
            6 => ::std::option::Option::Some(HttpRequest_Method::HEAD),
            7 => ::std::option::Option::Some(HttpRequest_Method::TRACE),
            8 => ::std::option::Option::Some(HttpRequest_Method::CONNECT),
            9 => ::std::option::Option::Some(HttpRequest_Method::PATCH),
            _ => ::std::option::Option::None
        }
    }

    fn enum_descriptor_static(_: Option<HttpRequest_Method>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("HttpRequest_Method", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for HttpRequest_Method {
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum HttpRequest_Version {
    V09 = 1,
    V10 = 2,
    V11 = 3,
    V20 = 4,
}

impl ::protobuf::ProtobufEnum for HttpRequest_Version {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<HttpRequest_Version> {
        match value {
            1 => ::std::option::Option::Some(HttpRequest_Version::V09),
            2 => ::std::option::Option::Some(HttpRequest_Version::V10),
            3 => ::std::option::Option::Some(HttpRequest_Version::V11),
            4 => ::std::option::Option::Some(HttpRequest_Version::V20),
            _ => ::std::option::Option::None
        }
    }

    fn enum_descriptor_static(_: Option<HttpRequest_Version>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("HttpRequest_Version", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for HttpRequest_Version {
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x12, 0x68, 0x74, 0x74, 0x70, 0x5f, 0x72, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x2e, 0x70,
    0x72, 0x6f, 0x74, 0x6f, 0x12, 0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x22, 0xf5, 0x02,
    0x0a, 0x0b, 0x48, 0x74, 0x74, 0x70, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x12, 0x2b, 0x0a,
    0x06, 0x6d, 0x65, 0x74, 0x68, 0x6f, 0x64, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x1b, 0x2e,
    0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2e, 0x48, 0x74, 0x74, 0x70, 0x52, 0x65, 0x71, 0x75,
    0x65, 0x73, 0x74, 0x2e, 0x4d, 0x65, 0x74, 0x68, 0x6f, 0x64, 0x12, 0x0c, 0x0a, 0x04, 0x70, 0x61,
    0x74, 0x68, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09, 0x12, 0x2d, 0x0a, 0x07, 0x76, 0x65, 0x72, 0x73,
    0x69, 0x6f, 0x6e, 0x18, 0x03, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x1c, 0x2e, 0x6d, 0x65, 0x73, 0x73,
    0x61, 0x67, 0x65, 0x2e, 0x48, 0x74, 0x74, 0x70, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x2e,
    0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x12, 0x2c, 0x0a, 0x07, 0x68, 0x65, 0x61, 0x64, 0x65,
    0x72, 0x73, 0x18, 0x04, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x1b, 0x2e, 0x6d, 0x65, 0x73, 0x73, 0x61,
    0x67, 0x65, 0x2e, 0x48, 0x74, 0x74, 0x70, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x2e, 0x48,
    0x65, 0x61, 0x64, 0x65, 0x72, 0x12, 0x0c, 0x0a, 0x04, 0x62, 0x6f, 0x64, 0x79, 0x18, 0x05, 0x20,
    0x01, 0x28, 0x0c, 0x1a, 0x25, 0x0a, 0x06, 0x48, 0x65, 0x61, 0x64, 0x65, 0x72, 0x12, 0x0c, 0x0a,
    0x04, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x12, 0x0d, 0x0a, 0x05, 0x76,
    0x61, 0x6c, 0x75, 0x65, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09, 0x22, 0x6a, 0x0a, 0x06, 0x4d, 0x65,
    0x74, 0x68, 0x6f, 0x64, 0x12, 0x0b, 0x0a, 0x07, 0x4f, 0x50, 0x54, 0x49, 0x4f, 0x4e, 0x53, 0x10,
    0x01, 0x12, 0x07, 0x0a, 0x03, 0x47, 0x45, 0x54, 0x10, 0x02, 0x12, 0x08, 0x0a, 0x04, 0x50, 0x4f,
    0x53, 0x54, 0x10, 0x03, 0x12, 0x07, 0x0a, 0x03, 0x50, 0x55, 0x54, 0x10, 0x04, 0x12, 0x0a, 0x0a,
    0x06, 0x44, 0x45, 0x4c, 0x45, 0x54, 0x45, 0x10, 0x05, 0x12, 0x08, 0x0a, 0x04, 0x48, 0x45, 0x41,
    0x44, 0x10, 0x06, 0x12, 0x09, 0x0a, 0x05, 0x54, 0x52, 0x41, 0x43, 0x45, 0x10, 0x07, 0x12, 0x0b,
    0x0a, 0x07, 0x43, 0x4f, 0x4e, 0x4e, 0x45, 0x43, 0x54, 0x10, 0x08, 0x12, 0x09, 0x0a, 0x05, 0x50,
    0x41, 0x54, 0x43, 0x48, 0x10, 0x09, 0x22, 0x2d, 0x0a, 0x07, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f,
    0x6e, 0x12, 0x07, 0x0a, 0x03, 0x56, 0x30, 0x39, 0x10, 0x01, 0x12, 0x07, 0x0a, 0x03, 0x56, 0x31,
    0x30, 0x10, 0x02, 0x12, 0x07, 0x0a, 0x03, 0x56, 0x31, 0x31, 0x10, 0x03, 0x12, 0x07, 0x0a, 0x03,
    0x56, 0x32, 0x30, 0x10, 0x04, 0x4a, 0x88, 0x0a, 0x0a, 0x06, 0x12, 0x04, 0x00, 0x00, 0x21, 0x01,
    0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x00, 0x08, 0x0f, 0x0a, 0x3a, 0x0a, 0x02, 0x04, 0x00,
    0x12, 0x04, 0x03, 0x00, 0x21, 0x01, 0x1a, 0x2e, 0x20, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74,
    0x20, 0x64, 0x65, 0x66, 0x69, 0x6e, 0x65, 0x73, 0x20, 0x61, 0x20, 0x68, 0x74, 0x74, 0x70, 0x20,
    0x72, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x20, 0x74, 0x6f, 0x20, 0x61, 0x20, 0x73, 0x65, 0x72,
    0x76, 0x69, 0x63, 0x65, 0x2e, 0x0a, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01, 0x12, 0x03, 0x03,
    0x08, 0x13, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x00, 0x04, 0x00, 0x12, 0x04, 0x04, 0x02, 0x0e, 0x03,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x04, 0x00, 0x01, 0x12, 0x03, 0x04, 0x07, 0x0d, 0x0a, 0x0d,
    0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x05, 0x04, 0x10, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x05, 0x04, 0x0b, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x00, 0x02, 0x12, 0x03, 0x05, 0x0e, 0x0f, 0x0a, 0x0d, 0x0a,
    0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x06, 0x04, 0x0c, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x06, 0x04, 0x07, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x04, 0x00, 0x02, 0x01, 0x02, 0x12, 0x03, 0x06, 0x0a, 0x0b, 0x0a, 0x0d, 0x0a, 0x06,
    0x04, 0x00, 0x04, 0x00, 0x02, 0x02, 0x12, 0x03, 0x07, 0x04, 0x0d, 0x0a, 0x0e, 0x0a, 0x07, 0x04,
    0x00, 0x04, 0x00, 0x02, 0x02, 0x01, 0x12, 0x03, 0x07, 0x04, 0x08, 0x0a, 0x0e, 0x0a, 0x07, 0x04,
    0x00, 0x04, 0x00, 0x02, 0x02, 0x02, 0x12, 0x03, 0x07, 0x0b, 0x0c, 0x0a, 0x0d, 0x0a, 0x06, 0x04,
    0x00, 0x04, 0x00, 0x02, 0x03, 0x12, 0x03, 0x08, 0x04, 0x0c, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00,
    0x04, 0x00, 0x02, 0x03, 0x01, 0x12, 0x03, 0x08, 0x04, 0x07, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00,
    0x04, 0x00, 0x02, 0x03, 0x02, 0x12, 0x03, 0x08, 0x0a, 0x0b, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00,
    0x04, 0x00, 0x02, 0x04, 0x12, 0x03, 0x09, 0x04, 0x0f, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04,
    0x00, 0x02, 0x04, 0x01, 0x12, 0x03, 0x09, 0x04, 0x0a, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04,
    0x00, 0x02, 0x04, 0x02, 0x12, 0x03, 0x09, 0x0d, 0x0e, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04,
    0x00, 0x02, 0x05, 0x12, 0x03, 0x0a, 0x04, 0x0d, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00,
    0x02, 0x05, 0x01, 0x12, 0x03, 0x0a, 0x04, 0x08, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00,
    0x02, 0x05, 0x02, 0x12, 0x03, 0x0a, 0x0b, 0x0c, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00,
    0x02, 0x06, 0x12, 0x03, 0x0b, 0x04, 0x0e, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x06, 0x01, 0x12, 0x03, 0x0b, 0x04, 0x09, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x06, 0x02, 0x12, 0x03, 0x0b, 0x0c, 0x0d, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02,
    0x07, 0x12, 0x03, 0x0c, 0x04, 0x10, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x07,
    0x01, 0x12, 0x03, 0x0c, 0x04, 0x0b, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x07,
    0x02, 0x12, 0x03, 0x0c, 0x0e, 0x0f, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x08,
    0x12, 0x03, 0x0d, 0x04, 0x0e, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x08, 0x01,
    0x12, 0x03, 0x0d, 0x04, 0x09, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x08, 0x02,
    0x12, 0x03, 0x0d, 0x0c, 0x0d, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x00, 0x04, 0x01, 0x12, 0x04, 0x10,
    0x02, 0x15, 0x03, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x04, 0x01, 0x01, 0x12, 0x03, 0x10, 0x07,
    0x0e, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x01, 0x02, 0x00, 0x12, 0x03, 0x11, 0x04, 0x0c,
    0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x01, 0x02, 0x00, 0x01, 0x12, 0x03, 0x11, 0x04, 0x07,
    0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x01, 0x02, 0x00, 0x02, 0x12, 0x03, 0x11, 0x0a, 0x0b,
    0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x01, 0x02, 0x01, 0x12, 0x03, 0x12, 0x04, 0x0c, 0x0a,
    0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x01, 0x02, 0x01, 0x01, 0x12, 0x03, 0x12, 0x04, 0x07, 0x0a,
    0x0e, 0x0a, 0x07, 0x04, 0x00, 0x04, 0x01, 0x02, 0x01, 0x02, 0x12, 0x03, 0x12, 0x0a, 0x0b, 0x0a,
    0x0d, 0x0a, 0x06, 0x04, 0x00, 0x04, 0x01, 0x02, 0x02, 0x12, 0x03, 0x13, 0x04, 0x0c, 0x0a, 0x0e,
    0x0a, 0x07, 0x04, 0x00, 0x04, 0x01, 0x02, 0x02, 0x01, 0x12, 0x03, 0x13, 0x04, 0x07, 0x0a, 0x0e,
    0x0a, 0x07, 0x04, 0x00, 0x04, 0x01, 0x02, 0x02, 0x02, 0x12, 0x03, 0x13, 0x0a, 0x0b, 0x0a, 0x0d,
    0x0a, 0x06, 0x04, 0x00, 0x04, 0x01, 0x02, 0x03, 0x12, 0x03, 0x14, 0x04, 0x0c, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x00, 0x04, 0x01, 0x02, 0x03, 0x01, 0x12, 0x03, 0x14, 0x04, 0x07, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x00, 0x04, 0x01, 0x02, 0x03, 0x02, 0x12, 0x03, 0x14, 0x0a, 0x0b, 0x0a, 0x0c, 0x0a,
    0x04, 0x04, 0x00, 0x03, 0x00, 0x12, 0x04, 0x17, 0x02, 0x1a, 0x03, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x00, 0x03, 0x00, 0x01, 0x12, 0x03, 0x17, 0x0a, 0x10, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x03,
    0x00, 0x02, 0x00, 0x12, 0x03, 0x18, 0x04, 0x1d, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x00,
    0x02, 0x00, 0x04, 0x12, 0x03, 0x18, 0x04, 0x0c, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x00,
    0x02, 0x00, 0x05, 0x12, 0x03, 0x18, 0x0d, 0x13, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x00,
    0x02, 0x00, 0x01, 0x12, 0x03, 0x18, 0x14, 0x18, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x00,
    0x02, 0x00, 0x03, 0x12, 0x03, 0x18, 0x1b, 0x1c, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x03, 0x00,
    0x02, 0x01, 0x12, 0x03, 0x19, 0x04, 0x1e, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x00, 0x02,
    0x01, 0x04, 0x12, 0x03, 0x19, 0x04, 0x0c, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x00, 0x02,
    0x01, 0x05, 0x12, 0x03, 0x19, 0x0d, 0x13, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x00, 0x02,
    0x01, 0x01, 0x12, 0x03, 0x19, 0x14, 0x19, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x00, 0x02,
    0x01, 0x03, 0x12, 0x03, 0x19, 0x1c, 0x1d, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x00, 0x12,
    0x03, 0x1c, 0x02, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x04, 0x12, 0x03, 0x1c,
    0x02, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x06, 0x12, 0x03, 0x1c, 0x0b, 0x11,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x1c, 0x12, 0x18, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x1c, 0x1b, 0x1c, 0x0a, 0x0b, 0x0a, 0x04,
    0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x1d, 0x02, 0x1b, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02,
    0x01, 0x04, 0x12, 0x03, 0x1d, 0x02, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x05,
    0x12, 0x03, 0x1d, 0x0b, 0x11, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03,
    0x1d, 0x12, 0x16, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03, 0x1d, 0x19,
    0x1a, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x02, 0x12, 0x03, 0x1e, 0x02, 0x1f, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x04, 0x12, 0x03, 0x1e, 0x02, 0x0a, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x00, 0x02, 0x02, 0x06, 0x12, 0x03, 0x1e, 0x0b, 0x12, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00,
    0x02, 0x02, 0x01, 0x12, 0x03, 0x1e, 0x13, 0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02,
    0x03, 0x12, 0x03, 0x1e, 0x1d, 0x1e, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x03, 0x12, 0x03,
    0x1f, 0x02, 0x1e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x04, 0x12, 0x03, 0x1f, 0x02,
    0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x06, 0x12, 0x03, 0x1f, 0x0b, 0x11, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x01, 0x12, 0x03, 0x1f, 0x12, 0x19, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x03, 0x03, 0x12, 0x03, 0x1f, 0x1c, 0x1d, 0x0a, 0x0b, 0x0a, 0x04, 0x04,
    0x00, 0x02, 0x04, 0x12, 0x03, 0x20, 0x02, 0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x04,
    0x04, 0x12, 0x03, 0x20, 0x02, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x04, 0x05, 0x12,
    0x03, 0x20, 0x0b, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x04, 0x01, 0x12, 0x03, 0x20,
    0x11, 0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x04, 0x03, 0x12, 0x03, 0x20, 0x18, 0x19,
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
