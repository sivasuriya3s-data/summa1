pipeline {
    agent any
    
    environment {
        DOCKER_REGISTRY = 'your-registry.com'
        DOCKER_REPO = 'exam-converter'
        KUBECONFIG = credentials('kubeconfig')
        DOCKER_CREDENTIALS = credentials('docker-registry-credentials')
    }
    
    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }
        
        stage('Build Frontend') {
            steps {
                script {
                    def frontendImage = docker.build("${DOCKER_REGISTRY}/${DOCKER_REPO}/frontend:${BUILD_NUMBER}")
                    docker.withRegistry("https://${DOCKER_REGISTRY}", DOCKER_CREDENTIALS) {
                        frontendImage.push()
                        frontendImage.push("latest")
                    }
                }
            }
        }
        
        stage('Build Python Service') {
            steps {
                script {
                    def pythonImage = docker.build("${DOCKER_REGISTRY}/${DOCKER_REPO}/python-analyzer:${BUILD_NUMBER}", "./python-wasm")
                    docker.withRegistry("https://${DOCKER_REGISTRY}", DOCKER_CREDENTIALS) {
                        pythonImage.push()
                        pythonImage.push("latest")
                    }
                }
            }
        }
        
        stage('Build Rust Service') {
            steps {
                script {
                    def rustImage = docker.build("${DOCKER_REGISTRY}/${DOCKER_REPO}/rust-converter:${BUILD_NUMBER}", "./rust-wasm")
                    docker.withRegistry("https://${DOCKER_REGISTRY}", DOCKER_CREDENTIALS) {
                        rustImage.push()
                        rustImage.push("latest")
                    }
                }
            }
        }
        
        stage('Update Kubernetes Manifests') {
            steps {
                script {
                    // Update image tags in Kubernetes manifests
                    sh """
                        sed -i 's|image: exam-converter/frontend:.*|image: ${DOCKER_REGISTRY}/${DOCKER_REPO}/frontend:${BUILD_NUMBER}|g\' k8s/frontend-deployment.yaml
                        sed -i 's|image: exam-converter/python-analyzer:.*|image: ${DOCKER_REGISTRY}/${DOCKER_REPO}/python-analyzer:${BUILD_NUMBER}|g\' k8s/python-deployment.yaml
                        sed -i 's|image: exam-converter/rust-converter:.*|image: ${DOCKER_REGISTRY}/${DOCKER_REPO}/rust-converter:${BUILD_NUMBER}|g\' k8s/rust-deployment.yaml
                    """
                }
            }
        }
        
        stage('Deploy to Kubernetes') {
            steps {
                script {
                    withKubeConfig([credentialsId: 'kubeconfig']) {
                        sh '''
                            # Apply namespace first
                            kubectl apply -f k8s/namespace.yaml
                            
                            # Apply all deployments and services
                            kubectl apply -f k8s/frontend-deployment.yaml
                            kubectl apply -f k8s/python-deployment.yaml
                            kubectl apply -f k8s/rust-deployment.yaml
                            kubectl apply -f k8s/ingress.yaml
                            
                            # Wait for deployments to be ready
                            kubectl rollout status deployment/frontend -n exam-converter --timeout=300s
                            kubectl rollout status deployment/python-analyzer -n exam-converter --timeout=300s
                            kubectl rollout status deployment/rust-converter -n exam-converter --timeout=300s
                            
                            # Verify deployment
                            kubectl get pods -n exam-converter
                        '''
                    }
                }
            }
        }
        
        stage('Health Check') {
            steps {
                script {
                    sh '''
                        # Wait for services to be ready
                        sleep 30
                        
                        # Check service health
                        kubectl exec -n exam-converter deployment/python-analyzer -- curl -f http://localhost:8001/health || exit 1
                        kubectl exec -n exam-converter deployment/rust-converter -- curl -f http://localhost:8002/health || exit 1
                    '''
                }
            }
        }
    }
    
    post {
        always {
            // Clean up Docker images to save space
            sh 'docker system prune -f'
        }
        success {
            echo 'Deployment successful!'
            // Send notification (Slack, email, etc.)
        }
        failure {
            echo 'Deployment failed!'
            // Send failure notification
            // Optionally rollback to previous version
        }
    }
}