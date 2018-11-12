pipeline {
  agent any
  stages {
    stage('Build paste') {
      agent {
        dockerfile {
          filename 'Dockerfile'
          dir '.docker/buildenv'
          args '-u root:root' // required so that Jenkins runs the container as root. May not be safe.
        }
      }
      steps {
        sh '''#!/bin/bash
              source /root/.bashrc
              /root/.cargo/bin/cargo build --all --release
              cd target/release/
              strip webserver
              strip libworker_*.so
              cd ../../
              rm -rf exec
              mkdir -p exec
              cd exec
              cp ../target/release/{webserver,libworker_*.so} ./
              find . -name "*.so" -print0 | xargs -0 shasum >> shasums
              cd ../
              cp .docker/paste/* .
              cp .gitignore .dockerignore
              chmod +x ./run.sh
        '''
        archiveArtifacts artifacts: 'target/release/webserver, target/release/libworker_*.so', fingerprint: true
      }
    }
    stage('Build paste Docker Container') {
      agent {
        docker {
          image 'docker:latest'
          args '-u root:root -v /var/run/docker.sock:/var/run/docker.sock --privileged'
        }
      }
      steps {
        withDockerRegistry(url: 'https://index.docker.io/v1/', credentialsId: 'docker-hub-credentials') {
          sh 'docker build -t jkcclemens/paste-prebuilt .'
          sh 'docker push jkcclemens/paste-prebuilt'
        }
      }
    }
  }
  post {
    always {
      cleanWs(patterns: [[pattern: 'target/**', type: 'EXCLUDE'], [pattern: '.git/**', type: 'EXCLUDE']])
    }
  }
}
