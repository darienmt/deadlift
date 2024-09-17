# deadlift CLI

The [deadlift](https://github.com/zerosync-co/deadlift) CLI can be used to manage deadlift modules and workflows.

## Installation

### From Source

* requires [rust](https://www.rust-lang.org/tools/install) 1.80.0

```
git clone https://github.com/zerosync-co/deadlift.git
cd <into deadlift dir>
NATS_BEARER_JWT=eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiIzWU1GTTdTNUFTN1JUTlM1SkRaUUpWU0M2NlFFUjdHRkJRREw2NFZTR0VBU0xaUE9OUzJRIiwiaWF0IjoxNzI1MjI4MzQ4LCJpc3MiOiJBQk01R0dVVzUyTDUzSlRJNjMzTEdIUEJRSVJRNkk3MlhBQTdXNVJDNEhGUkszSldCQjRUWlZLQyIsIm5hbWUiOiJkZWFkbGlmdC1jbGktcmVzdHJpY3RlZC11c2VyIiwic3ViIjoiVUNTTERVWkdMNUc2NERETjJURlc0RUdVUFpLT1RSNlZUUlFCNTRNWFlFRlNaRE5XNzdIUDY1R0ciLCJuYXRzIjp7InB1YiI6eyJhbGxvdyI6WyJkZWFkbGlmdC5leGVjdXRpb25zLmNsaV9jcmVhdGVfdXNlciJdfSwic3ViIjp7ImFsbG93IjpbImRlYWRsaWZ0LmV4ZWN1dGlvbnMuY2xpX2NyZWF0ZV91c2VyLnJlcGx5Il19LCJzdWJzIjotMSwiZGF0YSI6LTEsInBheWxvYWQiOi0xLCJiZWFyZXJfdG9rZW4iOnRydWUsInR5cGUiOiJ1c2VyIiwidmVyc2lvbiI6Mn19.5PWdzeLr2oMpxD6HFw6qpcqUJPMy-NvVoWSZtdZq3cRTvZS9-Oz4kkCoIxwQd0i4uwvkvSSGH-QYwySEcMbdCg \
NATS_URL=97.118.226.37:4223 \
cargo install --path cli
```

## Usage example (Rust)

1. Create a user

```
deadlift user create --email <your email> --password <your password>
```

2. Generate a deadlift source project

```
deadlift project generate <project name>
```

3. Write your desired modules and workflow in `workflow.yml`

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

* A workflow is a graph that describes desired modules and an execution tree to facilitate integration needs

Calculator Workflow Example:
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

4. Publish your project

```
deadlift project publish
```

* requires the `wasm32-wasi` target to be installed, which can be installed with:

```
rustup target add wasm32-wasi
```

5. Call your workflow with input

```
deadlift call --fn-name <wasm function name> --input <wasm function input>
```
