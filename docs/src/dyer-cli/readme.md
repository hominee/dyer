# Dyer-cli

Before stepping into the topic, [dyer-cli](https://crates.io/crates/dyer-cli) is highly recommanded to be installed.
Dyer-cli is a handy tool for your easy and fast use of dyer, 

### Installation
`dyer-cli` is public crate, just run the following in the terminal,
```bash
cargo install dyer-cli 
```
once installed, type `dyer` in the terminal to check, if something like following it is successfully installed.
``` 
Handy tool for dyer

USAGE:
   dyer [subcommand] [options]
   eg. dyer new myproject --debug 
	 ...
```

### Create Project

Dyer-cli generates a template that contains many useful instances and instructions
 when using dyer with
following code:
```bash
dyer new myproject
```
It will create a project called `myproject` and the files layout displays:
```bash
|___Cargo.toml
|___Readme.md
|___data/
|___data/tasks/
|___src/
    |___src/affix.rs
    |___src/entity.rs
    |___src/parser.rs
    |___src/actor.rs
    |___src/middleware.rs
    |___src/pipeline.rs
```

### Project layout and its role

Main functionality of each file:
* the `affix.rs` serves as an actor to adjust and satisfy additional requirement
* the `entity.rs` contains entities/data structure to be used/collected
* the `parser.rs` contains functions that extract entities from response
* the `actor.rs` contains initial when opening and final things to do when closing
* the `middleware.rs` contains Some middlewares that process data at runtime
* the `pipeline.rs` contains entities manipulation including data-storage, displsying and so on
* the `lib.rs` exports all modules inside the directory, just do nothing here normally
* `Cargo.toml` is the basic configuration of the project
* `README.md` contains some instructions of the project
* `data/` place to store/load files of `App` when load-balancing and backup

### Basic Procedures
Then it is your show time, basically there are simple example items(`function`, `enum`, `struct`)
in each file you can follow. After that check your code
```bash
dyer check
```
if you run it the first time, dyer-cli will download the crates and then check the code.
if some warning happens such as `unused import` or `dead code` the command does a lot for you:
```bash
dyer fix
```
A wraper of `cargo fix`,  if some warning happens such as `unused import` or `dead code` the command does a lot for you. However it won't help if some errors occur, if so, you have to debug the code manually.

Edit `dyer.cfg` file in the root directory

the file contains some configurations of `ArgApp` that will update periodically, for more details see
[dyer.cfg Configuration]

When the program compiles, haha run it:
```bash
dyer run
```
