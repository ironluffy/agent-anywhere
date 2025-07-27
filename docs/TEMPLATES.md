# Agent Templates

Agent templates allow you to quickly create pre-configured agents for common use cases.

## Using Templates

### From the UI

When creating a new agent, you can optionally specify a template:
1. Press `n` to create new agent
2. Enter agent name
3. When prompted for template, enter the template name

### Available Templates

Templates are stored in:
1. Pool-specific: `<pool-root>/templates/`
2. Global: `~/.aany/templates/`

## Template Structure

Each template is a YAML file with the following structure:

```yaml
# Template metadata
name: "web-dev"
description: "Web development environment with Node.js"

# Agent configuration
agent:
  type: "development"
  description: "Web development agent with Node.js and common tools"

# Claude AI configuration
claude:
  model: "claude-3-opus-20240229"
  memory_enabled: true
  tools:
    - "file_operations"
    - "web_search"

# Resource limits
resources:
  max_context_size: 200000

# Environment variables
environment:
  NODE_ENV: "development"
  PORT: "3000"

# Initialization commands (run when agent starts)
init_commands:
  - "nvm install 22"
  - "nvm use 22"
  - "npm install -g pnpm"
```

## Creating Custom Templates

### 1. Create Template File

Create a YAML file in the templates directory:

```bash
# For pool-specific template
mkdir -p ./agent-pool/templates
vim ./agent-pool/templates/my-template.yaml

# For global template
mkdir -p ~/.aany/templates
vim ~/.aany/templates/my-template.yaml
```

### 2. Define Template

Example Python development template:

```yaml
name: "python-dev"
description: "Python development environment"

agent:
  type: "development"
  description: "Python development agent with poetry and testing tools"

claude:
  model: "claude-3-opus-20240229"
  memory_enabled: true
  tools:
    - "file_operations"
    - "terminal"

environment:
  PYTHON_VERSION: "3.11"
  POETRY_VIRTUALENVS_IN_PROJECT: "true"

init_commands:
  - "pyenv install 3.11.0"
  - "pyenv local 3.11.0"
  - "pip install poetry"
  - "poetry install"
```

### 3. Use Your Template

```bash
aany pool
# Press 'n' for new agent
# Enter name
# Enter template name: python-dev
```

## Example Templates

### Data Science Template

```yaml
name: "data-science"
description: "Data science environment with Jupyter and common libraries"

agent:
  type: "analysis"
  description: "Data science agent with Jupyter, pandas, and ML tools"

claude:
  model: "claude-3-opus-20240229"
  memory_enabled: true
  tools:
    - "file_operations"
    - "data_analysis"
    - "visualization"

environment:
  JUPYTER_PORT: "8888"
  PYTHON_VERSION: "3.10"

init_commands:
  - "conda create -n ds python=3.10 -y"
  - "conda activate ds"
  - "pip install jupyter pandas numpy scikit-learn matplotlib"
  - "jupyter notebook --no-browser --port=8888"
```

### DevOps Template

```yaml
name: "devops"
description: "DevOps environment with Docker and Kubernetes tools"

agent:
  type: "infrastructure"
  description: "DevOps agent with containerization and orchestration tools"

claude:
  model: "claude-3-opus-20240229"
  memory_enabled: true
  tools:
    - "file_operations"
    - "terminal"
    - "docker"
    - "kubernetes"

environment:
  KUBECONFIG: "$HOME/.kube/config"
  DOCKER_HOST: "unix:///var/run/docker.sock"

init_commands:
  - "kubectl version --client"
  - "docker --version"
  - "helm version"
```

### API Development Template

```yaml
name: "api-dev"
description: "API development with testing and documentation"

agent:
  type: "backend"
  description: "API development agent with testing and documentation tools"

claude:
  model: "claude-3-opus-20240229"
  memory_enabled: true
  tools:
    - "file_operations"
    - "api_testing"
    - "documentation"

environment:
  API_ENV: "development"
  DATABASE_URL: "postgresql://localhost/api_dev"
  REDIS_URL: "redis://localhost:6379"

init_commands:
  - "docker-compose up -d postgres redis"
  - "npm install"
  - "npm run db:migrate"
```

## Template Best Practices

### 1. Keep Templates Focused

Each template should serve a specific purpose. Don't try to create a "do everything" template.

### 2. Use Environment Variables

Make templates flexible by using environment variables that can be overridden:

```yaml
environment:
  DATABASE_URL: "${DATABASE_URL:-postgresql://localhost/dev}"
  API_KEY: "${API_KEY:-development-key}"
```

### 3. Document Requirements

Include comments about external requirements:

```yaml
# Requires: Docker, Node.js 18+
# Optional: PostgreSQL for local development
```

### 4. Version Control Templates

Store templates in version control for sharing and collaboration:

```bash
git init ~/.aany/templates
git add .
git commit -m "Add development templates"
```

### 5. Test Templates

Always test templates before sharing:
1. Create an agent with the template
2. Verify all init commands work
3. Check environment variables are set correctly

## Sharing Templates

### Export a Template

```bash
# Copy template to share
cp ~/.aany/templates/my-template.yaml ./my-template.yaml
```

### Import a Template

```bash
# Add to global templates
cp downloaded-template.yaml ~/.aany/templates/

# Or add to pool templates
cp downloaded-template.yaml ./agent-pool/templates/
```

## Template Variables

Templates support variable substitution:

- `{{agent_name}}` - The name of the agent
- `{{timestamp}}` - Current timestamp
- `{{user}}` - Current user name

Example:
```yaml
environment:
  LOG_FILE: "/tmp/{{agent_name}}-{{timestamp}}.log"
  OWNER: "{{user}}"
```

## Advanced Template Features

### Conditional Initialization

You can make initialization conditional:

```yaml
init_commands:
  - "[ -f package.json ] && npm install || echo 'No package.json found'"
  - "[ -d .git ] || git init"
```

### Template Inheritance

Templates can extend other templates (planned feature):

```yaml
extends: "base-dev"
name: "frontend-dev"
# Additional configuration...
```

## Troubleshooting Templates

### Template Not Found

1. Check template exists in correct directory
2. Verify template name matches filename (without .yaml)
3. Check file permissions

### Init Commands Failing

1. Test commands manually in agent workspace
2. Add error handling to commands
3. Check prerequisites are installed

### Environment Variables Not Set

1. Verify syntax in template
2. Check for typos in variable names
3. Look for conflicts with existing variables