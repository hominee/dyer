# [dyer](https://github.com/hominee/dyer)

[![API Document](https://img.shields.io/docsrs/dyer/latest)](https://docs.rs/dyer)
[![crates.io](https://img.shields.io/crates/v/dyer.svg)](https://crates.io/crates/dyer)
[![Cookbook](https://img.shields.io/static/v1?label=cookbook&message=dyer&color=green)](https://hominee.github.io/dyer/)

## special thinks
[![Powered by DartNode](https://dartnode.com/branding/DN-Open-Source-sm.png)](https://dartnode.com "Powered by DartNode - Free VPS for Open Source")

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

- `xpath-stable`: stably parse the response html with third-party C library `libxml2`   
- `xpath-alpha`: rust-native parse the html response, **NOTE** that it is experimental and unstale, remains to be improved  
- `compression`: Enable HTTP Compression: `br`, `deflate`, `gzip`
- `proxy`: Enable use proxies
- `full`: Enable all features

## Guide

**Get started** by installing [dyer-cli] and looking over the [examples] and [quick start].

Crates: [Link 🔗](https://crates.io/crates/dyer/)   
Documentation: [Link 🔗](https://docs.rs/dyer/latest/dyer)   
The [Cookbook](https://hominee.github.io/dyer/) gives a detailed view of dyer.

[dyer]: https://docs.rs/dyer
[examples]: https://github.com/hominee/dyer/tree/master/examples/
[quick start]: https://github.com/hominee/dyer/tree/master/quick-start.md/
[dyer-cli]: https://github.com/hominee/dyer-cli

