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

use mutf8::mutf8_to_utf8;
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::error::Error;
use nom::error::ErrorKind;
use nom::multi::length_count;
use nom::number::complete::be_u16;
use nom::number::complete::be_u32;
use nom::number::complete::be_u8;
use nom::Err;
use nom::IResult;

use crate::cowext::CowExt;
use crate::spec::Annotation;
use crate::spec::Attribute;
use crate::spec::AttributeInfo;
use crate::spec::BootstrapMethod;
use crate::spec::Classfile;
use crate::spec::ConstantPoolEntry;
use crate::spec::ElementValue;
use crate::spec::ElementValuePair;
use crate::spec::ExceptionTableEntry;
use crate::spec::Field;
use crate::spec::InnerClass;
use crate::spec::LineNumber;
use crate::spec::LocalVar;
use crate::spec::LocalVariable;
use crate::spec::LocalVariableType;
use crate::spec::Method;
use crate::spec::MethodParameter;
use crate::spec::ModuleExports;
use crate::spec::ModuleOpens;
use crate::spec::ModuleProvides;
use crate::spec::ModuleRequires;
use crate::spec::RecordComponent;
use crate::spec::TargetInfo;
use crate::spec::TypeAnnotation;
use crate::spec::TypePath;
use crate::spec::TypePathSegment;
use crate::spec::Version;

pub fn classfile_from_bytes(bytes: &[u8]) -> IResult<&[u8], Classfile> {
    // make sure the magic bytes are there, to indicate a valid Java classfile
    let (input_1, _) = tag([0xCA, 0xFE, 0xBA, 0xBE])(bytes)?;

    // parse classfile version
    let (input_2, version) = classfile_version_from_bytes(input_1)?;

    // parse constant pool length and constant pool
    let (input_3, constant_pool) = length_count(be_u16, constant_pool_entry_from_bytes)(input_2)?;

    // parse access flags
    let (input_4, access_flags) = be_u16(input_3)?;

    // parse this class
    let (input_5, this_class) = be_u16(input_4)?;

    // parse super class
    let (input_6, super_class) = be_u16(input_5)?;

    // parse interfaces
    let (input_7, interfaces) = length_count(be_u16, be_u16)(input_6)?;

    // parse fields
    let (input_8, fields) = length_count(be_u16, |bytes| {
        field_from_bytes(bytes, constant_pool.as_slice())
    })(input_7)?;

    // parse methods
    let (input_9, methods) = length_count(be_u16, |bytes| {
        method_from_bytes(bytes, constant_pool.as_slice())
    })(input_8)?;

    // parse attributes
    let (input_10, attributes) = length_count(be_u16, |bytes| {
        attribute_from_bytes(bytes, constant_pool.as_slice())
    })(input_9)?;

    Ok((
        input_10,
        Classfile {
            version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        },
    ))
}

fn annotation_from_bytes(bytes: &[u8]) -> IResult<&[u8], Annotation> {
    let (input_1, type_index) = be_u16(bytes)?;
    let (input_2, element_value_pairs) =
        length_count(be_u16, element_value_pair_from_bytes)(input_1)?;

    Ok((
        input_2,
        Annotation {
            type_index,
            element_value_pairs,
        },
    ))
}

