use schema_struct::schema_struct;

fn main() {
    schema_struct!(
        validate = true,
        schema = {
            "$schema": "http://json-schema.org/draft-04/schema#",
            "title": "Draft04",
            "description": "Core schema meta-schema",
            "$defs": {
                "schemaArray": {
                    "type": "array",
                    "minItems": 1,
                    "items": { "$ref": "#" }
                },
                "positiveInteger": {
                    "type": "integer",
                    "minimum": 0
                },
                "simpleTypes": {
                    "enum": [
                        "array",
                        "boolean",
                        "integer",
                        "null",
                        "number",
                        "object",
                        "string"
                    ]
                },
                "stringArray": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "uniqueItems": true
                }
            },
            "type": "object",
            "properties": {
                "id": {
                    "type": "string"
                },
                "$schema": {
                    "type": "string"
                },
                "title": {
                    "type": "string"
                },
                "description": {
                    "type": "string"
                },
                "multipleOf": {
                    "type": "number",
                    "minimum": 0,
                    "exclusiveMinimum": true
                },
                "maximum": {
                    "type": "number"
                },
                "exclusiveMaximum": {
                    "type": "boolean",
                    "default": false
                },
                "minimum": {
                    "type": "number"
                },
                "exclusiveMinimum": {
                    "type": "boolean",
                    "default": false
                },
                "maxLength": { "$ref": "#/$defs/positiveInteger" },
                "minLength": { "$ref": "#/$defs/positiveInteger" },
                "pattern": {
                    "type": "string",
                    "format": "regex"
                },
                "maxItems": { "$ref": "#/$defs/positiveInteger" },
                "minItems": { "$ref": "#/$defs/positiveInteger" },
                "uniqueItems": {
                    "type": "boolean",
                    "default": false
                },
                "maxProperties": { "$ref": "#/$defs/positiveInteger" },
                "minProperties": { "$ref": "#/$defs/positiveInteger" },
                "required": { "$ref": "#/$defs/stringArray" },
                "$defs": {
                    "type": "object",
                    "additionalProperties": { "$ref": "#" },
                    "default": {}
                },
                "properties": {
                    "type": "object",
                    "additionalProperties": { "$ref": "#" },
                    "default": {}
                },
                "patternProperties": {
                    "type": "object",
                    "additionalProperties": { "$ref": "#" },
                    "default": {}
                },
                "enum": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "uniqueItems": true
                },
                "type": { "$ref": "#/$defs/simpleTypes" },
                "format": { "type": "string" },
                "allOf": { "$ref": "#/$defs/schemaArray" },
                "anyOf": { "$ref": "#/$defs/schemaArray" },
                "oneOf": { "$ref": "#/$defs/schemaArray" },
                "not": { "$ref": "#" }
            }
        }
    );

    let schema_json = r#"
        {
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
    "#;
    let schema = Draft04::from_str(schema_json).unwrap();

    assert_eq!(schema.title, Some("ProductSchema".to_owned()));
    assert_eq!(
        schema.description,
        Some("A product from Acme's catalog".to_owned())
    );
    assert_eq!(schema.type_, Some(Box::new(Draft04DefSimpleTypes::Object)));
    assert_eq!(
        schema.required,
        Some(Box::new(vec![
            "id".to_owned(),
            "name".to_owned(),
            "price".to_owned()
        ]))
    );

    dbg!(schema);
}
