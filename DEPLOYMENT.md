# Exam Document Converter - Deployment Guide

## Overview

This guide covers deploying the Exam Document Converter application locally using Minikube or Kind, and in production using a Kubernetes cluster with Jenkins CI/CD.

## Architecture

- **Frontend**: React TypeScript application (Port 5173)
- **Python Service**: Document analysis service (Port 8001)
- **Rust Service**: Document conversion service (Port 8002)

## Prerequisites

### Local Development
- Docker Desktop
- Minikube or Kind
- kubectl
- Node.js 18+
- Python 3.11+
- Rust 1.75+

### Production
- Kubernetes cluster
- Jenkins with Docker and Kubernetes plugins
- Docker registry access
- kubectl configured for your cluster

## Local Deployment with Minikube

### 1. Start Minikube

```bash
# Start Minikube with sufficient resources
minikube start --memory=4096 --cpus=2

# Enable ingress addon
minikube addons enable ingress

# Enable registry addon (optional, for local images)
minikube addons enable registry
```

### 2. Build Docker Images

```bash
# Build frontend image
docker build -t exam-converter/frontend:latest .

# Build Python service image
docker build -t exam-converter/python-analyzer:latest ./python-wasm

# Build Rust service image
docker build -t exam-converter/rust-converter:latest ./rust-wasm

# Load images into Minikube
minikube image load exam-converter/frontend:latest
minikube image load exam-converter/python-analyzer:latest
minikube image load exam-converter/rust-converter:latest
```

### 3. Deploy to Kubernetes

```bash
# Apply all Kubernetes manifests
kubectl apply -f k8s/

# Wait for deployments to be ready
kubectl wait --for=condition=available --timeout=300s deployment --all -n exam-converter

# Check pod status
kubectl get pods -n exam-converter
```

### 4. Access the Application

```bash
# Get Minikube IP
minikube ip

# Add to /etc/hosts (replace <MINIKUBE_IP> with actual IP)
echo "<MINIKUBE_IP> exam-converter.local" | sudo tee -a /etc/hosts

# Access the application
open http://exam-converter.local
```

## Local Deployment with Kind

### 1. Create Kind Cluster

```bash
# Create cluster with ingress support
cat <<EOF | kind create cluster --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
EOF

# Install NGINX Ingress Controller
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml

# Wait for ingress controller to be ready
kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=90s
```

### 2. Build and Load Images

```bash
# Build images
docker build -t exam-converter/frontend:latest .
docker build -t exam-converter/python-analyzer:latest ./python-wasm
docker build -t exam-converter/rust-converter:latest ./rust-wasm

# Load images into Kind
kind load docker-image exam-converter/frontend:latest
kind load docker-image exam-converter/python-analyzer:latest
kind load docker-image exam-converter/rust-converter:latest
```

### 3. Deploy Application

```bash
# Deploy all services
kubectl apply -f k8s/

# Wait for deployments
kubectl wait --for=condition=available --timeout=300s deployment --all -n exam-converter

# Check status
kubectl get all -n exam-converter
```

### 4. Access Application

```bash
# Add to /etc/hosts
echo "127.0.0.1 exam-converter.local" | sudo tee -a /etc/hosts

# Access the application
open http://exam-converter.local
```

## Production Deployment with Jenkins

### 1. Jenkins Setup

Install required plugins:
- Docker Pipeline
- Kubernetes CLI
- Git

Configure credentials:
- `docker-registry-credentials`: Docker registry username/password
- `kubeconfig`: Kubernetes cluster configuration

### 2. Create Jenkins Pipeline

1. Create a new Pipeline job in Jenkins
2. Configure Git repository URL
3. Set Pipeline script path to `Jenkinsfile`
4. Configure webhook for automatic builds on git push

### 3. Environment Variables

Set these in Jenkins or your Kubernetes cluster:

```bash
# Docker registry configuration
DOCKER_REGISTRY=your-registry.com
DOCKER_REPO=exam-converter

# Kubernetes configuration
KUBECONFIG=/path/to/kubeconfig
```

### 4. Deploy

Push code to trigger Jenkins pipeline:

```bash
git add .
git commit -m "Deploy exam converter application"
git push origin main
```

## Monitoring and Troubleshooting

### Check Application Health

```bash
# Check all pods
kubectl get pods -n exam-converter

# Check service endpoints
kubectl get endpoints -n exam-converter

# Check ingress
kubectl get ingress -n exam-converter

# View logs
kubectl logs -f deployment/frontend -n exam-converter
kubectl logs -f deployment/python-analyzer -n exam-converter
kubectl logs -f deployment/rust-converter -n exam-converter
```

### Common Issues

1. **Images not found**: Ensure images are built and available in the registry
2. **Pod crashes**: Check resource limits and application logs
3. **Service not accessible**: Verify service selectors and port configurations
4. **Ingress not working**: Ensure ingress controller is installed and running

### Scaling

```bash
# Scale deployments
kubectl scale deployment frontend --replicas=3 -n exam-converter
kubectl scale deployment python-analyzer --replicas=3 -n exam-converter
kubectl scale deployment rust-converter --replicas=3 -n exam-converter
```

### Updates

```bash
# Rolling update
kubectl set image deployment/frontend frontend=exam-converter/frontend:new-tag -n exam-converter

# Check rollout status
kubectl rollout status deployment/frontend -n exam-converter

# Rollback if needed
kubectl rollout undo deployment/frontend -n exam-converter
```

## Security Considerations

1. **Network Policies**: Implement network policies to restrict inter-pod communication
2. **RBAC**: Configure proper role-based access control
3. **Secrets Management**: Use Kubernetes secrets for sensitive data
4. **Image Security**: Scan Docker images for vulnerabilities
5. **TLS**: Configure TLS certificates for production ingress

## Performance Optimization

1. **Resource Limits**: Set appropriate CPU and memory limits
2. **Horizontal Pod Autoscaler**: Configure HPA for automatic scaling
3. **Persistent Volumes**: Use PVs for file storage if needed
4. **CDN**: Use a CDN for static assets in production
5. **Caching**: Implement Redis for caching converted documents

## Backup and Recovery

1. **Database Backups**: If using persistent storage, configure regular backups
2. **Configuration Backups**: Version control all Kubernetes manifests
3. **Disaster Recovery**: Document recovery procedures
4. **Monitoring**: Set up monitoring and alerting with Prometheus/Grafana