fn attribute_from_bytes<'a>(
    bytes: &'a [u8],
    constant_pool: &[ConstantPoolEntry<'a>],
) -> IResult<&'a [u8], Attribute<'a>> {
    let (input_1, attribute_name_index) = be_u16(bytes)?;
    let ConstantPoolEntry::Utf8 { bytes } = constant_pool[attribute_name_index as usize - 1] else {
        return Err(Err::Failure(Error::new(bytes, ErrorKind::IsNot)));
    };

    let (input_2, length) = be_u32(input_1)?;

    let Ok(utf8) = mutf8_to_utf8(bytes) else {
        return Err(Err::Failure(Error::new(bytes, ErrorKind::Verify)));
    };
    let (input_3, info) = unsafe {
        // SAFETY: the UTF-8 conversion above would have been failed if the MUTF-8 from Java cannot be converted
        // into conventional UTF-8 and returned an error; it is guaranteed that at this point the slice contains
        // bytes of valid UTF-8.
        match utf8.to_str_lossy().as_ref() {
            "AnnotationDefault" => attribute_annotation_default_from_bytes(input_2)?,
            "BootstrapMethods" => attribute_bootstrap_methods_from_bytes(input_2)?,
            "Code" => attribute_code_from_bytes(input_2, constant_pool)?,
            "ConstantValue" => attribute_constant_value_from_bytes(input_2)?,
            "Deprecated" => (input_2, AttributeInfo::Deprecated),
            "EnclosingMethod" => attribute_enclosing_method_from_bytes(input_2)?,
            "Exceptions" => attribute_exceptions_from_bytes(input_2)?,
            "InnerClasses" => attribute_inner_classes_from_bytes(input_2)?,
            "LineNumberTable" => attribute_line_number_table_from_bytes(input_2)?,
            "LocalVariableTable" => attribute_local_variable_table_from_bytes(input_2)?,
            "LocalVariableTypeTable" => attribute_local_variable_type_table_from_bytes(input_2)?,
            "MethodParameters" => attribute_method_parameters_from_bytes(input_2)?,
            "Module" => attribute_module_from_bytes(input_2)?,
            "ModuleMainClass" => attribute_module_main_class_from_bytes(input_2)?,
            "ModulePackages" => attribute_module_packages_from_bytes(input_2)?,
            "NestHost" => attribute_nest_host_from_bytes(input_2)?,
            "NestMembers" => attribute_nest_members_from_bytes(input_2)?,
            "PermittedSubclasses" => attribute_permitted_subclasses_from_bytes(input_2)?,
            "Record" => attribute_record_from_bytes(input_2, constant_pool)?,
            "RuntimeInvisibleAnnotations" => {
                attribute_runtime_invisible_annotations_from_bytes(input_2)?
            }
            "RuntimeInvisibleParameterAnnotations" => {
                attribute_runtime_invisible_parameter_annotations_from_bytes(input_2)?
            }
            "RuntimeInvisibleTypeAnnotations" => {
                attribute_runtime_invisible_type_annotations_from_bytes(input_2)?
            }
            "RuntimeVisibleAnnotations" => {
                attribute_runtime_visible_annotations_from_bytes(input_2)?
            }
            "RuntimeVisibleParameterAnnotations" => {
                attribute_runtime_visible_parameter_annotations_from_bytes(input_2)?
            }
            "RuntimeVisibleTypeAnnotations" => {
                attribute_runtime_visible_type_annotations_from_bytes(input_2)?
            }
            "Signature" => attribute_signature_from_bytes(input_2)?,
            "SourceDebugExtension" => attribute_source_debug_extension_from_bytes(input_2, length)?,
            "SourceFile" => todo!(),
            "StackMapTable" => todo!(),
            "Synthetic" => todo!(),
            _ => return Err(Err::Failure(Error::new(bytes, ErrorKind::Tag))),
        }
    };

    Ok((input_3, Attribute { info }))
}

fn attribute_annotation_default_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, element_value) = element_value_from_bytes(bytes)?;

    Ok((
        input,
        AttributeInfo::AnnotationDefault {
            default_value: element_value,
        },
    ))
}

fn attribute_bootstrap_methods_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, bootstrap_methods) = length_count(be_u16, bootstrap_method_from_bytes)(bytes)?;

    Ok((input, AttributeInfo::BootstrapMethods { bootstrap_methods }))
}

fn attribute_code_from_bytes<'a>(
    bytes: &'a [u8],
    constant_pool: &[ConstantPoolEntry<'a>],
) -> IResult<&'a [u8], AttributeInfo<'a>> {
    let (input_1, max_stack) = be_u16(bytes)?;
    let (input_2, max_locals) = be_u16(input_1)?;
    let (input_3, code_length) = be_u16(input_2)?;
    let (input_4, code) = take(code_length as usize)(input_3)?;
    let (input_5, exception_table) = exception_table_from_bytes(input_4)?;
    let (input_6, attributes) =
        length_count(be_u16, |bytes| attribute_from_bytes(bytes, constant_pool))(input_5)?;

    Ok((
        input_6,
        AttributeInfo::Code {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        },
    ))
}

