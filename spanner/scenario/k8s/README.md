```
gcloud container clusters get-credentials google-cloud-rust-example --region asia-northeast1 --project atl-dev1
gcloud config set container/cluster google-cloud-rust-example
```

```
kubectl apply -f kube/locust-master-deployment.yaml -n app
kubectl apply -f kube/locust-worker-deployment.yaml -n app
```

