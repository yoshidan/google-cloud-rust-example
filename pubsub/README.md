# Pub/Sub Example

## Run on GKE

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
gcloud container clusters get-credentials $YOUR_CLUSTER --region asia-northeast1 --project $YOUR_PROJECT
gcloud config set container/cluster $YOUR_CLUSTER

cd rust
docker build -t asia.gcr.io/${YOUR_PROJECT}/rust-ws:latest .
docker push asia.gcr.io/${YOUR_PROJECT}/rust-ws:latest

cd ..
sed -e "s/<your_project>/${YOUR_PROJECT}/" k8s-rust.tmpl.yaml > k8s-rust.gen.yaml
kubectl apply -f k8s-rust.gen.yaml
```

### Connect
```
export SERVICE_ENDPOINT=`kubectl describe services rust-ws | grep 'LoadBalancer Ingress' | awk '{print $3}'`
wscat -c "ws://${SERVICE_ENDPOINT}:8091/Connect?channelId=012345678901234567890123456789abcdefg&userId=hogehoge2"
wscat -c "ws://${SERVICE_ENDPOINT}:8091/Connect?channelId=012345678901234567890123456789abcdefg&userId=hogehoge1"
```