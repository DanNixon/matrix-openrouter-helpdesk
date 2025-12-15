# matrix-openrouter-helpdesk

A Matrix bot written in Rust that responds to messages with AI-generated answers using OpenRouter.

## Features

- **Automatic Room Invites**: The bot automatically accepts room invitations
- **Message Processing**: Responds to messages in the format `@bot_username question` (case insensitive)
- **Visual Feedback**: Adds a "👀" (eyes) emoji reaction to messages it's processing
- **AI-Powered Responses**: Uses OpenRouter API to generate intelligent responses
- **Self-Aware**: Ignores its own messages to prevent loops
- **Prometheus Metrics**: Exposes metrics in Prometheus format for monitoring
- **Template Support**: Customize question and reply formatting with Handlebars templates
- **Pretty Error Messages**: Uses miette for beautiful, helpful error reporting

## Prerequisites

- Rust 1.70 or later
- A Matrix account for the bot
- An OpenRouter API key

## Configuration

The bot is configured using environment variables or command-line arguments (parsed with [clap](https://github.com/clap-rs/clap)):

- `MATRIX_HOMESERVER_URL` / `--matrix-homeserver-url`: The Matrix homeserver URL (e.g., `https://matrix.org`)
- `MATRIX_USERNAME` / `--matrix-username`: The bot's Matrix username (e.g., `@helpdesk:matrix.org`)
- `MATRIX_PASSWORD` / `--matrix-password`: The bot's Matrix password
- `OPENROUTER_API_KEY` / `--openrouter-api-key`: Your OpenRouter API key
- `OPENROUTER_MODEL` / `--openrouter-model` (optional): The OpenRouter model to use (defaults to `openai/gpt-3.5-turbo`)
- `METRICS_PORT` / `--metrics-port` (optional): Port for Prometheus metrics endpoint (defaults to `9090`)
- `QUESTION_TEMPLATE_FILE` / `--question-template-file` (optional): Path to a file containing the Handlebars template for formatting questions sent to OpenRouter (defaults to `{{ query }}`)
- `REPLY_TEMPLATE_FILE` / `--reply-template-file` (optional): Path to a file containing the Handlebars template for formatting replies sent to Matrix (defaults to `{{ response }}`)

Run with `--help` to see all available options.

## Building

### From Source

```bash
cargo build --release
```

### Container Image

```bash
podman build -f Containerfile -t matrix-openrouter-helpdesk:latest .
```

## Running

### From Source

```bash
export MATRIX_HOMESERVER_URL="https://matrix.org"
export MATRIX_USERNAME="@your-bot:matrix.org"
export MATRIX_PASSWORD="your-password"
export OPENROUTER_API_KEY="your-openrouter-api-key"
export OPENROUTER_MODEL="openai/gpt-3.5-turbo"  # optional

cargo run --release
```

Or using environment file:

```bash
# Create a .env file with your configuration
cat > .env << EOF
MATRIX_HOMESERVER_URL=https://matrix.org
MATRIX_USERNAME=@your-bot:matrix.org
MATRIX_PASSWORD=your-password
OPENROUTER_API_KEY=your-openrouter-api-key
OPENROUTER_MODEL=openai/gpt-3.5-turbo
EOF

# Load environment variables and run
set -a; source .env; set +a
cargo run --release
```

### Using Container

```bash
podman run -d \
  -e MATRIX_HOMESERVER_URL="https://matrix.org" \
  -e MATRIX_USERNAME="@your-bot:matrix.org" \
  -e MATRIX_PASSWORD="your-password" \
  -e OPENROUTER_API_KEY="your-openrouter-api-key" \
  -e OPENROUTER_MODEL="openai/gpt-3.5-turbo" \
  -p 9090:9090 \
  matrix-openrouter-helpdesk:latest
```

To use custom templates with the container, mount them as volumes:

```bash
podman run -d \
  -e MATRIX_HOMESERVER_URL="https://matrix.org" \
  -e MATRIX_USERNAME="@your-bot:matrix.org" \
  -e MATRIX_PASSWORD="your-password" \
  -e OPENROUTER_API_KEY="your-openrouter-api-key" \
  -e QUESTION_TEMPLATE_FILE=/templates/question.hbs \
  -e REPLY_TEMPLATE_FILE=/templates/reply.hbs \
  -v ./templates:/templates:ro \
  -p 9090:9090 \
  matrix-openrouter-helpdesk:latest
```

## Usage

1. Invite the bot to a Matrix room
2. The bot will automatically join the room
3. Send a message mentioning the bot: `@bot_username What is the weather today?`
4. The bot will:
   - React with 👀 to indicate it's processing
   - Send a quoted reply with the AI-generated response

## Example

```
You: @helpdesk:matrix.org What is Rust?
Bot: 👀 (reaction)
Bot: > What is Rust?
     Rust is a systems programming language focused on safety, speed, and concurrency...
```

## Template Customization

You can customize how questions are sent to OpenRouter and how responses are formatted using Handlebars templates loaded from files.

### Using Template Files

Templates are loaded from files specified by environment variables. This makes multi-line templates much easier to manage:

**Question Template Example:**

Create a file `question_template.hbs`:
```handlebars
You are a helpful assistant with expertise in technology.

Please answer the following question:
{{ query }}

Provide a clear and concise response.
```

Then set the environment variable:
```bash
export QUESTION_TEMPLATE_FILE=/path/to/question_template.hbs
```

**Reply Template Example:**

Create a file `reply_template.hbs`:
```handlebars
📝 **Response:**

{{ response }}

---
*Powered by OpenRouter*
```

Then set the environment variable:
```bash
export REPLY_TEMPLATE_FILE=/path/to/reply_template.hbs
```

### Simple Template Example

For a simple customization, create `question_template.hbs`:
```handlebars
Using only information from wikipedia.org, answer this question: {{ query }}
```

If a user asks "what is rust?", the text sent to OpenRouter will be:
```
Using only information from wikipedia.org, answer this question: what is rust?
```

### Available Template Variables

**Question Template:**
- `{{ query }}`: The user's question (without the bot mention)

**Reply Template:**
- `{{ response }}`: The response from OpenRouter

## Metrics

The bot exposes Prometheus metrics on the configured port (default: 9090). The metrics endpoint is available at `http://localhost:9090/metrics`.

### Available Metrics

- `helpdesk_requests_total`: Counter tracking total number of requests processed
  - Labels:
    - `matrix_user`: The Matrix user ID who sent the request
    - `matrix_room`: The Matrix room ID where the request was made
    - `result`: Either `success` or `failure`

Example Prometheus query:
```promql
# Total successful requests
helpdesk_requests_total{result="success"}

# Request rate per room
rate(helpdesk_requests_total[5m])
```

## License

This project is open source and available under your preferred license.