# Actor

Actor is trait that processing some methods to set up necessary conditions before the whole programs starts/ends. it basically categorizes:

- **Preparations Beforehand** method `new`, `open_actor` and `close_actor` are provided to serve that purpose. First of all, you can define a struct according to your requirement, eg:
```rust no_run
#[dyer::actor]
struct MyActor {
	start_uris: Vec<String>
	...
}
```
the struct `MyActor` should contain appropirate fields which initialized by the method `new`. 
```rust no_run
#[dyer::async_trait]
impl Actor<_, _> for MyActor {
	async fn new() -> Self {
		Self {
			start_uris: vec!["https://example.domain/path/to/site1".to_string(),
									"https://example.domain/path/to/site2".to_string() ] 
			//other fields 
			...
		}
	}
	// other method of Actor
	...
}
```
before the whole program starts, the method `open_actor` gets called. preparation should be done here! but wait, what should we do here? let's extend the example above a little bit.

> all start uris are stored by lines in a file `uris.txt`
```rust no_run
#[dyer::actor]
pub struct MyActor {
	start_uris: Vec<String>
}

#[dyer::async_trait]
impl Actor<_, _> for MyActor {
	async fn new() -> Self {
		use std::io::Read;

		let mut file = std::fs::File::open("path/to/uris.txt").unwrap();
		let buf = std::io::BufReader::new(file);
		let uris = buf.lines().map(|line| {
			 line.unwrap()
		}).collect::<Vec<String>>();
		Self {
			start_uris: uris
		}
	}

	async fn open_actor(&mut self, _app: &mut App<_>) {
		self.start_uris.for_each(|uri| {
			Task::get(uri)
				.parser(..)
				.body(Body::empty(), "myactor_identifier".into())
				.unwrap()
		});
	}
	// other method of Actor
	...
}
```
Analogously you can do some staff with `close_actor` when program ends.

- **Assignments Entry** The program cannot starts without `Task`, `entry_task` serve as a way to add tasks to the lists. It expects a vector of `Task` when the function ends,    
```rust no_run
#[dyer::async_trait]
impl Actor<_, _> for MyActor {
	async fn entry_task(&mut self) -> Result<Vec<Task>, Box<dyn Error>> {
		self.start_uris.map(|uri| {
			Task::get(uri)
				.parser(..)
				.body(Body::empty(), "myactor_identifier".into())
				.unwrap()
		}).collect::<_>()
	}
	// other method of Actor
	...
}
```
As for `entry_affix`, it is commonly not necessary unless modification is required for that `Task`, 
But what is that? before we answer that let's take a look at the structure of [Task], 
```rust no_run
pub struct Task {
    /// main infomation that represents a `Task`
    pub(crate) inner: InnerTask,
    /// Formdata, files or other request parameters stored here
    pub(crate) body: Body,
		...
}
pub struct InnerTask {
    pub uri: Uri,
    /// request's vesoin
    pub method: Method,
    /// additional headers if necessary
    pub headers: HeaderMap<HeaderValue>,
		...
}
```
it is obvious to see that a `Task` almost contains infomation to make a request.

But when does `entry_affix` play its role? Here are some scenarios that you may use it.

1. Headers Modification (eg. Cookies, User-Agents, Tokens, and etc.)
2. javascript simulation
3. FFI and others

Here we focus on the first one(most used) and an example is given at section [Actor].

[Task]: https://docs.rs/dyer/latest/dyer/struct.Task.html 
[Actor]: ../actor/readme.md
