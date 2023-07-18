use schema_struct::schema_struct;

fn main() {
    schema_struct!(
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

    let product_json = r#"
        {
            "id": 5,
            "name": "product name",
            "price": 12.34
        }
    "#;
    let product = ProductSchema::from_str(product_json).unwrap();

    assert_eq!(product.id, 5);
    assert_eq!(product.name, "product name".to_owned());
    assert_eq!(product.price, 12.34);

    dbg!(product);
}
