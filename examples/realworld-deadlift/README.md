# RealWorld Deadlift

An example fullstack application that utilizes the fetch API with WASM

[credit](https://github.com/sveltejs/realworld)

### Run the example

1. Run the web service

```
cargo run -p deadlift-service --bin deadlift-service
```

2. Run the queue worker

```
cargo run -p deadlift-service --bin queue-worker
```

3. Start a local NATS server

[install](https://formulae.brew.sh/formula/nats-server#default)

4. In another terminal window, navigate to the realworld-deadlift example dir, seed the db, and run the app

```
cd examples/realworld-example
npm i
npm run seed
npm run dev
```

#### Note

Running `npm run seed` will attempt to compile the WASM modules, which uses the following items:

- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
- wasm32-unknown-unknown target `rustup target add wasm32-unknown-unknown`
- wasm32-wasi target `rustup target add wasm32-wasi`

5. Create the workflow

```
curl 'http://localhost:8080/api/v1/workflows' \
  -H 'Content-Type: application/json' \
  --data-raw '{"name":"realworld workflow","pipeline":["deadlift.modules.ingest.realworld-fetch","deadlift.modules.default.realworld-fetch-filter","deadlift.modules.deliver.realworld-stdout-notification"]}'
```

6. Open your browser and navigate to http://localhost:5173/

7. Create an account or sign in

8. Create a blog post

9. See a notification that the post was created in the queue-worker logs!
