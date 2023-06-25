#![forbid(unsafe_code)]

use schema_struct::schema_struct;

/// Test constructing a struct from a schema.
#[test]
fn test_from_schema() {
    schema_struct!(
        vis = pub,
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "ProductSchema",
            "description": "A product from Acme's catalog",
            "type": "object",
            "properties": {
                "id": {
                    "description": "The unique identifier for a product",
                    "type": "integer"
                },
                "name": {
                    "description": "Name of the product",
                    "type": "string"
                },
                "price": {
                    "type": "number",
                    "minimum": 0,
                    "exclusiveMinimum": true
                }
            },
            "required": ["id", "name", "price"]
        }
    );

    let product_json = "{\"id\":5,\"name\":\"product name\",\"price\":12.34}";
    let product = ProductSchema::from_str(product_json).unwrap();
    assert_eq!(&product.to_str().unwrap(), product_json);

    assert_eq!(product.id, 5);
    assert_eq!(product.name, "product name".to_owned());
    assert_eq!(product.price, 12.34);
}

/// Test constructing a struct from a schema in a file.
#[test]
fn test_from_file() {
    schema_struct!(file = "schema-struct/tests/schemas/product.json");

    let product_json = "{\"id\":5,\"name\":\"product name\",\"price\":12.34}";
    let product = Product::from_str(product_json).unwrap();
    assert_eq!(&product.to_str().unwrap(), product_json);

    assert_eq!(product.id, 5);
    assert_eq!(product.name, "product name".to_owned());
    assert_eq!(product.price, 12.34);
}

/// Test constructing a struct from a schema at a URL.
#[test]
fn test_from_url() {
    // TODO

    // schema_struct!(
    //     ident = Draft4,
    //     url = "http://json-schema.org/draft-04/schema#"
    // );
}

/// Test constructing a struct with optional fields.
#[test]
fn test_optional_field() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithOptional",
            "description": "A schema with a nullable field",
            "type": "object",
            "properties": {
                "name": {
                    "type": "string"
                }
            }
        }
    );

    let json_without_null = "{\"name\":\"this is not null\"}";
    let value_without_null = SchemaWithOptional::from_str(json_without_null).unwrap();
    assert_eq!(&value_without_null.to_str().unwrap(), json_without_null);
    assert_eq!(value_without_null.name, Some("this is not null".to_owned()));

    let json_with_null = "{\"name\":null}";
    let value_with_null = SchemaWithOptional::from_str(json_with_null).unwrap();
    assert_eq!(&value_with_null.to_str().unwrap(), json_with_null);
    assert_eq!(value_with_null.name, None);

    let json_with_null_empty = "{}";
    let value_with_null_empty = SchemaWithOptional::from_str(json_with_null_empty).unwrap();
    assert_eq!(&value_with_null_empty.to_str().unwrap(), json_with_null);
    assert_eq!(value_with_null_empty.name, None);
}

/// Test constructing a struct with null fields.
#[test]
fn test_null() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithNull",
            "description": "A schema with a field of type `null`",
            "type": "object",
            "properties": {
                "null_field": {
                    "type": "null"
                }
            },
            "required": ["null_field"]
        }
    );

    let json_with_null = "{\"null_field\":null}";
    let value_with_null = SchemaWithNull::from_str(json_with_null).unwrap();
    assert_eq!(&value_with_null.to_str().unwrap(), json_with_null);
}

/// Test constructing a struct with boolean fields.
#[test]
fn test_boolean() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithBoolean",
            "description": "A schema with a field of type `boolean`",
            "type": "object",
            "properties": {
                "boolean_field": {
                    "type": "boolean"
                }
            },
            "required": ["boolean_field"]
        }
    );

    let json_with_false = "{\"boolean_field\":false}";
    let value_with_false = SchemaWithBoolean::from_str(json_with_false).unwrap();
    assert_eq!(&value_with_false.to_str().unwrap(), json_with_false);
    assert!(!value_with_false.boolean_field);

    let json_with_true = "{\"boolean_field\":true}";
    let value_with_true = SchemaWithBoolean::from_str(json_with_true).unwrap();
    assert_eq!(&value_with_true.to_str().unwrap(), json_with_true);
    assert!(value_with_true.boolean_field);
}

