# deadlift CLI

The [deadlift](https://github.com/zerosync-co/deadlift) CLI can be used to manage deadlift modules and workflows.

## Installation

### Docker

```
docker run --rm -it adunne09/deadlift-cli:latest deadlift <args>
```

### From Source

* requires [rust](https://www.rust-lang.org/tools/install) 1.80.0

```
git clone https://github.com/zerosync-co/deadlift.git
cd <into deadlift dir>
cargo install --path cli
```

## Usage example (Rust)

1. Start a local development [NATS server](https://docs.nats.io/running-a-nats-service/introduction/installation) with jetstream enabled

2. Generate a module source project

```
deadlift module generate <module name>
```

3. Write your desired module functionality

* A module is a standard function that can receive input and return output

* [docs](https://extism.org/docs/quickstart/plugin-quickstart)

Rust example:
```
use extism_pdk::*;

#[plugin_fn]
pub fn add_one(input: i32) -> FnResult<i32> {
  input + 1
}
```

4. Compile your module

```
cd <into module dir>
cargo build --release
```

* requires the `wasm32-wasi` target to be installed, which can be installed with:

```
rustup target add wasm32-wasi
```

5. Publish your module

```
deadlift module publish <module name> --path <wasm file path>
```

* the wasm file path will be located at `<module dir>/target/wasm32-wasi/release/<project name>.wasm`

6. Repeat this process for other desired workflow modules

7. Create a workflow

```
deadlift workflow generate <workflow name>
```

8. Specify your desired workflow nodes and edges per the template in `<workflow name>/workflow.yaml`

Calculator Example:
```
name: "do some math"
nodes:
  - object_name: "add one"
    plugin_function_name: add_one
  - object_name: "mulitply by five"
    plugin_function_name: multiply_by_five
node_holes: []
edge_property: directed
edges:
  - - 0
    - 1
    - null
```

9. Publish your workflow

```
deadlift workflow publish <workflow YAML file path>
```

* the workflow file path will be located at `<workflow dir>/workflow.yml`

10. Call your workflow with input

```
deadlift workflow call <workflow name> --input <your expected workflow input>
```
