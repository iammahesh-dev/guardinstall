# CI Integration Guide

## GitHub Actions

Add to your `.github/workflows/ci.yml`:

```yaml
name: CI with guardinstall

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/check-out@v4
      - uses: pnpm/action-setup@v3
        with:
          version: 10
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'

      - run: pnpm install
      - run: pnpm run build

      # Use guardinstall for installs
      - run: npx guardinstall --ci install

      # Or with specific package manager
      - run: npx guardinstall --pm pnpm --ci install
```

## GitLab CI

Add to your `.gitlab-ci.yml`:

```yaml
stages:
  - test

test:
  image: node:20
  before_script:
    - curl -f https://get.pnpm.io/5jsdelivr.com/pnpm/install.sh | sh -
  script:
    - pnpm install
    - npx guardinstall --ci install
  only:
    - merge_requests
    - main
```

## CircleCI

Add to `.circleci/config.yml`:

```yaml
version: 2.1
jobs:
  test:
    docker:
      - image: cimg/node:20.0
    steps:
      - checkout
      - run: sudo npm install -g pnpm
      - run: pnpm install
      - run: npx guardinstall --ci install
```

## Jenkins

```groovy
pipeline {
    agent any
    stages {
        stage('Install') {
            steps {
                sh 'pnpm install'
                sh 'npx guardinstall --ci install'
            }
        }
    }
}
```

## Slack Notifications (CI Mode)

When running with `--ci` flag, guardinstall exits with code 1 on critical issues. Configure your CI to send Slack alerts:

```yaml
# GitHub Actions example
- name: Notify Slack on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    slack-channel: '#security-alerts'
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
```