/// Test constructing a struct with integer fields.
#[test]
fn test_integer() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithInteger",
            "description": "A schema with a field of type `integer`",
            "type": "object",
            "properties": {
                "integer_field": {
                    "type": "integer"
                }
            },
            "required": ["integer_field"]
        }
    );

    let json_with_pos_int = "{\"integer_field\":1729}";
    let value_with_pos_int = SchemaWithInteger::from_str(json_with_pos_int).unwrap();
    assert_eq!(&value_with_pos_int.to_str().unwrap(), json_with_pos_int);
    assert_eq!(value_with_pos_int.integer_field, 1729);

    let json_with_neg_int = "{\"integer_field\":-123}";
    let value_with_neg_int = SchemaWithInteger::from_str(json_with_neg_int).unwrap();
    assert_eq!(&value_with_neg_int.to_str().unwrap(), json_with_neg_int);
    assert_eq!(value_with_neg_int.integer_field, -123);

    let json_with_zero = "{\"integer_field\":0}";
    let value_with_zero = SchemaWithInteger::from_str(json_with_zero).unwrap();
    assert_eq!(&value_with_zero.to_str().unwrap(), json_with_zero);
    assert_eq!(value_with_zero.integer_field, 0);
}

/// Test constructing a struct with numeric fields.
#[test]
fn test_number() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithNumber",
            "description": "A schema with a field of type `number`",
            "type": "object",
            "properties": {
                "number_field": {
                    "type": "number"
                }
            },
            "required": ["number_field"]
        }
    );

    let json_with_pos_int = "{\"number_field\":1729.0}";
    let value_with_pos_int = SchemaWithNumber::from_str(json_with_pos_int).unwrap();
    assert_eq!(&value_with_pos_int.to_str().unwrap(), json_with_pos_int);
    assert_eq!(value_with_pos_int.number_field, 1729.0);

    let json_with_neg_int = "{\"number_field\":-123.0}";
    let value_with_neg_int = SchemaWithNumber::from_str(json_with_neg_int).unwrap();
    assert_eq!(&value_with_neg_int.to_str().unwrap(), json_with_neg_int);
    assert_eq!(value_with_neg_int.number_field, -123.0);

    let json_with_pos_float = "{\"number_field\":9.8765}";
    let value_with_pos_float = SchemaWithNumber::from_str(json_with_pos_float).unwrap();
    assert_eq!(&value_with_pos_float.to_str().unwrap(), json_with_pos_float);
    assert_eq!(value_with_pos_float.number_field, 9.8765);

    let json_with_neg_float = "{\"number_field\":-0.618}";
    let value_with_neg_float = SchemaWithNumber::from_str(json_with_neg_float).unwrap();
    assert_eq!(&value_with_neg_float.to_str().unwrap(), json_with_neg_float);
    assert_eq!(value_with_neg_float.number_field, -0.618);

    let json_with_zero = "{\"number_field\":0.0}";
    let value_with_zero = SchemaWithNumber::from_str(json_with_zero).unwrap();
    assert_eq!(&value_with_zero.to_str().unwrap(), json_with_zero);
    assert_eq!(value_with_zero.number_field, 0.0);
}

/// Test constructing a struct with string fields.
#[test]
fn test_string() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithString",
            "description": "A schema with a field of type `string`",
            "type": "object",
            "properties": {
                "string_field": {
                    "type": "string"
                }
            },
            "required": ["string_field"]
        }
    );

    let json_with_empty_str = "{\"string_field\":\"\"}";
    let value_with_empty_str = SchemaWithString::from_str(json_with_empty_str).unwrap();
    assert_eq!(&value_with_empty_str.to_str().unwrap(), json_with_empty_str);
    assert_eq!(value_with_empty_str.string_field, "");

    let json_with_str = "{\"string_field\":\"a string value\"}";
    let value_with_str = SchemaWithString::from_str(json_with_str).unwrap();
    assert_eq!(&value_with_str.to_str().unwrap(), json_with_str);
    assert_eq!(value_with_str.string_field, "a string value");
}

