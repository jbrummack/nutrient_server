# Nutrient Server
## Simplified nutrient API
```rust
struct FoodResponseV0 {
    id: String,
    nutrients: Vec<String>,
    urls: Vec<String>,
}
struct FoodResponseV1 {
    productname: String,
    id: String,
    ingredients: String,
    brands: String,
    categories: String,
    quantity: String,
    nutrients: Vec<String>,
    urls: Vec<String>,
}
```
nutrients are encoded as ```label:amount```

## Performance
this should run really fast, everything is hardcoded in memory and read only (no locking required)
