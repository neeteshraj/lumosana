#!/bin/bash

# Deploy to Kubernetes

echo "Deploying Transaction Debugger to Kubernetes..."

kubectl create namespace transaction-debugger --dry-run=client -o yaml | kubectl apply -f -

echo "Deploying Jaeger..."
kubectl apply -f k8s/jaeger.yaml -n transaction-debugger

echo "Deploying ConfigMap..."
kubectl apply -f k8s/configmap.yaml -n transaction-debugger

echo "Deploying Transaction Debugger..."
kubectl apply -f k8s/deployment.yaml -n transaction-debugger
kubectl apply -f k8s/service.yaml -n transaction-debugger

echo "Deploying Envoy proxy..."
kubectl apply -f k8s/envoy.yaml -n transaction-debugger

echo ""
echo "Deployment completed!"
echo ""
echo "Check deployment status:"
echo "kubectl get pods -n transaction-debugger"
echo ""
echo "Access the service:"
echo "kubectl port-forward svc/transaction-debugger-lb 8080:80 -n transaction-debugger"
echo ""
echo "Access Jaeger UI:"
echo "kubectl port-forward svc/jaeger-ui 16686:80 -n transaction-debugger"