fn attribute_constant_value_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, constantvalue_index) = be_u16(bytes)?;

    Ok((
        input,
        AttributeInfo::ConstantValue {
            constantvalue_index,
        },
    ))
}

fn attribute_enclosing_method_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input_1, class_index) = be_u16(bytes)?;
    let (input_2, method_index) = be_u16(input_1)?;

    Ok((
        input_2,
        AttributeInfo::EnclosingMethod {
            class_index,
            method_index,
        },
    ))
}

fn attribute_exceptions_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, exception_index_table) = length_count(be_u16, be_u16)(bytes)?;

    Ok((
        input,
        AttributeInfo::Exceptions {
            exception_index_table,
        },
    ))
}

fn attribute_inner_classes_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, classes) = length_count(be_u16, inner_class_from_bytes)(bytes)?;

    Ok((input, AttributeInfo::InnerClasses { classes }))
}

fn attribute_line_number_table_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, line_number_table) = length_count(be_u16, line_number_from_bytes)(bytes)?;

    Ok((input, AttributeInfo::LineNumberTable { line_number_table }))
}

fn attribute_local_variable_table_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, local_variable_table) = length_count(be_u16, local_variable_from_bytes)(bytes)?;

    Ok((
        input,
        AttributeInfo::LocalVariableTable {
            local_variable_table,
        },
    ))
}

fn attribute_local_variable_type_table_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, local_variable_type_table) =
        length_count(be_u16, local_variable_type_from_bytes)(bytes)?;

    Ok((
        input,
        AttributeInfo::LocalVariableTypeTable {
            local_variable_type_table,
        },
    ))
}

fn attribute_method_parameters_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, parameters) = length_count(be_u16, method_parameter_from_bytes)(bytes)?;

    Ok((input, AttributeInfo::MethodParameters { parameters }))
}

fn attribute_module_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input_1, module_name_index) = be_u16(bytes)?;
    let (input_2, module_flags) = be_u16(input_1)?;
    let (input_3, module_version_index) = be_u16(input_2)?;
    let (input_4, requires) = length_count(be_u16, module_require_from_bytes)(input_3)?;
    let (input_5, exports) = length_count(be_u16, module_export_from_bytes)(input_4)?;
    let (input_6, opens) = length_count(be_u16, module_opens_from_bytes)(input_5)?;
    let (input_7, uses) = length_count(be_u16, be_u16)(input_6)?;
    let (input_8, provides) = length_count(be_u16, module_provides_from_bytes)(input_7)?;

    Ok((
        input_8,
        AttributeInfo::Module {
            module_name_index,
            module_flags,
            module_version_index,
            requires,
            exports,
            opens,
            uses,
            provides,
        },
    ))
}

fn attribute_module_main_class_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, main_class_index) = be_u16(bytes)?;

    Ok((input, AttributeInfo::ModuleMainClass { main_class_index }))
}

fn attribute_module_packages_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, package_index) = length_count(be_u16, be_u16)(bytes)?;

    Ok((input, AttributeInfo::ModulePackages { package_index }))
}

fn attribute_nest_host_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, host_class_index) = be_u16(bytes)?;

    Ok((input, AttributeInfo::NestHost { host_class_index }))
}

fn attribute_nest_members_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, classes) = length_count(be_u16, be_u16)(bytes)?;

    Ok((input, AttributeInfo::NestMembers { classes }))
}

fn attribute_permitted_subclasses_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, classes) = length_count(be_u16, be_u16)(bytes)?;

    Ok((input, AttributeInfo::PermittedSubclasses { classes }))
}

