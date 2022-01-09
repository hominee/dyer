# Pipeline & Database Intergration

the end of data flow, it will be consumed. 
When an entity has been collected, it eventually will be sent to pipelines.
Pipeline provides way to do:
- cleaning/validating collected entity 
- de-duplicates 
- database storing

### Inspection of Pipeline

Let's take a look at the simplified code of `Pipeline` before diving deeper.
```rust no_run
pub struct PipeLine<'pl, E, C> { 
    initializer: Option<&'pl dyn for<'a> Fn(&'a mut App<E>) -> Option<C>>,
 
    disposer: Option<&'pl dyn for<'a> Fn(&'a mut App<E>)>,          
 
    process_entity: 
        Option<&'pl dyn for<'a> Fn(Vec<E>, &'a mut App<E>)>, 
 
    process_yerr: Option< 
        &'pl dyn for<'a> Fn( 
            Vec<Result<Response, MetaResponse>>, 
            &'a mut App<E>, 
        )>,
		// other fields omitted
		...
}
```
- the method `initializer` get called only once over the runtime, it returns a generic type `C` which defined by user, the generic type is usually a connection cursor to storage destination. 
- the method `disposer` get called once when the pipeline ends. 
- the method `process_entity` processes a vector of entity then consume them.
- the method `process_yerr` processes a vector of failed response then consume them.

### Diesel Sql

[Diesel] is the most productive way to interact with SQL databases. It is recommanded to get around the basics of diesel [ here ]( https://diesel.rs/guides/getting-started ).
A detailed example is given at [examples](https://github.com/HomelyGuy/dyer/tree/master/examples/dyer-diesel).


[Diesel]: https://diesel.rs

### Other Database

Almost other databases are equipmented with rust-based driver, it is just as simple as following the documentation, implementing the necessary methods.     

Here is an simple example for MongoDB Intergration with driver [mongodb](https://crates.io/crates/mongodb).
``` rust no_run 
pub async fn establish_connection(_app: &mut App<_>) -> Option<&'static mongodb::Client> {
		static INIT: Once = Once::new();
    static mut VAL: Option<mongodb::Client> = None;
    unsafe {                        
        let uri = "mongodb://127.0.0.1:27017";
        INIT.call_once(|| {
            VAL = Some(mongodb::Client::with_uri_str(uri).await.unwrap());  
        });                
        VAL.as_ref()                                                                    
    }
}

pub async fn store_item(ens: Vec<_>, _app: &mut App<_>) {
	// do stuff here like validating and dropping 
	...
	let client = establish_connection(_app).await;
	client.database("database_name_here")
		.collection("collection_name_here")
		.insert_one(...)
		.await
		.unwrap();
}

// set up pipiline 
let pipeline = Pipeline::builder()
	.initializer(establish_connection)
	.entity_mut(store_item)
	.build("marker".into());
```
This pipeline will insert collected entity into MongoDB.
