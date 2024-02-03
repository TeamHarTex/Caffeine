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
use crate::spec::Method;
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

    // ignore length
    let (input_2, _) = be_u32(input_1)?;

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
            high_bytes,
            low_bytes,
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

    Ok((input, ConstantPoolEntry::Float { bytes: float }))
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
            high_bytes,
            low_bytes,
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
