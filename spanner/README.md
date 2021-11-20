# Deploy TO GCP
```
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
