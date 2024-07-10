import { error } from '@sveltejs/kit';
import runModule from 'deadlift-sdk';

const base = 'https://api.realworld.io/api';

async function send(params) {
	const res = await runModule(
		'wasm-fetch/target/wasm32-wasi/release/wasm_fetch.wasm',
		{ useWasi: true, allowedHosts: ['api.realworld.io'], runInWorker: true },
		{ ...params, url: `${base}/${params.path}` }
	);

	if (res.ok || res.status === 422) {
		return res.body ?? {};
	}

	error(res.status);
}

export function get(path, token) {
	return send({ method: 'GET', path, token });
}

export function del(path, token) {
	return send({ method: 'DELETE', path, token });
}

export function post(path, data, token) {
	return send({ method: 'POST', path, data, token });
}

export function put(path, data, token) {
	return send({ method: 'PUT', path, data, token });
}
