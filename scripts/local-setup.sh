#!/bin/bash

# Local Development Setup Script for Exam Document Converter

set -e

echo "🚀 Setting up Exam Document Converter locally..."

# Check prerequisites
check_prerequisites() {
    echo "📋 Checking prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        echo "❌ Docker is not installed. Please install Docker Desktop."
        exit 1
    fi
    
    if ! command -v kubectl &> /dev/null; then
        echo "❌ kubectl is not installed. Please install kubectl."
        exit 1
    fi
    
    echo "✅ Prerequisites check passed!"
}

# Setup with Docker Compose (Quick Start)
setup_docker_compose() {
    echo "🐳 Setting up with Docker Compose..."
    
    # Build and start services
    docker-compose up --build -d
    
    echo "⏳ Waiting for services to be ready..."
    sleep 30
    
    # Check service health
    echo "🔍 Checking service health..."
    curl -f http://localhost:8001/health || echo "⚠️  Python service not ready"
    curl -f http://localhost:8002/health || echo "⚠️  Rust service not ready"
    
    echo "✅ Docker Compose setup complete!"
    echo "🌐 Frontend: http://localhost:5173"
    echo "🐍 Python API: http://localhost:8001"
    echo "🦀 Rust API: http://localhost:8002"
}

# Setup with Minikube
setup_minikube() {
    echo "☸️  Setting up with Minikube..."
    
    # Start Minikube
    minikube start --memory=4096 --cpus=2
    
    # Enable addons
    minikube addons enable ingress
    
    # Build and load images
    echo "🏗️  Building Docker images..."
    docker build -t exam-converter/frontend:latest .
    docker build -t exam-converter/python-analyzer:latest ./python-wasm
    docker build -t exam-converter/rust-converter:latest ./rust-wasm
    
    echo "📦 Loading images into Minikube..."
    minikube image load exam-converter/frontend:latest
    minikube image load exam-converter/python-analyzer:latest
    minikube image load exam-converter/rust-converter:latest
    
    # Deploy to Kubernetes
    echo "🚀 Deploying to Kubernetes..."
    kubectl apply -f k8s/
    
    # Wait for deployments
    echo "⏳ Waiting for deployments to be ready..."
    kubectl wait --for=condition=available --timeout=300s deployment --all -n exam-converter
    
    # Setup ingress
    MINIKUBE_IP=$(minikube ip)
    echo "🌐 Adding exam-converter.local to /etc/hosts..."
    echo "$MINIKUBE_IP exam-converter.local" | sudo tee -a /etc/hosts
    
    echo "✅ Minikube setup complete!"
    echo "🌐 Application: http://exam-converter.local"
}

# Setup with Kind
setup_kind() {
    echo "🔧 Setting up with Kind..."
    
    # Create Kind cluster with ingress support
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
    
    # Wait for ingress controller
    kubectl wait --namespace ingress-nginx \
      --for=condition=ready pod \
      --selector=app.kubernetes.io/component=controller \
      --timeout=90s
    
    # Build and load images
    echo "🏗️  Building Docker images..."
    docker build -t exam-converter/frontend:latest .
    docker build -t exam-converter/python-analyzer:latest ./python-wasm
    docker build -t exam-converter/rust-converter:latest ./rust-wasm
    
    echo "📦 Loading images into Kind..."
    kind load docker-image exam-converter/frontend:latest
    kind load docker-image exam-converter/python-analyzer:latest
    kind load docker-image exam-converter/rust-converter:latest
    
    # Deploy application
    echo "🚀 Deploying to Kubernetes..."
    kubectl apply -f k8s/
    
    # Wait for deployments
    kubectl wait --for=condition=available --timeout=300s deployment --all -n exam-converter
    
    # Setup hosts file
    echo "🌐 Adding exam-converter.local to /etc/hosts..."
    echo "127.0.0.1 exam-converter.local" | sudo tee -a /etc/hosts
    
    echo "✅ Kind setup complete!"
    echo "🌐 Application: http://exam-converter.local"
}

# Main menu
main() {
    check_prerequisites
    
    echo ""
    echo "Choose deployment method:"
    echo "1) Docker Compose (Quick Start)"
    echo "2) Minikube (Full Kubernetes)"
    echo "3) Kind (Lightweight Kubernetes)"
    echo ""
    read -p "Enter your choice (1-3): " choice
    
    case $choice in
        1)
            setup_docker_compose
            ;;
        2)
            setup_minikube
            ;;
        3)
            setup_kind
            ;;
        *)
            echo "❌ Invalid choice. Please run the script again."
            exit 1
            ;;
    esac
    
    echo ""
    echo "🎉 Setup complete! Your Exam Document Converter is ready to use."
    echo ""
    echo "📚 Next steps:"
    echo "1. Open the application in your browser"
    echo "2. Select an exam type (NEET, JEE, UPSC, CAT, GATE)"
    echo "3. Upload your documents"
    echo "4. Click 'Convert Documents' to process them"
    echo "5. Download the converted files"
    echo ""
    echo "🔧 Troubleshooting:"
    echo "- Check logs: kubectl logs -f deployment/<service-name> -n exam-converter"
    echo "- Check pods: kubectl get pods -n exam-converter"
    echo "- Check services: kubectl get svc -n exam-converter"
}

# Run main function
main "$@"