fn attribute_record_from_bytes<'a>(
    bytes: &'a [u8],
    constant_pool: &[ConstantPoolEntry<'a>],
) -> IResult<&'a [u8], AttributeInfo<'a>> {
    let (input, components) = length_count(be_u16, |bytes| {
        record_component_from_bytes(bytes, constant_pool)
    })(bytes)?;

    Ok((input, AttributeInfo::Record { components }))
}

fn attribute_runtime_invisible_annotations_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, annotations) = length_count(be_u16, annotation_from_bytes)(bytes)?;

    Ok((
        input,
        AttributeInfo::RuntimeInvisibleAnnotations { annotations },
    ))
}

fn attribute_runtime_invisible_parameter_annotations_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, parameter_annotations) = length_count(be_u16, annotation_from_bytes)(bytes)?;

    Ok((
        input,
        AttributeInfo::RuntimeInvisibleParameterAnnotations {
            parameter_annotations,
        },
    ))
}

fn attribute_runtime_invisible_type_annotations_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, type_annotations) = length_count(be_u16, type_annotation_from_bytes)(bytes)?;

    Ok((
        input,
        AttributeInfo::RuntimeInvisibleTypeAnnotations { type_annotations },
    ))
}

fn attribute_runtime_visible_annotations_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, annotations) = length_count(be_u16, annotation_from_bytes)(bytes)?;

    Ok((
        input,
        AttributeInfo::RuntimeVisibleAnnotations { annotations },
    ))
}

fn attribute_runtime_visible_parameter_annotations_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, parameter_annotations) = length_count(be_u16, annotation_from_bytes)(bytes)?;

    Ok((
        input,
        AttributeInfo::RuntimeVisibleParameterAnnotations {
            parameter_annotations,
        },
    ))
}

fn attribute_runtime_visible_type_annotations_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, type_annotations) = length_count(be_u16, type_annotation_from_bytes)(bytes)?;

    Ok((
        input,
        AttributeInfo::RuntimeVisibleTypeAnnotations { type_annotations },
    ))
}

fn attribute_signature_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, signature_index) = be_u16(bytes)?;

    Ok((input, AttributeInfo::Signature { signature_index }))
}

fn attribute_source_debug_extension_from_bytes<'a>(
    bytes: &'a [u8],
    length: u32,
) -> IResult<&[u8], AttributeInfo<'a>> {
    let (input, debug_extension) = take(length as usize)(bytes)?;

    Ok((
        input,
        AttributeInfo::SourceDebugExtension { debug_extension },
    ))
}

fn bootstrap_method_from_bytes(bytes: &[u8]) -> IResult<&[u8], BootstrapMethod> {
    let (input_1, bootstrap_method_ref) = be_u16(bytes)?;
    let (input_2, bootstrap_arguments) = length_count(be_u16, be_u16)(input_1)?;

    Ok((
        input_2,
        BootstrapMethod {
            bootstrap_method_ref,
            bootstrap_arguments,
        },
    ))
}

fn classfile_version_from_bytes(bytes: &[u8]) -> IResult<&[u8], Version> {
    let (input_1, minor) = be_u16(bytes)?;
    let (input_2, major) = be_u16(input_1)?;

    Ok((input_2, Version { minor, major }))
}

fn constant_pool_entry_from_bytes<'a>(bytes: &'a [u8]) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, tag) = be_u8(bytes)?;

    match tag {
        1 => constant_pool_utf8_entry_from_bytes(input),
        3 => constant_pool_integer_entry_from_bytes(input),
        4 => constant_pool_float_entry_from_bytes(input),
        5 => constant_pool_long_entry_from_bytes(input),
        6 => constant_pool_double_entry_from_bytes(input),
        7 => constant_pool_class_entry_from_bytes(input),
        8 => constant_pool_string_entry_from_bytes(input),
        9 => constant_pool_field_ref_entry_from_bytes(input),
        10 => constant_pool_method_ref_entry_from_bytes(input),
        11 => constant_pool_instance_method_ref_entry_from_bytes(input),
        12 => constant_pool_name_and_type_entry_from_bytes(input),
        15 => constant_pool_method_handle_entry_from_bytes(input),
        16 => constant_pool_method_type_entry_from_bytes(input),
        17 => constant_pool_dynamic_entry_from_bytes(input),
        18 => constant_pool_invoke_dynamic_entry_from_bytes(input),
        19 => constant_pool_module_entry_from_bytes(input),
        20 => constant_pool_package_entry_from_bytes(input),
        _ => Err(Err::Error(Error::new(bytes, ErrorKind::Tag))),
    }
}

