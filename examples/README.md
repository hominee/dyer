The folder [`examples`] holds a few examples that show you how to use [`dyer`] integrated with [`dyer-cli`], and of course there are many explanations inside each example.

## How to Use

Make sure that [`rust`] and [`cargo`] are installed in your OS, then clone this repository with:
```bash
git clone --depth=1 https://github.com/HomelyGuy/dyer.git
```
One more thing that this crate requires a safe popular cryptography library `OpenSSL`, some additional stuff is needed to compile. in general,
```bash
# for debian-base OS
sudo apt install pkg-config libssl-dev

# for Arch 
sudo pacman -S pkg-config openssl

# for MacOS
brew install openssl
```
Things is a little complicated for Windows users, but it's okay if you follow the steps:
- [OpenSSL] installation:
  you can install [3rd-party binary OpenSSL]  (recommended) or compile it from [source]. If you install `git-bash`, then OpenSSL is already installed, anyway, open your prompt or terminal, type
	```bash
	openssl
	```

- Set up `OPENSSL_DIR`
  download the [openssl-dev] file and unzip it, you see the files:
	```bash
	|__ssl/
	|__x64/
	|__x86
	```

	then export the directory as `OPENSSL_DIR` based on your system. In `Start` -> `This Computer` -> `Property`(right click) -> `Advanced System Setting` -> `Advanced` -> `Environment Variables` -> `New`, and type
	|Variable| Value |
	| --- | --- |
	| OPENSSL_DIR| `path/to/x64`(for 64-bit system) |
	| OPENSSL_DIR| `path/to/x86`(for 32-bit system) |

	then restart the terminal/prompt, and you are ready to go.

[3rd-party binary OpenSSL]: https://wiki.openssl.org/index.php/Binaries
[OpenSSL]: https://www.openssl.org/
[source]: https://github.com/openssl/openssl/
[openssl-dev]: https://mirror.firedaemon.com/OpenSSL/openssl-1.1.1k.zip

Run those examples with:
```
cd dyer\examples\simple-demo 
dyer-cli c 
dyer-cli run(or cargo run ) 
```	

Note that the folder [`template`], as the name suggests, is a template [`dyer-cli`] created for illustration.


[`examples`]: https://github.com/HomelyGuy/dyer/tree/master/examples
[`dyer`]: https://github.com/HomelyGuy/dyer/
[`dyer-cli`]:  https://github.com/HomelyGuy/dyer-cli/
[`rust`]: https://www.rust-lang.org
[`cargo`]: https://doc.rust-lang.org/cargo/
[`template`]:  https://github.com/HomelyGuy/dyer/tree/master/examples/template
