# broadcaster
Broadcaster is a tool to broadcast JSON-RPC calls on a set of ethereum nodes. 

The main priority of the tool is to send requests to all nodes at the same time, in parallel. 

# Installation from source
To install broadcaster from source first clone the repository:
```
git clone git@github.com:piwonskp/broadcaster.git
```
Then cd into the directory:
```
cd broadcaster
```
Broadcaster may be installed from source by building docker image or using cargo. 

## Docker
1. Install [docker](https://docs.docker.com/engine/install/).
2. Build image:
```
docker build . -t broadcaster:dev
```
## Cargo
1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Run
```cargo install```

# Usage
To use broadcaster run it and specify node addresses as positional arguments:
* Docker: `docker run -p 8545:8545 broadcaster:dev [A list of node addresses separated by space]`
* Hst: `broadcaster [A list of node addresses separated by space]`

Once the broadcaster is started you can send requests to `127.0.0.1:8545`. Requests will be broadcasted to all nodes specified in command line arguments.
