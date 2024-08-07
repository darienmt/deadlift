# Deadlift

Integration solution that utilizes serverless execution of WASM modules to facilitate workflows.

Allows for a repeatable implementation process of integrations. Deploy an integration
once and easily repeat the process across various services and languages without having to
write new code or allocate additional services.

Use Deadlift to build integration modules once and automatically zerosync data anywhere.
Developers can write and reuse high-level code to describe different pieces of a workflow
without having to use a specific language or service.

## How it works

Deadlift automatically compiles integration code to WebAssembly and embeds a portable
execution engine inside any application, enabling workflows to safely and automatically run
within any environment. Utilizing NATS enables real-time messaging of
data between applications and systems of record.

### Web Assembly

WASM modules are executed such that they can be embedded within any environment in a secure and performant manner.

### NATS

NATS messaging is used to execute modules asynchronously with builtin retry mechanisms, in real time and with extremely high throughput.

## Our thesis

Integrations should be:

- platform independent -> able to run anywhere (cloud, on prem, edge, etc)
- embeddable -> able to run within existing services (doesn't require new containers or services)
- reusable -> able to utilize existing integration modules for new workflows (also enables out of the box solutions)

## Quickstart

Install the [rust toolchain](https://www.rust-lang.org/tools/install)

### Running the web service

```
cargo run -p deadlift-service --bin deadlift-service
```

### Creating a module

[extism](https://github.com/extism/extism)

[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
