/*
 * Copyright (c) 2024 The Caffeine Project Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

pub enum AttributeInfo<'class> {
    ConstantValue {
        constantvalue_index: u16,
    },
    Code {
        max_stack: u16,
        max_locals: u16,
        code: &'class [u8],
        exception_table: Vec<ExceptionTableEntry>,
        attributes: Vec<Attribute<'class>>,
    },
    Deprecated,
    EnclosingMethod {
        class_index: u16,
        method_index: u16,
    },
    Exception {
        exception_index_table: &'class [u16],
    },
    InnerClasses {
        classes: Vec<InnerClass>,
    },
    LineNumberTable {
        line_number_table: Vec<LineNumber>,
    },
    LocalVariableTable {
        local_variable_table: Vec<LocalVariable>,
    },
    LocalVariableTypeTable {
        local_variable_type_table: Vec<LocalVariableType>,
    },
    Signature {
        signature_index: u16,
    },
    SourceDebugExtension {
        debug_extension: &'class [u8],
    },
    SourceFile {
        sourcefile_index: u16,
    },
    StackMapTable {
        entries: Vec<StackMapFrame>,
    },
    Synthetic,
}

pub enum ConstantPoolEntry<'class> {
    // Tag: 1
    Utf8 {
        bytes: &'class [u8],
    },
    // Tag: 3
    Integer {
        bytes: u32,
    },
    // Tag: 4
    Float {
        bytes: u32,
    },
    // Tag: 5
    Long {
        high_bytes: u32,
        low_bytes: u32,
    },
    // Tag: 6
    Double {
        high_bytes: u32,
        low_bytes: u32,
    },
    // Tag: 7
    Class {
        name_index: u16,
    },
    // Tag: 8
    String {
        string_index: u16,
    },
    // Tag: 9
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    // Tag: 10
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    // Tag: 11
    InstanceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    // Tag: 12
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    // Tag: 15
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    // Tag: 16
    MethodType {
        reference_index: u16,
    },
    // Tag: 17
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    // Tag: 18
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    // Tag: 19
    Module {
        name_index: u16,
    },
    // Tag: 20
    Package {
        name_index: u16,
    },
}

pub enum StackMapFrame {
    AppendFrame {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    },
    ChopFrame {
        offset_delta: u16,
    },
    FullFrame {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
        stack: Vec<VerificationTypeInfo>,
    },
    SameFrame,
    SameFrameExtended {
        offset_delta: u16,
    },
    SameLocals1StackItemFrame {
        stack: VerificationTypeInfo,
    },
    SameLocals1StackItemFrameExtended {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
}

pub enum VerificationTypeInfo {
    DoubleVariable,
    FloatVariable,
    IntegerVariable,
    LongVariable,
    NullVariable,
    ObjectVariable,
    TopVariable,
    UninitializedThisVariable,
    UninitializedVariable,
}

pub struct AccessFlags;

impl AccessFlags {
    pub const PUBLIC: u16 = 0x0001;
    pub const FINAL: u16 = 0x0010;
    pub const SUPER: u16 = 0x0020;
    pub const INTERFACE: u16 = 0x0200;
    pub const ABSTRACT: u16 = 0x0400;
    pub const SYNTHETIC: u16 = 0x1000;
    pub const ANNOTATION: u16 = 0x2000;
    pub const ENUM: u16 = 0x4000;
    pub const MODULE: u16 = 0x8000;
}

pub struct Attribute<'class> {
    pub attribute_name_index: u16,
    pub info: AttributeInfo<'class>,
}

pub struct Classfile<'a> {
    pub version: Version,
    pub constant_pool: Vec<ConstantPoolEntry<'a>>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
}

pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

pub struct InnerClass {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: u16,
}

pub struct LineNumber {
    pub start_pc: u16,
    pub line_number: u16,
}

pub struct LocalVariable {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

pub struct LocalVariableType {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

pub struct Version {
    pub minor: u16,
    pub major: u16,
}