/// Test constructing a struct with array fields.
#[test]
fn test_array() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithArray",
            "description": "A schema with a field of type `array`",
            "type": "object",
            "properties": {
                "array_field": {
                    "type": "array",
                    "items": {
                        "type": "integer"
                    }
                }
            },
            "required": ["array_field"]
        }
    );

    let json_with_empty_array = "{\"array_field\":[]}";
    let value_with_empty_array = SchemaWithArray::from_str(json_with_empty_array).unwrap();
    assert_eq!(
        &value_with_empty_array.to_str().unwrap(),
        json_with_empty_array
    );
    assert_eq!(value_with_empty_array.array_field, Vec::<i64>::new());

    let json_with_array = "{\"array_field\":[1,3,7,9]}";
    let value_with_array = SchemaWithArray::from_str(json_with_array).unwrap();
    assert_eq!(&value_with_array.to_str().unwrap(), json_with_array);
    assert_eq!(value_with_array.array_field, vec![1, 3, 7, 9]);
}

/// Test constructing a struct with object fields.
#[test]
fn test_object() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithObject",
            "description": "A schema with a field of type `object`",
            "type": "object",
            "properties": {
                "object_field": {
                    "type": "object",
                    "properties": {
                        "inner_field": {
                            "type": "string"
                        }
                    },
                    "required": ["inner_field"]
                }
            },
            "required": ["object_field"]
        }
    );

    let json_with_object = "{\"object_field\":{\"inner_field\":\"an inner object field\"}}";
    let value_with_object = SchemaWithObject::from_str(json_with_object).unwrap();
    assert_eq!(&value_with_object.to_str().unwrap(), json_with_object);
    assert_eq!(
        value_with_object.object_field.inner_field,
        "an inner object field"
    );
}

/// Test constructing a struct with enum fields.
#[test]
fn test_enum() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithEnum",
            "description": "A schema with an enum field",
            "type": "object",
            "properties": {
                "enum_field": {
                    "enum": ["first", "second", "third"]
                }
            },
            "required": ["enum_field"]
        }
    );

    let json_with_enum_first = "{\"enum_field\":\"first\"}";
    let value_with_enum_first = SchemaWithEnum::from_str(json_with_enum_first).unwrap();
    assert_eq!(
        &value_with_enum_first.to_str().unwrap(),
        json_with_enum_first
    );
    assert!(matches!(
        value_with_enum_first.enum_field,
        SchemaWithEnumEnumField::First
    ));

    let json_with_enum_second = "{\"enum_field\":\"second\"}";
    let value_with_enum_second = SchemaWithEnum::from_str(json_with_enum_second).unwrap();
    assert_eq!(
        &value_with_enum_second.to_str().unwrap(),
        json_with_enum_second
    );
    assert!(matches!(
        value_with_enum_second.enum_field,
        SchemaWithEnumEnumField::Second
    ));

    let json_with_enum_third = "{\"enum_field\":\"third\"}";
    let value_with_enum_third = SchemaWithEnum::from_str(json_with_enum_third).unwrap();
    assert_eq!(
        &value_with_enum_third.to_str().unwrap(),
        json_with_enum_third
    );
    assert!(matches!(
        value_with_enum_third.enum_field,
        SchemaWithEnumEnumField::Third
    ));

    let json_with_enum_invalid_variant = "{\"enum_field\":\"fourth\"}";
    assert!(SchemaWithEnum::from_str(json_with_enum_invalid_variant).is_err());
}

/// Test constructing a struct with tuple fields.
#[test]
fn test_tuple() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithTuple",
            "description": "A schema with a tuple field",
            "type": "object",
            "properties": {
                "tuple_field": {
                    "type": "array",
                    "prefixItems": [
                        {
                            "type": "integer",
                            "description": "The address number"
                        },
                        {
                            "type": "string",
                            "description": "The street name"
                        },
                        {
                            "enum": ["Street", "Avenue", "Boulevard"],
                            "description": "The street type"
                        },
                        {
                            "enum": ["NW", "NE", "SW", "SE"],
                            "description": "The city quadrant of the address"
                        }
                    ]
                }
            },
            "required": ["tuple_field"]
        }
    );

    let json_with_tuple = "{\"tuple_field\":[1600,\"Pennsylvania\",\"Avenue\",\"NW\"]}";
    let value_with_tuple = SchemaWithTuple::from_str(json_with_tuple).unwrap();
    assert_eq!(&value_with_tuple.to_str().unwrap(), json_with_tuple);
    assert_eq!(value_with_tuple.tuple_field.0, 1600);
    assert_eq!(value_with_tuple.tuple_field.1, "Pennsylvania".to_owned());
    assert!(matches!(
        value_with_tuple.tuple_field.2,
        SchemaWithTupleTupleField2::Avenue
    ));
    assert!(matches!(
        value_with_tuple.tuple_field.3,
        SchemaWithTupleTupleField3::Nw
    ));
}

