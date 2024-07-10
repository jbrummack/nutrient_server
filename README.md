# Nutrient Server
## Simplified nutrient API
```rust
struct FoodResponseV0 {
    id: String,
    nutrients: Vec<String>,
    urls: Vec<String>,
}
```
nutrients are encoded as ```label:amount```

## Performance
this should run really fast, everything is hardcoded in memory and read only (no locking required)
