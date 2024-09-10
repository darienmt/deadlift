# deadlift CLI

The [deadlift](https://github.com/zerosync-co/deadlift) CLI can be used to manage deadlift modules and workflows.

## Installation

### From source

* requires [rust](https://www.rust-lang.org/tools/install) 1.80.0

```
git clone https://github.com/zerosync-co/deadlift.git
cd <into deadlift dir>
cargo install --path cli
```

## Usage

1. Start a local development [NATS server](https://docs.nats.io/running-a-nats-service/introduction/installation) with jetstream enabled

2. Generate a plugin source project

```
deadlift module generate <module name> --lang <your preferred language, default rust>
```

3. Publish your plugin

```
deadlift module publish <module name> --path <wasm file path>
```

4. Create a workflow

```
deadlift workflow generate <workflow name>
```

5. Publish your workflow

```
deadlift workflow publish <workflow YAML file path>
```

6. Call your workflow with input

```
deadlift workflow call <workflow name> --input <input string>
```