fn constant_pool_class_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, name_index) = be_u16(bytes)?;

    Ok((input, ConstantPoolEntry::Class { name_index }))
}

fn constant_pool_double_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, high_bytes) = be_u32(bytes)?;
    let (input_2, low_bytes) = be_u32(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::Double {
            value: f64::from_bits((high_bytes as u64) << 32 + low_bytes as u64),
        },
    ))
}

fn constant_pool_dynamic_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, bootstrap_method_attr_index) = be_u16(bytes)?;
    let (input_2, name_and_type_index) = be_u16(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::Dynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        },
    ))
}

fn constant_pool_float_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, float) = be_u32(bytes)?;

    Ok((
        input,
        ConstantPoolEntry::Float {
            value: f32::from_bits(float),
        },
    ))
}

fn constant_pool_field_ref_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, class_index) = be_u16(bytes)?;
    let (input_2, name_and_type_index) = be_u16(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::FieldRef {
            class_index,
            name_and_type_index,
        },
    ))
}

fn constant_pool_instance_method_ref_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, class_index) = be_u16(bytes)?;
    let (input_2, name_and_type_index) = be_u16(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::InstanceMethodRef {
            class_index,
            name_and_type_index,
        },
    ))
}

fn constant_pool_integer_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, integer) = be_u32(bytes)?;

    Ok((input, ConstantPoolEntry::Integer { bytes: integer }))
}

fn constant_pool_invoke_dynamic_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, bootstrap_method_attr_index) = be_u16(bytes)?;
    let (input_2, name_and_type_index) = be_u16(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::InvokeDynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        },
    ))
}

fn constant_pool_long_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, high_bytes) = be_u32(bytes)?;
    let (input_2, low_bytes) = be_u32(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::Long {
            value: (high_bytes as u64) << 32 + low_bytes as u64,
        },
    ))
}

fn constant_pool_method_handle_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, reference_kind) = be_u8(bytes)?;
    let (input_2, reference_index) = be_u16(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::MethodHandle {
            reference_kind,
            reference_index,
        },
    ))
}

fn constant_pool_method_type_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, reference_index) = be_u16(bytes)?;

    Ok((input, ConstantPoolEntry::MethodType { reference_index }))
}

fn constant_pool_method_ref_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, class_index) = be_u16(bytes)?;
    let (input_2, name_and_type_index) = be_u16(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::MethodRef {
            class_index,
            name_and_type_index,
        },
    ))
}

fn constant_pool_module_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, name_index) = be_u16(bytes)?;

    Ok((input, ConstantPoolEntry::Module { name_index }))
}

fn constant_pool_name_and_type_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, name_index) = be_u16(bytes)?;
    let (input_2, descriptor_index) = be_u16(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::NameAndType {
            name_index,
            descriptor_index,
        },
    ))
}

fn constant_pool_package_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, name_index) = be_u16(bytes)?;

    Ok((input, ConstantPoolEntry::Package { name_index }))
}

fn constant_pool_string_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, string_index) = be_u16(bytes)?;

    Ok((input, ConstantPoolEntry::String { string_index }))
}

fn constant_pool_utf8_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, length) = be_u16(bytes)?;
    let (input_2, str_bytes) = take(length as usize)(input_1)?;

    Ok((input_2, ConstantPoolEntry::Utf8 { bytes: str_bytes }))
}

