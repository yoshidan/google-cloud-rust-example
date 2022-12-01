# google-cloud-rust-example

```
docker-compose up -d pubsub spanner
docker-compose run spanner-init
docker-compose run spanner-create
```

## WebAPI
```
cargo run webapi

# Create user and get inventory.
user_id=`curl -X POST localhost:8100/api/user`
curl localhost:8100/api/user/$user_id/inventory
```