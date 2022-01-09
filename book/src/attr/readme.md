# Attribute

You may notice that some components are annotated with something like `#[dyer::entity]`, `#[dyer::actor]` or others, they are attribute Macros what transforms a block of code into another code block.
All of availiable attributes are following.

- `#[dyer::affix]` mark the type annotated for `Affix` 
- `#[dyer::actor]` mark the type annotated for `Actor` 
- `#[dyer::middleware]` mark the type annotated for `Middleware` 
- `#[dyer::pipeline]` mark the type annotated for `Pipeline` 
- `#[dyer::parser]` mark the type annotated for `parser`, any function with this attribute can parse response.  
- `#[dyer::entity]` mark the type annotated for `entity`, any type with this attribute can contain data to be collected. 
- `#[dyer::async_trait]` mark the type annotated for `async_trait`, note that it is a wrapper of crate [async_trait](https://crates.io/crates/async-trait) 
