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
    AnnotationDefault {
        default_value: ElementValue,
    },
    BootstrapMethods {
        bootstrap_methods: Vec<BootstrapMethod<'class>>,
    },
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
    MethodParameters {
        parameters: Vec<MethodParameter>,
    },
    Module {
        module_name_index: u16,
        module_flags: u16,
        module_version_index: u16,

        requires: Vec<ModuleRequire>,
        exports: Vec<ModuleExport<'class>>,
        opens: Vec<ModuleOpens<'class>>,
        uses: &'class [u16],
        provides: Vec<ModuleProvides<'class>>,
    },
    ModuleMainClass {
        main_class_index: u16,
    },
    ModulePackages {
        package_index: &'class [u16],
    },
    NestHost {
        host_class_index: u16,
    },
    NestMembers {
        classes: &'class [u16],
    },
    PermittedSubclasses {
        classes: &'class [u16],
    },
    Record {
        components: Vec<RecordComponent<'class>>,
    },
    RuntimeInvisibleAnnotations {
        annotations: Vec<Annotation>,
    },
    RuntimeInvisibleParameterAnnotations {
        parameter_annotations: Vec<Annotation>,
    },
    RuntimeInvisibleTypeAnnotations {
        type_annotations: Vec<TypeAnnotation>,
    },
    RuntimeVisibleAnnotations {
        annotations: Vec<Annotation>,
    },
    RuntimeVisibleParameterAnnotations {
        parameter_annotations: Vec<Annotation>,
    },
    RuntimeVisibleTypeAnnotations {
        type_annotations: Vec<TypeAnnotation>,
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

pub enum ElementValue {
    Annotation(Annotation),
    ClassInfo(u16),
    ConstValue(u16),
    EnumConst {
        type_name_index: u16,
        const_name_index: u16,
    },
    Array {
        values: Vec<ElementValue>,
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

pub enum TargetInfo {
    Catch(u16),
    Empty,
    FormalParameter(u8),
    LocalVar {
        table: Vec<LocalVar>,
    },
    Offset(u16),
    Supertype(u16),
    Throws(u16),
    TypeArgument {
        offset: u16,
        type_argument_index: u16,
    },
    TypeParameter(u8),
    TypeParameterBound {
        type_parameter_index: u8,
        bound_index: u8,
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

pub struct Annotation {
    pub type_index: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}

pub struct BootstrapMethod<'a> {
    pub bootstrap_method_ref: u16,
    pub bootstrap_arguments: &'a [u16],
}

pub struct Classfile<'a> {
    pub version: Version,
    pub constant_pool: Vec<ConstantPoolEntry<'a>>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field<'a>>,
    pub methods: Vec<Method<'a>>,
}

pub struct ElementValuePair {
    pub element_name_index: u16,
    pub value: ElementValue,
}

pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

pub struct Field<'a> {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute<'a>>,
}

pub struct FieldAccessFlags;

impl FieldAccessFlags {
    pub const PUBLIC: u16 = 0x0001;
    pub const PRIVATE: u16 = 0x0002;
    pub const PROTECTED: u16 = 0x0004;
    pub const STATIC: u16 = 0x0008;
    pub const FINAL: u16 = 0x0010;
    pub const VOLATILE: u16 = 0x0040;
    pub const TRANSIENT: u16 = 0x0080;
    pub const SYNTHETIC: u16 = 0x1000;
    pub const ENUM: u16 = 0x4000;
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

pub struct LocalVar {
    pub start_pc: u16,
    pub length: u16,
    pub index: u16,
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

pub struct Method<'a> {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute<'a>>,
}

pub struct MethodParameter {
    pub name_index: u16,
    pub access_flags: u16,
}

pub struct ModuleExport<'a> {
    pub exports_index: u16,
    pub exports_flags: u16,
    pub exports_to_indices: &'a [u16],
}

pub struct ModuleExportFlags;

impl ModuleExportFlags {
    pub const SYNTHETIC: u16 = 0x1000;
    pub const MANDATED: u16 = 0x8000;
}

pub struct ModuleFlags;

impl ModuleFlags {
    pub const OPEN: u16 = 0x0020;
    pub const SYNTHETIC: u16 = 0x1000;
    pub const MANDATED: u16 = 0x8000;
}

pub struct ModuleOpens<'a> {
    pub opens_index: u16,
    pub opens_flags: u16,
    pub opens_to_indices: &'a [u16],
}

pub struct ModuleOpensFlags;

impl ModuleOpensFlags {
    pub const SYNTHETIC: u16 = 0x1000;
    pub const MANDATED: u16 = 0x8000;
}

pub struct ModuleProvides<'a> {
    pub provides_index: u16,
    pub provides_with_indices: &'a [u16],
}

pub struct ModuleRequire {
    pub requires_index: u16,
    pub requires_flags: u16,
    pub requires_version_index: u16,
}

pub struct ModuleRequireFlags;

impl ModuleRequireFlags {
    pub const TRANSITIVE: u16 = 0x0020;
    pub const STATIC_PHASE: u16 = 0x0040;
    pub const SYNTHETIC: u16 = 0x1000;
    pub const MANDATED: u16 = 0x8000;
}

pub struct RecordComponent<'a> {
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute<'a>>,
}

pub struct TypeAnnotation {
    pub target_type: u8,
    pub target_info: TargetInfo,
    pub target_path: TypePath,
    pub type_index: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}

pub struct TypePath {
    pub path: Vec<TypePathSegment>,
}

pub struct TypePathSegment {
    pub type_path_kind: u8,
    pub type_argument_index: u8,
}

pub struct Version {
    pub minor: u16,
    pub major: u16,
}
