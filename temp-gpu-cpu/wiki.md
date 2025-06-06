### CPU/GPU temperature-monitoring daemon written in Rust

#### :one: Prerequisites

Rust programming language ([Official installation guide (using rustup)](https://www.rust-lang.org/tools/install))

#### :two: Build & Install:
    # supposing, that you cloned this repo
    $ cd macos-in-sketchy-aerospace/temp-gpu-cpu

    # this builds the binary from source code (found in src/)
    $ cargo build --release
    
    # (optionally) this minimizes the binary by removing redundunt metadata/debug symbols
    $ strip target/release/temp-gpu-cpu

    # this moves a binary to a directory of your choice
    $ mv target/release/temp-gpu-cpu /$HOME/.config/sketchybar/helpers/temp-gpu-cpu/bin/

    # this turns it into executable 
    $ chmod +x /$HOME/.config/sketchybar/helpers/temp-gpu-cpu/bin/

    # this modifies .aerospace.toml config to execute this binary after startup (before sketchybar)
    $ nvim /$HOME/.aerospace.toml
    ...
    after-startup-command = [
    "exec-and-forget /Users/<your_user>/<path_to_directory_where_you_moved_binary_to>/temp-gpu-cpu", 
    "exec-and-forget sketchybar"]

#### :three: How this daemon works

I have tried to comment the source code in a very detailed way, so you can find comments for almost every function, every construct ecc.

*NB! This is my first experience with Rust, so I am sure the code can be improved. It’s up to you - I hope my comments can help you if you’d like to work on it.*

I am not a Rust dev; however, it’s not arabic to me, as I have quite of experience with Linux systems and low-level stuff like sensor data.

Of course I did not write this code from scratch by myself. I have searched a lot on GitHub and various forums for existing CPU temperature tools on Apple Silicon chips, however, I found out that for the latest M4 chips, CPU/GPU temperature data is a mystery. I found this amazing article that contains many answers and explanations about how to get this data and where to start: [How to get macOS power metrics with Rust? by vladkens](https://medium.com/@vladkens/how-to-get-macos-power-metrics-with-rust-d42b0ad53967). Plus, the author of this article actually developed a CLI tool that performs performance monitoring for Apple Silicon chips (not only M4). I provided a link to it on the main page of the repo.

So, here are some bullet points about this Rust daemon so it may remove your worries that it contains something sketchy or that it brute forcefully amends something on your system:

- no `sudo` required: this means it does not perform any write operations and does not rewrite anything in your system kernel.

- the algorithm does not retrieve the temperature sensors' data in some intrusive way. There are two core languages of macOS: Objective C and Swift. As an owner of MacBook, you are aware that there are many apps and tools running on macOS; some are native and some are 3rd party software. That means there are developers who create them, and if the applications are about using some hardware, of course devs get access to MacBook hardware and drivers. There is [official documentation for Apple developrs on how to access hardware devices and drivers from apps and services](https://developer.apple.com/documentation/iokit). 

So technically, using this documentation and knowing Objective-C, it would not be a big deal to retrieve sensor data, because sensor temperatures are hardware, and of course it is possible to retrieve data from them.

- macOS exposes hardware via IOKit *framework*. This framework implements nonkernel access to IOKit objects such drivers and nubs through the device-interface mechanism. There are different IOServices that can be queryied from *user space* via [*IOKit functions*](https://developer.apple.com/documentation/iokit/iokit_functions). 

Actually, you do not need Objective C skills to start exploring IOKit framework and what it can expose you about your system. This can be done with the `ioreg` command:

```
$ ioreg
ioreg displays the I/O Kit registry. It shows the hierarchical registry structure as an inverted tree.
```

- there is such a thing as the System Management Controller (SMC), which is a microcontroller used to monitor and manage CPU's processes. This SMC has an endpoint (*IOService*) that is meant to expose some metrics of temperature sensors that are located on the CPU and GPU. You can explore it with `ioreg -c AppleSMCKeysEndpoint -c SMCEndpoint1 -l`. Data extraction from this endpoint works on some sort of subscrition basis: a connection should be opened, request must me sent and this IOService will push requested data in response. 
  
- it is not Rust itself that directly communicates with IOKit and establishes this connection; core components of this Rust daemon are actually IOKit functions that are actually just Rust wrappers around Objective C functions. The main complication is not that you simply have to execute these functions and fetch responses, but that as a response, these functions do not provide a nice data in a human readable way. Objective C responds with data structures it understands, and Rust needs an external Rust crate [*core-foundation*](https://docs.rs/core-foundation/latest/core_foundation/) to make reuqests, translate responses and then transform them into a human readable format.