/// Test constructing a struct containing arrays of objects.
#[test]
fn test_array_of_objects() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithArrayOfObjects",
            "description": "A schema with arrays of objects",
            "type": "object",
            "properties": {
                "array_of_objects": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "object_id": {
                                "type": "integer"
                            }
                        },
                        "required": ["object_id"]
                    }
                }
            },
            "required": ["array_of_objects"]
        }
    );

    let json_with_array_of_objects = "{\"array_of_objects\":[{\"object_id\":3},{\"object_id\":7}]}";
    let value_with_array_of_objects =
        SchemaWithArrayOfObjects::from_str(json_with_array_of_objects).unwrap();
    assert_eq!(
        &value_with_array_of_objects.to_str().unwrap(),
        json_with_array_of_objects
    );
    assert_eq!(
        value_with_array_of_objects.array_of_objects,
        vec![
            SchemaWithArrayOfObjectsArrayOfObjects { object_id: 3 },
            SchemaWithArrayOfObjectsArrayOfObjects { object_id: 7 }
        ]
    );

    let json_with_empty_array_of_objects = "{\"array_of_objects\":[]}";
    let value_with_empty_array_of_objects =
        SchemaWithArrayOfObjects::from_str(json_with_empty_array_of_objects).unwrap();
    assert_eq!(
        &value_with_empty_array_of_objects.to_str().unwrap(),
        json_with_empty_array_of_objects
    );
    assert_eq!(value_with_empty_array_of_objects.array_of_objects, vec![]);
}

/// Test constructing a struct containing nested arrays.
#[test]
fn test_nested_arrays() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithNestedArrays",
            "description": "A schema with nested arrays",
            "type": "object",
            "properties": {
                "nested_arrays": {
                    "description": "This should be defined as `Vec<Vec<Vec<...>>>`",
                    "type": "array",
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "array",
                            "items": {
                                "type": "integer"
                            }
                        }
                    }
                }
            },
            "required": ["nested_arrays"]
        }
    );

    let json_with_nested_arrays = "{\"nested_arrays\":[[[1,2],[3,4]],[[5,6],[7,8]]]}";
    let value_with_nested_arrays =
        SchemaWithNestedArrays::from_str(json_with_nested_arrays).unwrap();
    assert_eq!(
        &value_with_nested_arrays.to_str().unwrap(),
        json_with_nested_arrays
    );
    assert_eq!(
        value_with_nested_arrays.nested_arrays,
        vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]]
    );

    let json_with_empty_nested_array1 = "{\"nested_arrays\":[]}";
    let value_with_empty_nested_array1 =
        SchemaWithNestedArrays::from_str(json_with_empty_nested_array1).unwrap();
    assert_eq!(
        &value_with_empty_nested_array1.to_str().unwrap(),
        json_with_empty_nested_array1
    );
    assert_eq!(
        value_with_empty_nested_array1.nested_arrays,
        Vec::<Vec<Vec<i64>>>::new()
    );

    let json_with_empty_nested_array2 = "{\"nested_arrays\":[[]]}";
    let value_with_empty_nested_array2 =
        SchemaWithNestedArrays::from_str(json_with_empty_nested_array2).unwrap();
    assert_eq!(
        &value_with_empty_nested_array2.to_str().unwrap(),
        json_with_empty_nested_array2
    );
    assert_eq!(
        value_with_empty_nested_array2.nested_arrays,
        vec![Vec::<Vec<i64>>::new()]
    );

    let json_with_empty_nested_array3 = "{\"nested_arrays\":[[[]]]}";
    let value_with_empty_nested_array3 =
        SchemaWithNestedArrays::from_str(json_with_empty_nested_array3).unwrap();
    assert_eq!(
        &value_with_empty_nested_array3.to_str().unwrap(),
        json_with_empty_nested_array3
    );
    assert_eq!(
        value_with_empty_nested_array3.nested_arrays,
        vec![vec![Vec::<i64>::new()]]
    );
}

