[dyer] is designed for reliable, flexible and fast web-crawling, providing some high-level, comprehensive  features without compromising speed.

By means of event-driven, non-blocking I/O [tokio] and powerful, reliable [Rust programming
language], inspired by [scrapy], `dyer` provides some high-level features:  

* asynchronous, concurrent streamimg and I/O, make the best of thread pool, network, and system
resource.
* event-driven, once you set the initials and recursive generator of [Task], `dyer` will handle
the rest of it.
* user-friendly, considering the philosophy of rust programming language, more source code,
proper architecture may set up yourself in a dilemma when efficiency and learning cost are taken
into consideration. `dyer` offers high-level,flexible wrappers and APIs what does a lot for you.    

**Get started** by installing [dyer-cli] and looking over the [examples].

[dyer]: https://docs.rs/dyer
[tokio]: https://docs.rs/tokio
[scrapy]: https://scrapy.org
[Rust programming language]: https://www.rust-lang.org
[examples]: https://github.com/HomelyGuy/dyer/tree/master/examples/
[dyer-cli]: https://github.com/HomelyGuy/dyer-cli

# Main Functionalities

`Dyer` is newly developed rust library, and has achieved some basic functionalities for
building a crawer, web service and data processing. Nevertheless, it can tackle most common problems you meet.

## Real-browser Customization

It is disabled by default, but As you wish you can enable it by specifying [`ArgProfile`]. In general, for each feeded [Task], `dyer` will fake a [Profile] and combines them into a [Request] to
meet the requirement of the target site. By means of [ `ffi` ] interface of and web
assemble of rust, combination with javascript or python script may do you a favor hopefully. 

## Signal Handling

Think about a scenario that errors, bugs and unexpected accidents are found when your app is running, what would you
do? Stop the app, the entire program and the data are corupted. Nope, the result is not
reliable. `dyer` will backup your history between certain gap, resumption is at your choice.

## Run-time Control

Each [Task] and each [Profile] is scheduled with some gap, has a time stamp for validation,
only the expired can be feeded to engine of `dyer`. Nevertheless `dyer` will limit the
[Request] sent to poll, the [Profile] to make, [Task] to load or store and so on [see `ArgApp`].

[see `ArgApp`]: https://docs.rs/dyer/1.1.1/dyer/engine/struct.ArgApp.html
[Task]: https://docs.rs/dyer/1.1.1/dyer/component/task/struct.Task.html
[Profile]: https://docs.rs/dyer/1.1.1/dyer/component/profile/struct.Profile.html
[`ArgProfile`]: https://docs.rs/dyer/1.1.1/dyer/engine/arg/struct.ArgProfile.html
[Request]: https://docs.rs/dyer/1.1.1/dyer/component/request/struct.Request.html
[`ffi`]: https://doc.rust-lang.org/nomicon/ffi.html
