# [dyer](https://github.com/homelyguy/dyer)

[![API Document](https://img.shields.io/docsrs/dyer/latest)](https://docs.rs/dyer)
[![crates.io](https://img.shields.io/crates/v/dyer.svg)](https://crates.io/crates/dyer)
[![Cookbook](https://img.shields.io/static/v1?label=cookbook&message=dyer&color=green)](https://homelyguy.github.io/dyer/)

## Overview

[dyer] is designed for reliable, flexible and fast Request-Response based service, including data processing, web-crawling and so on, providing some friendly,  interoperable, comprehensive  features without compromising speed.

## Features

* asynchronous, concurrent streaming and I/O, make the best of thread pool, network, and system
resource.
* Event-driven, once you set the initials and recursive generator, `dyer` will handle
the rest of it interoperably.
* User-friendly and flexible, `dyer` offers high-level, flexible, easy to use wrappers and APIs what does a lot for you.    

## Feature Flag
To reduce code redundancy and speed up compilation, dyer use feature flag to mark the necessary modules/functions, Currently here are some supported Features:

- `xpath`: Enable parse the html response with xpath 
- `compression`: Enable HTTP Compression: `br`, `deflate`, `gzip`
- `proxy`: Enable use proxies
- `full`: Enable all features

## Guide

**Get started** by installing [dyer-cli] and looking over the [examples] and [quick start].

Crates: [Link 🔗](https://crates.io/crates/dyer/)   
Documentation: [Link 🔗](https://docs.rs/dyer/latest/dyer)   
The [Cookbook](https://homelyguy.github.io/dyer/) gives a detailed view of dyer.

[dyer]: https://docs.rs/dyer
[examples]: https://github.com/HomelyGuy/dyer/tree/master/examples/
[quick start]: https://github.com/HomelyGuy/dyer/tree/master/quick-start.md/
[dyer-cli]: https://github.com/HomelyGuy/dyer-cli

