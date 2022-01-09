# Affix

Affix is the fringerprint when making a request. In general, affix is not necessay unless the target site requires visitor meet some criteria. Affix, by far, mainly focus on modification of Headers.

> assign a user-agent for each Task with file `user-agents.txt` containing user-agents by lines

```rust no_run 
// src/affix.rs
pub struct Aff {
	uas: Vec<String>
	iter: std::iter::Cycle<String>,
}

#[dyer::async_trait]
impl Affixor for Aff {
	// this function only runs once 
	async fn init(&mut self) {
		use std::io::Read;

		let mut file = std::fs::File::open("path/to/user-agents.txt").unwrap();
		let buf = std::io::BufReader::new(file);
		let uas = buf.lines().map(|line| {
			 line.unwrap()
		}).collect::<Vec<String>>();
		self.uas = uas;
		self.iter = self.uas.iter().cycle();
	}

	// if the affix isn't obtained via network(request-response), just return `None` 
	async fn invoke(&mut self) -> Option<dyer::Request> {
			None
	}
	// dyer combine the `Affix` returned by this function to each `Task` before make an request
	async fn parse(&mut self, _: Option<Result<Response, MetaResponse>>) -> Option<dyer::Affix> {
		// return the user-agent in order
		self.iter.next().to_owned()
	}

	// other method of Affixor
	...
}

// src/actor.rs

#[dyer::async_trait]
impl Actor<_, _> for MyActor {
	async fn entry_affix(&mut self) -> Option<Aff> {
			Some(Aff)
	}

	// other method of Actor
	...
}

```

