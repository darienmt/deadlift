pub async fn get_or_create_object_store(
    js: &async_nats::jetstream::Context,
    bucket_name: &str,
) -> anyhow::Result<async_nats::jetstream::object_store::ObjectStore> {
    match js.get_object_store(bucket_name).await {
        Ok(store) => Ok(store),
        Err(e) => {
            if e.kind() == async_nats::jetstream::context::ObjectStoreErrorKind::GetStore {
                js.create_object_store(async_nats::jetstream::object_store::Config {
                    bucket: bucket_name.to_string(),
                    num_replicas: 1, // why does this fix the permissions issue
                    ..Default::default()
                })
                .await
                .map_err(anyhow::Error::from)
            } else {
                Err(anyhow::Error::from(e))
            }
        }
    }
}

// TODO-- fix being able to upload the same object under the same name multiple times