/// Test constructing a struct containing nested objects.
#[test]
fn test_nested_objects() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithNestedObjects",
            "description": "A schema with nested objects",
            "type": "object",
            "properties": {
                "foo": {
                    "type": "object",
                    "properties": {
                        "bar": {
                            "type": "object",
                            "properties": {
                                "baz": {
                                    "type": "object",
                                    "properties": {
                                        "message": {
                                            "type": "string"
                                        }
                                    },
                                    "required": ["message"]
                                }
                            },
                            "required": ["baz"]
                        }
                    },
                    "required": ["bar"]
                }
            },
            "required": ["foo"]
        }
    );

    let json_with_nested_objects =
        "{\"foo\":{\"bar\":{\"baz\":{\"message\":\"Hello, nested object!\"}}}}";
    let value_with_nested_objects =
        SchemaWithNestedObjects::from_str(json_with_nested_objects).unwrap();
    assert_eq!(
        &value_with_nested_objects.to_str().unwrap(),
        json_with_nested_objects
    );
    assert_eq!(
        value_with_nested_objects.foo.bar.baz.message,
        "Hello, nested object!"
    );
}

/// Test structs with default fields.
#[test]
fn test_default() {
    // TODO
}

/// Test refs.
#[test]
fn test_ref() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithRef",
            "description": "A schema with ref fields",
            "$defs": {
                "myInteger": {
                    "description": "An alias for an integer value",
                    "type": "integer"
                },
                "stringArray": {
                    "description": "An array of strings",
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
                "objectWithStringArray": {
                    "description": "An object containing a string array",
                    "type": "object",
                    "properties": {
                        "inner_array": {
                            "$ref": "#/$defs/stringArray"
                        }
                    },
                    "required": ["inner_array"]
                },
                "arrayWithItemsRef": {
                    "description": "An array with item type ref",
                    "type": "array",
                    "items": {
                        "$ref": "#/$defs/myInteger"
                    }
                }
            },
            "type": "object",
            "properties": {
                "my_integer_field": {
                    "$ref": "#/$defs/myInteger"
                },
                "string_array_field": {
                    "$ref": "#/$defs/stringArray"
                },
                "object_with_string_array_field": {
                    "$ref": "#/definitions/objectWithStringArray"
                },
                "array_with_items_ref_field": {
                    "$ref": "#/$defs/arrayWithItemsRef"
                },
                "self_referential_field": {
                    "$ref": "#"
                }
            },
            "required": ["string_array_field"]
        }
    );

    let json_with_ref = "{\"array_with_items_ref_field\":[1,3,7,9],\"my_integer_field\":123,\"object_with_string_array_field\":{\"inner_array\":[\"four\"]},\"self_referential_field\":{\"array_with_items_ref_field\":null,\"my_integer_field\":null,\"object_with_string_array_field\":null,\"self_referential_field\":null,\"string_array_field\":[]},\"string_array_field\":[\"one\",\"two\",\"three\"]}";
    let value_with_ref = SchemaWithRef::from_str(json_with_ref).unwrap();
    assert_eq!(&value_with_ref.to_str().unwrap(), json_with_ref);
    assert_eq!(value_with_ref.my_integer_field, Some(Box::new(123)));
    assert_eq!(
        value_with_ref.string_array_field,
        Box::new(vec!["one".to_owned(), "two".to_owned(), "three".to_owned()])
    );
    assert_eq!(
        value_with_ref.object_with_string_array_field,
        Some(Box::new(SchemaWithRefDefObjectWithStringArray {
            inner_array: Box::new(vec!["four".to_owned()])
        }))
    );
    assert_eq!(
        value_with_ref.array_with_items_ref_field,
        Some(Box::new(vec![
            Box::new(1),
            Box::new(3),
            Box::new(7),
            Box::new(9)
        ]))
    );
    assert_eq!(
        value_with_ref.self_referential_field,
        Some(Box::new(SchemaWithRef {
            my_integer_field: None,
            string_array_field: Box::<SchemaWithRefDefStringArray>::default(),
            object_with_string_array_field: None,
            array_with_items_ref_field: None,
            self_referential_field: None
        }))
    );
}

/// Test struct visibility configuration.
#[test]
fn test_vis() {
    mod vis_test {
        use super::schema_struct;

        schema_struct!(
            vis = pub,
            ident = PublicProduct,
            file = "schema-struct/tests/schemas/product.json"
        );

        schema_struct!(
            vis = ,
            ident = PrivateProduct,
            file = "schema-struct/tests/schemas/product.json"
        );
    }

    let product_json = "{\"id\":5,\"name\":\"product name\",\"price\":12.34}";
    let product = vis_test::PublicProduct::from_str(product_json).unwrap();
    assert_eq!(&product.to_str().unwrap(), product_json);

    assert_eq!(product.id, 5);
    assert_eq!(product.name, "product name".to_owned());
    assert_eq!(product.price, 12.34);
}

