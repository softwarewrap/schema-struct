use schema_struct::schema_struct;

fn main() {
    schema_struct!(file = "schema-struct/tests/schemas/product-file.json");

    let product_json = r#"
        {
            "id": 5,
            "name": "product name",
            "price": 12.34
        }
    "#;
    let product = Product::from_str(product_json).unwrap();

    assert_eq!(product.id, 5);
    assert_eq!(product.name, "product name".to_owned());
    assert_eq!(product.price, 12.34);

    dbg!(product);
}
