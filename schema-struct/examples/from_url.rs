use schema_struct::schema_struct;

fn main() {
    schema_struct!(url = "https://raw.githubusercontent.com/WKHAllen/schema-struct/main/schema-struct/tests/schemas/product-url.json");

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