/// Test constructing a struct with a custom identifier.
#[test]
fn test_custom_ident() {
    schema_struct!(
        ident = CustomIdentifier,
        file = "schema-struct/tests/schemas/product.json"
    );

    let product_json = "{\"id\":5,\"name\":\"product name\",\"price\":12.34}";
    let product = CustomIdentifier::from_str(product_json).unwrap();
    assert_eq!(&product.to_str().unwrap(), product_json);

    assert_eq!(product.id, 5);
    assert_eq!(product.name, "product name".to_owned());
    assert_eq!(product.price, 12.34);
}

/// Test renaming structs and fields.
#[test]
fn test_renaming() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithRenamedFields",
            "description": "A schema with several fields that will be renamed in the struct definition",
            "type": "object",
            "properties": {
                "$schema": {
                    "type": "null"
                },
                "123strip_starting_number456": {
                    "type": "null"
                },
                "invalid%_+CHARACTERS*_here@#!0`~1,>2<&3.4/5\\6[(7)]8?^9'\"": {
                    "type": "null"
                }
            }
        }
    );

    let json_with_renamed_fields = "{\"$schema\":null,\"123strip_starting_number456\":null,\"invalid%_+CHARACTERS*_here@#!0`~1,>2<&3.4/5\\\\6[(7)]8?^9'\\\"\":null}";
    let value_with_renamed_fields =
        SchemaWithRenamedFields::from_str(json_with_renamed_fields).unwrap();
    assert_eq!(
        &value_with_renamed_fields.to_str().unwrap(),
        json_with_renamed_fields
    );
    assert_eq!(value_with_renamed_fields.schema, None);
    assert_eq!(value_with_renamed_fields.strip_starting_number_456, None);
    assert_eq!(
        value_with_renamed_fields.invalid_characters_here0123456789,
        None
    );

    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "Schema with bad title",
            "description": "A schema with several fields that will be renamed in the struct definition",
            "type": "object"
        }
    );

    let json_with_bad_title = "{}";
    let value_with_bad_title = SchemaWithBadTitle::from_str(json_with_bad_title).unwrap();
    assert_eq!(&value_with_bad_title.to_str().unwrap(), json_with_bad_title);
}

/// Test serializing and deserializing generated structs.
#[test]
fn test_serializing() {
    schema_struct!(
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "SchemaWithNestedObjects",
            "description": "A schema with nested objects",
            "type": "object",
            "properties": {
                "foo": {
                    "type": "object",
                    "properties": {
                        "bar": {
                            "type": "object",
                            "properties": {
                                "baz": {
                                    "type": "object",
                                    "properties": {
                                        "message": {
                                            "type": "string"
                                        }
                                    },
                                    "required": ["message"]
                                }
                            },
                            "required": ["baz"]
                        }
                    },
                    "required": ["bar"]
                }
            },
            "required": ["foo"]
        }
    );

    let json1 = "{\"foo\":{\"bar\":{\"baz\":{\"message\":\"Hello, nested object 1!\"}}}}";
    let value1 = SchemaWithNestedObjects::from_str(json1).unwrap();
    assert_eq!(&value1.to_str().unwrap(), json1);
    assert_eq!(value1.foo.bar.baz.message, "Hello, nested object 1!");

    let json2 = "{\"bar\":{\"baz\":{\"message\":\"Hello, nested object 2!\"}}}";
    let value2 = SchemaWithNestedObjectsFoo::from_str(json2).unwrap();
    assert_eq!(&value2.to_str().unwrap(), json2);
    assert_eq!(value2.bar.baz.message, "Hello, nested object 2!");

    let json3 = "{\"baz\":{\"message\":\"Hello, nested object 3!\"}}";
    let value3 = SchemaWithNestedObjectsFooBar::from_str(json3).unwrap();
    assert_eq!(&value3.to_str().unwrap(), json3);
    assert_eq!(value3.baz.message, "Hello, nested object 3!");

    let json4 = "{\"message\":\"Hello, nested object 4!\"}";
    let value4 = SchemaWithNestedObjectsFooBarBaz::from_str(json4).unwrap();
    assert_eq!(&value4.to_str().unwrap(), json4);
    assert_eq!(value4.message, "Hello, nested object 4!");
}
