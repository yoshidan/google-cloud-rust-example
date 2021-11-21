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

### Create Spanner Database
* [Create spanner database](https://console.cloud.google.com/spanner) and create tables these [schema.sql](./ddl/schema.sql)

### Create GKE Autopilot Cluster
```
export GOOGLE_APPLICATION_CREDENTIALS=<your_credentials_json_path>
export YOUR_PROJECT=<your_project>
export YOUR_ACCOUNT=<your_account>
export YOUR_CLUSTER=example

# activate service account which have permission to use spanner and gke (owner is easy to setup).
gcloud auth activate-service-account ${YOUR_ACCOUNT}@${YOUR_PROJECT}.iam.gserviceaccount.com --key-file=$GOOGLE_APPLICATION_CREDENTIALS

# create GKE Autopilot cluster
gcloud container clusters create-auto $YOUR_CLUSTER --region asia-northeast1 --project=$YOUR_PROJECT

gcloud container clusters get-credentials $YOUR_CLUSTER --region asia-northeast1 --project $YOUR_PROJECT
gcloud config set container/cluster $YOUR_CLUSTER

# enable workload identity
kubectl create serviceaccount --namespace default example
gcloud iam service-accounts add-iam-policy-binding ${YOUR_ACCOUNT}@${YOUR_PROJECT}.iam.gserviceaccount.com \
    --role roles/iam.workloadIdentityUser \
    --member "serviceAccount:${YOUR_PROJECT}.svc.id.goog[default/example]"
kubectl annotate serviceaccount --namespace default example \
    iam.gke.io/gcp-service-account=${YOUR_ACCOUNT}@${YOUR_PROJECT}.iam.gserviceaccount.com 
``` 

### Deploy k8s
```
cd rust
docker build -t asia.gcr.io/${YOUR_PROJECT}/rust-api:latest .
docker push asia.gcr.io/${YOUR_PROJECT}/rust-api:latest
sed -e "s/<your_project>/${YOUR_PROJECT}/" k8s.tmpl.yaml > k8s.yaml
kubectl apply -f k8s.yaml

cd scenario
docker build -t asia.gcr.io/${YOUR_PROJECT}/loadtest:latest .
docker push asia.gcr.io/${YOUR_PROJECT}/loadtest:latest

cd ..
sed -e "s/<your_project>/${YOUR_PROJECT}/" k8s-loadteset.tmpl.yaml > k8s-loadtest.yaml
kubectl apply -f k8s-loadtest.yaml
sed -e "s/<your_project>/${YOUR_PROJECT}/" k8s-rust.tmpl.yaml > k8s-rust.yaml
kubectl apply -f k8s-rust.yaml
```