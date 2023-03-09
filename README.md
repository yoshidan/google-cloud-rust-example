# google-cloud-rust-example

```
docker-compose up pubsub spanner
docker-compose run spanner-init
docker-compose run spanner-create
```

## WebAPI
```
SPANNER_EMULATOR_HOST=localhost:9010 DATABASE=projects/local-project/instances/test-instance/databases/local-database cargo run --bin google-cloud-example-webapi

# Create user and get inventory.
user_id=`curl -X POST localhost:8100/api/user`
curl localhost:8100/api/user/$user_id/inventory
```

## Chat
```
PUBSUB_EMULATOR_HOST=localhost:8681 cargo run --bin google-cloud-example-chat

# userA
wscat -c "ws://127.0.0.1:8091/Connect?channelId=ch01&userId=userA"
Connected (press CTRL+C to quit)
> 

# userB
wscat -c "ws://127.0.0.1:8091/Connect?channelId=ch01&userId=userB"
Connected (press CTRL+C to quit)
> 

```
