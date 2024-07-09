const sqlite3 = require('sqlite3').verbose();
const fs = require('fs');
const crypto = require('crypto');
const { exec } = require('child_process');
const util = require('util');

const execPromise = util.promisify(exec);

main();

async function main() {
	const modules = [
		{
			wasmPath: './wasm-fetch/pkg/wasm_fetch_bg.wasm',
			title: 'Receive blog data',
			description: null,
			subject: 'deadlift.modules.ingest.realworld-fetch',
			compileFn: compileWasmFetch
		},
		{
			wasmPath: './wasm-fetch-filter/target/wasm32-unknown-unknown/release/wasm_fetch_filter.wasm',
			title: 'Filter for article creation',
			description: null,
			subject: 'deadlift.modules.default.realworld-fetch-filter',
			compileFn: compileWasmFetchFilter
		},
		{
			wasmPath:
				'./wasm-stdout-notification/target/wasm32-wasi/release/wasm_stdout_notification.wasm',
			title: 'Send stdout notification',
			description: null,
			subject: 'deadlift.modules.deliver.realworld-stdout-notification',
			compileFn: compileWasmStdoutNotification
		}
	];

	await Promise.all(modules.map((v) => v.compileFn()));

	let db = new sqlite3.Database('../../database.sqlite', (err) => {
		if (err) {
			console.error(err.message);
			return;
		}
		console.log('Connected to the SQLite database.');
	});

	const createTableSql = fs.readFileSync(
		'../../crates/service/migrations/2024-05-29-202154_add-modules/up.sql',
		'utf8'
	);

	db.serialize(async () => {
		db.run(createTableSql, (err) => {
			if (err) {
				console.error(err.message);
			}
		});

		modules.forEach((v) => insertModule(db, v));
	});

	db.close((err) => {
		if (err) {
			console.error(err.message);
		}
		console.log('Closed the database connection.');
	});
}

async function compileWasmFetch() {
	await execPromise('wasm-pack build wasm-fetch --release');
	console.log('successfully compiled wasm-fetch');
}

async function compileWasmFetchFilter() {
	await execPromise(
		'cargo build --manifest-path wasm-fetch-filter/Cargo.toml --release --target wasm32-unknown-unknown'
	);
	console.log('successfully compiled wasm-fetch-filter');
}

async function compileWasmStdoutNotification() {
	await execPromise(
		'cargo build --manifest-path wasm-stdout-notification/Cargo.toml --release --target wasm32-wasi'
	);
	console.log('successfully compiled wasm-stdout-notification');
}

function insertModule(db, moduleData) {
	const { wasmPath, title, description, subject } = moduleData;
	const binary = fs.readFileSync(wasmPath);
	const hash = crypto.createHash('sha256').update(binary).digest('hex');

	const insertSql = `INSERT INTO modules (hash, binary, title, description, subject) VALUES (?, ?, ?, ?, ?)`;
	db.run(insertSql, [hash, binary, title, description, subject], (err) => {
		if (err) {
			console.error(err.message);
		} else {
			console.log(`Inserted module with hash: ${hash}`);
		}
	});
}