fn element_value_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], ElementValue> {
    let (input_1, tag) = be_u8(bytes)?;

    Ok(match tag as char {
        // byte or char or double or float or int or long or short or boolean or string
        'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => {
            let (input_2, const_value_index) = be_u16(input_1)?;
            (input_2, ElementValue::ConstValue(const_value_index))
        }
        // enum class
        'e' => {
            let (input_2, type_name_index) = be_u16(input_1)?;
            let (input_3, const_name_index) = be_u16(input_2)?;
            (
                input_3,
                ElementValue::EnumConst {
                    type_name_index,
                    const_name_index,
                },
            )
        }
        // class
        'c' => {
            let (input_2, class_info_index) = be_u16(input_1)?;
            (input_2, ElementValue::ClassInfo(class_info_index))
        }
        // annotation interface
        '@' => {
            let (input_2, annotation) = annotation_from_bytes(input_1)?;
            (input_2, ElementValue::Annotation(annotation))
        }
        // array type
        '[' => {
            let (input_2, values) = length_count(be_u16, element_value_from_bytes)(input_1)?;
            (input_2, ElementValue::Array { values })
        }
        _ => return Err(Err::Failure(Error::new(bytes, ErrorKind::Tag))),
    })
}

fn element_value_pair_from_bytes<'a>(bytes: &[u8]) -> IResult<&[u8], ElementValuePair> {
    let (input_1, element_name_index) = be_u16(bytes)?;
    let (input_2, element_value) = element_value_from_bytes(input_1)?;

    Ok((
        input_2,
        ElementValuePair {
            element_name_index,
            value: element_value,
        },
    ))
}

fn exception_table_from_bytes(bytes: &[u8]) -> IResult<&[u8], Vec<ExceptionTableEntry>> {
    length_count(be_u16, exception_table_entry_from_bytes)(bytes)
}

fn exception_table_entry_from_bytes(bytes: &[u8]) -> IResult<&[u8], ExceptionTableEntry> {
    let (input_1, start_pc) = be_u16(bytes)?;
    let (input_2, end_pc) = be_u16(input_1)?;
    let (input_3, handler_pc) = be_u16(input_2)?;
    let (input_4, catch_type) = be_u16(input_3)?;

    Ok((
        input_4,
        ExceptionTableEntry {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        },
    ))
}

fn field_from_bytes<'a>(
    bytes: &'a [u8],
    constant_pool: &[ConstantPoolEntry<'a>],
) -> IResult<&'a [u8], Field<'a>> {
    let (input_1, access_flags) = be_u16(bytes)?;
    let (input_2, name_index) = be_u16(input_1)?;
    let (input_3, descriptor_index) = be_u16(input_2)?;
    let (input_4, attributes) =
        length_count(be_u16, |bytes| attribute_from_bytes(bytes, constant_pool))(input_3)?;

    Ok((
        input_4,
        Field {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        },
    ))
}

fn inner_class_from_bytes(bytes: &[u8]) -> IResult<&[u8], InnerClass> {
    let (input_1, inner_class_info_index) = be_u16(bytes)?;
    let (input_2, outer_class_info_index) = be_u16(input_1)?;
    let (input_3, inner_name_index) = be_u16(input_2)?;
    let (input_4, inner_class_access_flags) = be_u16(input_3)?;

    Ok((
        input_4,
        InnerClass {
            inner_class_info_index,
            outer_class_info_index,
            inner_name_index,
            inner_class_access_flags,
        },
    ))
}

fn line_number_from_bytes(bytes: &[u8]) -> IResult<&[u8], LineNumber> {
    let (input_1, start_pc) = be_u16(bytes)?;
    let (input_2, line_number) = be_u16(input_1)?;

    Ok((
        input_2,
        LineNumber {
            start_pc,
            line_number,
        },
    ))
}

fn local_var_from_bytes(bytes: &[u8]) -> IResult<&[u8], LocalVar> {
    let (input_1, start_pc) = be_u16(bytes)?;
    let (input_2, length) = be_u16(input_1)?;
    let (input_3, index) = be_u16(input_2)?;

    Ok((
        input_3,
        LocalVar {
            start_pc,
            length,
            index,
        },
    ))
}

