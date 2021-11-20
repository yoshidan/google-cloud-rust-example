# Spanner Example
* Directory [rust](./rust) is Rust example
* Directory [go](./go) is GoLang example for comparison.
* Directory [scenario](./scenario) is just load test scenario.

## Run on Local 
```
docker-compose run init     # initialize local spanner emulator
docker-compose run create   # create database and tables
docker-compose up rust-api
```

```
curl -X POST localhost:3031/CreateUser
```

## Run on GKE

### Setup your GCP project 
* [Create spanner database](https://console.cloud.google.com/spanner) and create tables these [schema.sql](./ddl/schema.sql)
* Create [GKE](https://cloud.google.com/kubernetes-engine?hl=ja) cluster.

### Deploy to GKE
```
# change <your_project> to your GCP project id
YOUR_PROJECT=<your_project>

cd rust
docker build -t asia.gcr.io/${YOUR_PROJECT}/rust-api:latest .
docker push asia.gcr.io/${YOUR_PROJECT}/rust-api:latest
sed -e "s/<your_project>/${YOUR_PROJECT}/" k8s.tmpl.yaml > k8s.yaml
kubectl apply -f k8s.yaml

cd scenario
docker build -t asia.gcr.io/${YOUR_PROJECT}/loadtest:latest .
docker push asia.gcr.io/${YOUR_PROJECT}/loadtest:latest
sed -e "s/<your_project>/${YOUR_PROJECT}/" k8s.tmpl.yaml > k8s.yaml
kubectl apply -f k8s.yaml
