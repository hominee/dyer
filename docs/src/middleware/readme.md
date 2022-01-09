# Middleware

Middleware hooks all requests/responses and their derivatives of dyer, including `Task`, `Affix`, `Request`, `Response`, `error` and `entiry`. it's flexible, low-level, scale to modify the data flow of dyer.

### Inspection of Middleware

before we dive deeper into what middleware is, let take a look at some simplified code of `Middleware`
```rust no_run
pub struct MiddleWare<'md, E> {
    handle_affix:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Affix>, &'a mut App<E>)>,
    handle_task:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Task>, &'a mut App<E>)>,
    handle_req:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Request>, &'a mut App<E>)>,
    handle_res:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Response>, &'a mut App<E>)>,
    handle_entity:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<E>, &'a mut App<E>)>,
    handle_yerr: Option<
        &'md dyn for<'a> Fn(
            &'a mut Vec<Result<Response, MetaResponse>>,
            &'a mut App<E>,
        )>,
    handle_err: Option<
        &'md dyn for<'a> Fn(
            &'a mut Vec<Result<Response, MetaResponse>>,
            &'a mut App<E>,
        )>,
		// some other fields
		...
}
```

As shown above, it accepts some nullable async function as handlers for requests, response and its derivatives.
let's log out errors:
```rust no_run 
pub async fn log_err(errs: &mut Vec<Result<Response, MetaResponse>, _: &mut App<E>> {
	for r in errs.iter() {
		match r {
			Ok(data) => {
				println!("failed request to {}", data.metas.info.uri);
			},
			Err(e) => {
				println!("failed request to {}", e.info.uri);
			}
		}
	}
}

// set up `handle_err` 
let middleware = MiddleWare::builder().err_mut(&log_err).build("marker".into());
```
that middleware will log out uri of failed response.  