fn local_variable_from_bytes(bytes: &[u8]) -> IResult<&[u8], LocalVariable> {
    let (input_1, start_pc) = be_u16(bytes)?;
    let (input_2, length) = be_u16(input_1)?;
    let (input_3, name_index) = be_u16(input_2)?;
    let (input_4, descriptor_index) = be_u16(input_3)?;
    let (input_5, index) = be_u16(input_4)?;

    Ok((
        input_5,
        LocalVariable {
            start_pc,
            length,
            name_index,
            descriptor_index,
            index,
        },
    ))
}

fn local_variable_type_from_bytes(bytes: &[u8]) -> IResult<&[u8], LocalVariableType> {
    let (input_1, start_pc) = be_u16(bytes)?;
    let (input_2, length) = be_u16(input_1)?;
    let (input_3, name_index) = be_u16(input_2)?;
    let (input_4, descriptor_index) = be_u16(input_3)?;
    let (input_5, index) = be_u16(input_4)?;

    Ok((
        input_5,
        LocalVariableType {
            start_pc,
            length,
            name_index,
            descriptor_index,
            index,
        },
    ))
}

fn method_from_bytes<'a>(
    bytes: &'a [u8],
    constant_pool: &[ConstantPoolEntry<'a>],
) -> IResult<&'a [u8], Method<'a>> {
    let (input_1, access_flags) = be_u16(bytes)?;
    let (input_2, name_index) = be_u16(input_1)?;
    let (input_3, descriptor_index) = be_u16(input_2)?;
    let (input_4, attributes) =
        length_count(be_u16, |bytes| attribute_from_bytes(bytes, constant_pool))(input_3)?;

    Ok((
        input_4,
        Method {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        },
    ))
}

fn method_parameter_from_bytes(bytes: &[u8]) -> IResult<&[u8], MethodParameter> {
    let (input_1, name_index) = be_u16(bytes)?;
    let (input_2, access_flags) = be_u16(input_1)?;

    Ok((
        input_2,
        MethodParameter {
            name_index,
            access_flags,
        },
    ))
}

fn module_export_from_bytes(bytes: &[u8]) -> IResult<&[u8], ModuleExports> {
    let (input_1, exports_index) = be_u16(bytes)?;
    let (input_2, exports_flags) = be_u16(input_1)?;
    let (input_3, exports_to_indices) = length_count(be_u16, be_u16)(input_2)?;

    Ok((
        input_3,
        ModuleExports {
            exports_index,
            exports_flags,
            exports_to_indices,
        },
    ))
}

fn module_opens_from_bytes(bytes: &[u8]) -> IResult<&[u8], ModuleOpens> {
    let (input_1, opens_index) = be_u16(bytes)?;
    let (input_2, opens_flags) = be_u16(input_1)?;
    let (input_3, opens_to_indices) = length_count(be_u16, be_u16)(input_2)?;

    Ok((
        input_3,
        ModuleOpens {
            opens_index,
            opens_flags,
            opens_to_indices,
        },
    ))
}

fn module_provides_from_bytes(bytes: &[u8]) -> IResult<&[u8], ModuleProvides> {
    let (input_1, provides_index) = be_u16(bytes)?;
    let (input_2, provides_with_indices) = length_count(be_u16, be_u16)(input_1)?;

    Ok((
        input_2,
        ModuleProvides {
            provides_index,
            provides_with_indices,
        },
    ))
}

fn module_require_from_bytes(bytes: &[u8]) -> IResult<&[u8], ModuleRequires> {
    let (input_1, requires_index) = be_u16(bytes)?;
    let (input_2, requires_flags) = be_u16(input_1)?;
    let (input_3, requires_version_index) = be_u16(input_2)?;

    Ok((
        input_3,
        ModuleRequires {
            requires_index,
            requires_flags,
            requires_version_index,
        },
    ))
}

fn record_component_from_bytes<'a>(
    bytes: &'a [u8],
    constant_pool: &[ConstantPoolEntry<'a>],
) -> IResult<&'a [u8], RecordComponent<'a>> {
    let (input_1, name_index) = be_u16(bytes)?;
    let (input_2, descriptor_index) = be_u16(input_1)?;
    let (input_3, attributes) =
        length_count(be_u16, |bytes| attribute_from_bytes(bytes, constant_pool))(input_2)?;

    Ok((
        input_3,
        RecordComponent {
            name_index,
            descriptor_index,
            attributes,
        },
    ))
}

fn target_info_from_bytes(bytes: &[u8], target_type: u8) -> IResult<&[u8], TargetInfo> {
    Ok(match target_type {
        0x00 | 0x01 => {
            let (input_1, type_parameter_index) = be_u8(bytes)?;

            (input_1, TargetInfo::TypeParameter(type_parameter_index))
        }
        0x10 => {
            let (input_1, supertype_index) = be_u16(bytes)?;

            (input_1, TargetInfo::Supertype(supertype_index))
        }
        0x11 | 0x12 => {
            let (input_1, type_parameter_index) = be_u8(bytes)?;
            let (input_2, bound_index) = be_u8(input_1)?;

            (
                input_2,
                TargetInfo::TypeParameterBound {
                    type_parameter_index,
                    bound_index,
                },
            )
        }
        0x13 | 0x14 | 0x15 => (bytes, TargetInfo::Empty),
        0x16 => {
            let (input, formal_parameter_index) = be_u8(bytes)?;

            (input, TargetInfo::FormalParameter(formal_parameter_index))
        }
        0x17 => {
            let (input, throws_type_index) = be_u16(bytes)?;

            (input, TargetInfo::Throws(throws_type_index))
        }
        0x40 | 0x41 => {
            let (input, table) = length_count(be_u16, local_var_from_bytes)(bytes)?;

            (input, TargetInfo::LocalVar { table })
        }
        0x42 => {
            let (input, exception_table_index) = be_u16(bytes)?;

            (input, TargetInfo::Catch(exception_table_index))
        }
        0x43 | 0x44 | 0x45 | 0x46 => {
            let (input, offset) = be_u16(bytes)?;

            (input, TargetInfo::Offset(offset))
        }
        0x47 | 0x48 | 0x49 | 0x4A | 0x4B => {
            let (input_1, offset) = be_u16(bytes)?;
            let (input_2, type_argument_index) = be_u8(input_1)?;

            (
                input_2,
                TargetInfo::TypeArgument {
                    offset,
                    type_argument_index,
                },
            )
        }
        _ => return Err(Err::Error(Error::new(bytes, ErrorKind::Tag))),
    })
}

fn type_annotation_from_bytes(bytes: &[u8]) -> IResult<&[u8], TypeAnnotation> {
    let (input_1, target_type) = be_u8(bytes)?;
    let (input_2, target_info) = target_info_from_bytes(input_1, target_type)?;
    let (input_3, target_path) = type_path_from_bytes(input_2)?;
    let (input_4, type_index) = be_u16(input_3)?;
    let (input_5, element_value_pairs) =
        length_count(be_u16, element_value_pair_from_bytes)(input_4)?;

    Ok((
        input_5,
        TypeAnnotation {
            target_type,
            target_info,
            target_path,
            type_index,
            element_value_pairs,
        },
    ))
}

fn type_path_from_bytes(bytes: &[u8]) -> IResult<&[u8], TypePath> {
    let (input, path) = length_count(be_u8, type_path_segment_from_bytes)(bytes)?;

    Ok((input, TypePath { path }))
}

fn type_path_segment_from_bytes(bytes: &[u8]) -> IResult<&[u8], TypePathSegment> {
    let (input_1, type_path_kind) = be_u8(bytes)?;
    let (input_2, type_argument_index) = be_u8(input_1)?;

    Ok((
        input_2,
        TypePathSegment {
            type_path_kind,
            type_argument_index,
        },
    ))
}
